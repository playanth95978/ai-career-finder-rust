//! Calcul et persistance des vecteurs d'offres, plus le poller qui rattrape les offres en attente.
//!
//! Transcription du couple `JobOfferEmbeddingService` / poller `@Scheduled` de l'application
//! Spring. L'ingestion marque les offres `PENDING` ; ce service les passe en `COMPLETED` une fois
//! le vecteur ecrit, ce qui est exactement ce que compte `/jobs/indexed-count`.
//!
//! L'embedding n'est volontairement pas calcule dans le chemin de la requete de recherche : le
//! modele met plusieurs centaines de millisecondes par offre, et une recherche qui ramene
//! cinquante nouvelles annonces ne doit pas attendre cinquante appels reseau.

use chrono::Utc;
use diesel::prelude::*;
use pgvector::Vector;
use uuid::Uuid;

use crate::db::connection::DbPool;
use crate::db::schema::job_offer;
use crate::db::DbConnection;
use crate::errors::AppError;
use crate::services::embedding_service::{EmbeddingService, EMBEDDING_DIMENSIONS};
use crate::services::job_search_service::{
    EMBEDDING_STATUS_COMPLETED, EMBEDDING_STATUS_FAILED, EMBEDDING_STATUS_PENDING,
};

/// Nombre d'offres traitees par reveil du poller. Volontairement modeste : chaque offre coute un
/// appel au modele d'embedding, et un lot trop gros retarderait d'autant l'arret du serveur.
const BATCH_SIZE: i64 = 25;

/// Intervalle entre deux passages, aligne sur le poller Java.
const POLL_INTERVAL_SECS: u64 = 60;

/// Au-dela de ce nombre d'echecs, l'offre est marquee `FAILED` et n'est plus retentee.
///
/// Sans plafond, une annonce dont le texte fait systematiquement echouer le modele serait
/// reprise a chaque reveil, indefiniment, en consommant tout le lot.
const MAX_RETRIES: i32 = 3;

/// Longueur de description retenue dans le texte a vectoriser. Les annonces scrapees embarquent
/// des mentions legales et du boilerplate : au-dela, le vecteur decrit le pied de page de
/// l'entreprise plutot que le poste.
const DESCRIPTION_MAX_CHARS: usize = 2_000;

pub struct JobOfferEmbeddingService;

impl JobOfferEmbeddingService {
    /// Texte vectorise pour une offre : intitule, entreprise, lieu, competences, description.
    ///
    /// Le titre est en tete parce que c'est le signal le plus dense, et il est repete dans le
    /// texte structure de la meme facon que `EmbeddingService::build_profile_text` cote profil,
    /// pour que les deux espaces vectoriels restent comparables.
    pub fn build_offer_text(
        title: &str,
        company: Option<&str>,
        location: Option<&str>,
        skills: Option<&str>,
        description: Option<&str>,
    ) -> String {
        let mut parts: Vec<String> = Vec::new();
        parts.push(format!("Poste: {}", title.trim()));

        if let Some(company) = company.map(str::trim).filter(|s| !s.is_empty()) {
            parts.push(format!("Entreprise: {company}"));
        }
        if let Some(location) = location.map(str::trim).filter(|s| !s.is_empty()) {
            parts.push(format!("Lieu: {location}"));
        }

        let skills = crate::dto::parse_string_list(skills);
        if !skills.is_empty() {
            parts.push(format!("Competences: {}", skills.join(", ")));
        }

        if let Some(description) = description.map(str::trim).filter(|s| !s.is_empty()) {
            parts.push(format!("Description: {}", truncate(description, DESCRIPTION_MAX_CHARS)));
        }

        parts.join("\n")
    }

    /// Vectorise une offre et enregistre le resultat.
    ///
    /// Idempotent : une offre deja `COMPLETED` est ignoree, pour que relancer le poller ou
    /// reindexer ne repaye pas les appels au modele.
    pub async fn embed_offer(
        conn: &mut DbConnection,
        offer_id: Uuid,
        force: bool,
    ) -> Result<bool, AppError> {
        let row: Option<PendingOffer> = job_offer::table
            .find(offer_id)
            .select(PENDING_COLUMNS)
            .first(conn)
            .optional()?;

        let Some(offer) = row else {
            return Err(AppError::NotFound("Job offer not found".into()));
        };

        if !force && offer.status.as_deref() == Some(EMBEDDING_STATUS_COMPLETED) {
            return Ok(false);
        }

        let text = Self::build_offer_text(
            &offer.title,
            offer.company.as_deref(),
            offer.location.as_deref(),
            offer.skills.as_deref(),
            offer.description.as_deref(),
        );

        match EmbeddingService::embed(&text).await {
            Ok(vector) => {
                Self::mark_completed(conn, offer_id, vector)?;
                Ok(true)
            }
            Err(e) => {
                // L'echec est enregistre plutot que propage : le poller doit continuer son lot,
                // et le compteur de tentatives est ce qui evite de boucler sur une offre morte.
                Self::mark_failure(conn, offer_id, offer.retry_count.unwrap_or(0), &e)?;
                Err(e)
            }
        }
    }

    fn mark_completed(
        conn: &mut DbConnection,
        offer_id: Uuid,
        vector: Vec<f32>,
    ) -> Result<(), AppError> {
        let now = Utc::now().naive_utc();
        diesel::update(job_offer::table.find(offer_id))
            .set((
                job_offer::embedding.eq(Some(Vector::from(vector))),
                job_offer::embedding_status.eq(EMBEDDING_STATUS_COMPLETED),
                job_offer::embedding_model.eq(EmbeddingService::model()),
                job_offer::indexing_error.eq(None::<String>),
                job_offer::indexed_at.eq(now),
                job_offer::updated_at.eq(now),
            ))
            .execute(conn)?;
        Ok(())
    }

    fn mark_failure(
        conn: &mut DbConnection,
        offer_id: Uuid,
        previous_retries: i32,
        error: &AppError,
    ) -> Result<(), AppError> {
        let retries = previous_retries.saturating_add(1);
        let status = if retries >= MAX_RETRIES {
            EMBEDDING_STATUS_FAILED
        } else {
            EMBEDDING_STATUS_PENDING
        };

        diesel::update(job_offer::table.find(offer_id))
            .set((
                job_offer::embedding_status.eq(status),
                job_offer::retry_count.eq(retries),
                job_offer::indexing_error.eq(Some(error.to_string())),
            ))
            .execute(conn)?;
        Ok(())
    }

    /// Traite un lot d'offres en attente. Renvoie le nombre de vecteurs effectivement ecrits.
    pub async fn process_pending_batch(pool: &DbPool) -> Result<usize, AppError> {
        let pending: Vec<Uuid> = {
            let mut conn = pool.get().map_err(|e| AppError::Internal(e.to_string()))?;
            job_offer::table
                .filter(job_offer::embedding_status.eq(EMBEDDING_STATUS_PENDING))
                .filter(job_offer::retry_count.lt(MAX_RETRIES).or(job_offer::retry_count.is_null()))
                // Les plus recentes d'abord : ce sont celles que l'utilisateur vient de voir
                // apparaitre dans sa recherche.
                .order(job_offer::created_at.desc().nulls_last())
                .limit(BATCH_SIZE)
                .select(job_offer::id)
                .load(&mut conn)?
        };

        if pending.is_empty() {
            return Ok(0);
        }

        let mut embedded = 0usize;
        for offer_id in pending {
            // Une connexion par offre, prise juste avant l'ecriture : le calcul du vecteur est un
            // appel reseau lent, garder une connexion ouverte pendant tout le lot epuiserait le
            // pool sous charge.
            let mut conn = pool.get().map_err(|e| AppError::Internal(e.to_string()))?;
            match Self::embed_offer(&mut conn, offer_id, false).await {
                Ok(true) => embedded += 1,
                Ok(false) => {}
                Err(e) => tracing::warn!(offer = %offer_id, error = %e, "Embedding d'offre echoue"),
            }
        }

        Ok(embedded)
    }

    /// Lance le poller d'arriere-plan. A appeler une fois au demarrage.
    ///
    /// Le premier passage est immediat : au demarrage, les offres ingerees lors de la session
    /// precedente attendent deja leur vecteur.
    pub fn spawn_poller(pool: DbPool) {
        tokio::spawn(async move {
            let mut ticker =
                tokio::time::interval(std::time::Duration::from_secs(POLL_INTERVAL_SECS));
            // `Delay` et non `Burst` : apres un lot plus long que l'intervalle, on ne veut pas que
            // tokio rattrape les tics manques en enchainant les lots sans repit.
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

            loop {
                ticker.tick().await;
                match Self::process_pending_batch(&pool).await {
                    Ok(0) => {}
                    Ok(n) => tracing::info!("Poller d'embedding : {n} offre(s) indexee(s)"),
                    Err(e) => tracing::error!(error = %e, "Poller d'embedding en echec"),
                }
            }
        });
    }
}

/// Colonnes chargees par le poller. Le vecteur lui-meme est exclu : on est en train de le
/// calculer, le relire serait 768 flottants transferes pour rien.
type PendingColumns = (
    job_offer::id,
    job_offer::title,
    job_offer::company,
    job_offer::location,
    job_offer::skills,
    job_offer::description,
    job_offer::embedding_status,
    job_offer::retry_count,
);

const PENDING_COLUMNS: PendingColumns = (
    job_offer::id,
    job_offer::title,
    job_offer::company,
    job_offer::location,
    job_offer::skills,
    job_offer::description,
    job_offer::embedding_status,
    job_offer::retry_count,
);

#[derive(Debug, Queryable)]
struct PendingOffer {
    #[allow(dead_code)]
    id: Uuid,
    title: String,
    company: Option<String>,
    location: Option<String>,
    skills: Option<String>,
    description: Option<String>,
    status: Option<String>,
    retry_count: Option<i32>,
}

/// Tronque sur une frontiere de caractere : couper en octets casserait l'UTF-8 des accents.
fn truncate(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    text.chars().take(max_chars).collect()
}

/// Garde-fou de compilation : le vecteur ecrit doit avoir la dimension de la colonne.
const _: () = assert!(EMBEDDING_DIMENSIONS == 768);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_offer_text_leads_with_the_title() {
        let text = JobOfferEmbeddingService::build_offer_text(
            "Developpeur backend",
            Some("ACME"),
            Some("Noumea"),
            Some(r#"["rust","axum"]"#),
            Some("Mission longue."),
        );
        assert!(text.starts_with("Poste: Developpeur backend"));
        // Les competences sont injectees en clair, pas en JSON brut.
        assert!(text.contains("Competences: rust, axum"));
        assert!(!text.contains(r#"["rust""#));
    }

    #[test]
    fn build_offer_text_omits_absent_fields() {
        let text = JobOfferEmbeddingService::build_offer_text("Poste", None, None, None, None);
        // Un champ absent ne laisse pas d'etiquette vide derriere lui, qui polluerait le vecteur.
        assert_eq!(text, "Poste: Poste");
    }

    #[test]
    fn build_offer_text_truncates_long_descriptions() {
        let long = "a".repeat(DESCRIPTION_MAX_CHARS * 2);
        let text = JobOfferEmbeddingService::build_offer_text("P", None, None, None, Some(&long));
        // Le plafond porte sur la description, pas sur le texte entier.
        assert!(text.len() < long.len());
        assert!(text.contains("Description: aaa"));
    }

    #[test]
    fn truncate_cuts_on_char_boundaries() {
        // Une troncature en octets sur des accents produirait de l'UTF-8 invalide.
        assert_eq!(truncate("ééééé", 3), "ééé");
        assert_eq!(truncate("court", 50), "court");
    }
}
