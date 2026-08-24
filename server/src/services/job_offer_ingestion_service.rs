//! Ingestion de masse planifiee des offres d'emploi.
//!
//! Equivalent fonctionnel de `ingestJobOffersJob` du backend Spring, sans moteur de batch
//! generique. Le choix est delibere : il n'y a qu'un seul job, ses connecteurs renvoient deja des
//! `Vec<JobOffer>` entierement materialises, et la moitie de ce qu'apporte Spring Batch — etat
//! persiste, comptes de reprise — se reduit ici a une table. Ce qui compte est reproduit :
//!
//!  - **partitionnement** par grand domaine ROME, seule facon de contourner le plafond de 3150
//!    resultats de l'API France Travail (voir [`ingestion_partitions`]) ;
//!  - **concurrence bornee**, pour tenir les quotas des API tout en interrogeant plusieurs
//!    partitions a la fois ;
//!  - **tolerance aux pannes par partition** : une partition qui echoue n'interrompt pas les
//!    autres, elle est journalisee et reprise au passage suivant ;
//!  - **journal d'execution** dans `ingestion_run`, qui sert a la fois de diagnostic et de base a
//!    la reprise.
//!
//! Comme cote Java, la **vectorisation ne fait pas partie de ce job** : l'ingestion marque les
//! offres `PENDING` et le poller d'embedding les traite hors ligne. Melanger les deux ferait
//! dependre la duree de l'ingestion de la disponibilite du modele d'embedding.

use std::collections::HashSet;
use std::sync::Arc;

use chrono::{Duration as ChronoDuration, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::config::ats_config::AtsConfig;
use crate::db::schema::{ingestion_run, job_offer};
use crate::db::{DbConnection, DbPool};
use crate::errors::AppError;
use crate::models::{
    FinishIngestionRun, JobOffer, NewIngestionRun, INGESTION_STATUS_COMPLETED,
    INGESTION_STATUS_FAILED, INGESTION_STATUS_RUNNING,
};
use crate::services::connectors::aggregators::FranceTravailConnector;
use crate::services::ingestion_partitions::{france_travail_partitions, Partition};
use crate::services::job_search_service::JobSearchService;

/// Partitions interrogees simultanement.
///
/// Borne par quota d'API et non par capacite machine : les appels sont domines par l'attente
/// reseau, donc rien n'empecherait techniquement de lancer les quatorze d'un coup — c'est France
/// Travail qui repondrait 429. Meme valeur que le `concurrencyLimit` du `TaskExecutor` Java.
const DEFAULT_CONCURRENCY: usize = 4;

/// Offres traitees par transaction d'ecriture.
///
/// C'est le `chunk-size` de Spring Batch. Un lot plus gros amortit mieux les requetes de
/// deduplication ; trop gros, il allonge la transaction et retarde l'arret du serveur.
const DEFAULT_CHUNK_SIZE: usize = 200;

/// Delai avant de considerer qu'une partition aboutie doit etre rejouee.
///
/// La reprise s'appuie dessus : une partition `COMPLETED` il y a moins de ce delai est sautee. Cela
/// rend le job **idempotent a la journee** — le relancer apres un incident ne rejoue que ce qui n'a
/// pas abouti, au lieu de reinterroger les quatorze domaines.
const COMPLETED_PARTITION_TTL_HOURS: i64 = 20;

/// Ce qu'une partition a produit. Sert a la fois de valeur de retour et de ligne de journal.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct PartitionOutcome {
    /// Offres rapportees par le connecteur.
    pub read: usize,
    /// Offres reellement inserees.
    pub written: usize,
    /// Offres ecartees : doublon deja en base, ou ligne inexploitable.
    pub skipped: usize,
}

/// Bilan d'un passage complet.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct IngestionSummary {
    pub partitions_total: usize,
    pub partitions_skipped: usize,
    pub partitions_failed: usize,
    pub read: usize,
    pub written: usize,
}

pub struct JobOfferIngestionService;

impl JobOfferIngestionService {
    /// Lance un passage complet d'ingestion et renvoie son bilan.
    ///
    /// Ne renvoie `Err` que si le passage n'a pas pu **demarrer** (base injoignable). Une partition
    /// en echec est comptee dans `partitions_failed`, pas propagee : interrompre les treize autres
    /// parce qu'une API a repondu 500 couterait une journee entiere d'ingestion.
    pub async fn run(pool: &DbPool, config: &AtsConfig) -> Result<IngestionSummary, AppError> {
        let run_id = Uuid::new_v4();
        let partitions = Self::plan(config);

        tracing::info!(
            %run_id,
            partitions = partitions.len(),
            "Ingestion planifiee : demarrage"
        );

        // Filtrage de reprise fait en une requete, avant de lancer quoi que ce soit : le faire
        // partition par partition ajouterait un aller-retour par unite de travail.
        let done = Self::completed_recently(pool)?;
        let (skipped, todo): (Vec<Partition>, Vec<Partition>) = partitions
            .into_iter()
            .partition(|p| done.contains(&(p.source.clone(), p.key.clone())));

        if !skipped.is_empty() {
            tracing::info!(
                %run_id,
                skipped = skipped.len(),
                "Partitions deja abouties recemment, non rejouees"
            );
        }

        let france_travail = Arc::new(FranceTravailConnector::new(
            reqwest::Client::new(),
            config.france_travail.clone(),
        ));
        // Le semaphore est ce qui borne la concurrence : les taches sont toutes lancees, mais
        // seules `DEFAULT_CONCURRENCY` d'entre elles detiennent un permis a un instant donne.
        let permits = Arc::new(tokio::sync::Semaphore::new(DEFAULT_CONCURRENCY));

        let mut summary = IngestionSummary {
            partitions_total: todo.len() + skipped.len(),
            partitions_skipped: skipped.len(),
            ..IngestionSummary::default()
        };

        let tasks: Vec<_> = todo
            .into_iter()
            .map(|partition| {
                let pool = pool.clone();
                let connector = Arc::clone(&france_travail);
                let permits = Arc::clone(&permits);
                async move {
                    // `acquire_owned` plutot que `acquire` : le permis doit vivre aussi longtemps
                    // que la tache, pas que l'expression qui l'obtient.
                    let _permit = permits.acquire_owned().await;
                    let outcome =
                        Self::run_partition(&pool, connector.as_ref(), run_id, &partition).await;
                    (partition, outcome)
                }
            })
            .collect();

        for (partition, outcome) in futures::future::join_all(tasks).await {
            match outcome {
                Ok(outcome) => {
                    summary.read += outcome.read;
                    summary.written += outcome.written;
                }
                Err(e) => {
                    summary.partitions_failed += 1;
                    tracing::warn!(
                        %run_id,
                        source = %partition.source,
                        partition = %partition.key,
                        error = %e,
                        "Partition d'ingestion en echec, reprise au prochain passage"
                    );
                }
            }
        }

        tracing::info!(
            %run_id,
            total = summary.partitions_total,
            skipped = summary.partitions_skipped,
            failed = summary.partitions_failed,
            read = summary.read,
            written = summary.written,
            "Ingestion planifiee : terminee"
        );
        Ok(summary)
    }

    /// Construit la liste des unites de travail d'un passage.
    ///
    /// Une source non configuree ne produit aucune partition : mieux vaut une absence dans le
    /// journal qu'une partition qui echoue quatorze fois par nuit sur des identifiants manquants.
    pub fn plan(config: &AtsConfig) -> Vec<Partition> {
        let mut partitions = Vec::new();
        if config.france_travail.is_configured() {
            partitions.extend(france_travail_partitions(
                "FRANCE_TRAVAIL",
                &config.france_travail.departements,
            ));
        } else {
            tracing::info!(
                "France Travail non configure : partitions ROME non planifiees pour ce passage"
            );
        }
        partitions
    }

    /// Traite une partition : recuperation, ecriture par lots, journalisation.
    async fn run_partition(
        pool: &DbPool,
        connector: &FranceTravailConnector,
        run_id: Uuid,
        partition: &Partition,
    ) -> Result<PartitionOutcome, AppError> {
        // La ligne est ouverte avant l'appel reseau : une partition tuee par un arret du serveur
        // reste ainsi visible en `RUNNING`, au lieu de disparaitre sans trace.
        let entry_id = Self::open(pool, run_id, partition)?;

        let fetched = match partition.grand_domaine {
            Some(domaine) => {
                connector
                    .fetch_by_grand_domaine(domaine, partition.departement.as_deref())
                    .await
            }
            // Aucune source non partitionnee n'est planifiee pour l'instant ; le cas est traite
            // plutot que `unreachable!()` pour qu'ajouter une source ne fasse pas paniquer un job
            // de nuit.
            None => {
                tracing::warn!(
                    source = %partition.source,
                    partition = %partition.key,
                    "Partition sans strategie de recuperation, ignoree"
                );
                Vec::new()
            }
        };

        let read = fetched.len();
        let mut outcome = PartitionOutcome { read, ..PartitionOutcome::default() };
        let mut last_error: Option<String> = None;

        // Ecriture par lots : c'est le `chunk` de Spring Batch. Un echec de lot n'annule que ce
        // lot, les precedents restent ecrits — une partition partiellement ingeree vaut mieux
        // qu'un rollback de trois mille offres.
        for chunk in fetched.chunks(DEFAULT_CHUNK_SIZE) {
            let chunk = chunk.to_vec();
            let pool = pool.clone();
            let written = tokio::task::spawn_blocking(move || {
                let mut conn = pool.get().map_err(|e| AppError::Internal(e.to_string()))?;
                Self::write_chunk(&mut conn, chunk)
            })
            .await
            .map_err(|e| AppError::Internal(format!("Ecriture de lot interrompue : {e}")))?;

            match written {
                Ok(written) => outcome.written += written,
                Err(e) => {
                    // Compte plutot que plafonne : voir le commentaire de `skipped_count` dans la
                    // migration. Le lot est perdu pour ce passage, la partition continue — mais
                    // elle sera marquee en echec, donc rejouee au passage suivant.
                    tracing::warn!(
                        source = %partition.source,
                        partition = %partition.key,
                        error = %e,
                        "Lot d'ingestion ecarte"
                    );
                    last_error = Some(e.to_string());
                }
            }
        }

        // Tout ce qui a ete lu sans etre ecrit : doublons deja en base, et offres des lots en
        // echec. Les distinguer demanderait de compter les doublons lot par lot, pour une
        // information que `error` porte deja — un lot en echec y laisse son message.
        outcome.skipped = read.saturating_sub(outcome.written);

        // Le statut conditionne la reprise : une partition dont un lot a echoue ne doit PAS etre
        // consideree comme aboutie, sinon `completed_recently` la sauterait au passage suivant et
        // les offres perdues ne seraient jamais rattrapees.
        let status = if last_error.is_some() {
            INGESTION_STATUS_FAILED
        } else {
            INGESTION_STATUS_COMPLETED
        };
        Self::close(pool, entry_id, status, outcome, last_error)?;
        Ok(outcome)
    }

    /// Ecrit un lot d'offres et renvoie le nombre d'insertions.
    ///
    /// Deduplication **par lot et non par offre** : une seule requete `IN` sur les couples
    /// `(source, source_id)` du lot remplace les deux cent recherches unitaires que ferait
    /// `JobSearchService::persist_all`. C'est le meme arbitrage que `JobOfferBatchWriter` cote
    /// Java, et il est ce qui rend une ingestion de dizaines de milliers d'offres tenable.
    fn write_chunk(conn: &mut DbConnection, offers: Vec<JobOffer>) -> Result<usize, AppError> {
        if offers.is_empty() {
            return Ok(0);
        }

        let source = offers
            .first()
            .and_then(|o| o.source.clone())
            .unwrap_or_default();

        let incoming_ids: Vec<String> = offers
            .iter()
            .filter_map(|o| o.source_id.clone())
            .filter(|id| !id.trim().is_empty())
            .collect();

        let known: HashSet<String> = if incoming_ids.is_empty() {
            HashSet::new()
        } else {
            job_offer::table
                .filter(job_offer::source.eq(&source))
                .filter(job_offer::source_id.eq_any(&incoming_ids))
                .select(job_offer::source_id)
                .load::<Option<String>>(conn)?
                .into_iter()
                .flatten()
                .collect()
        };

        // Les doublons *internes* au lot comptent autant que ceux deja en base : une meme offre
        // remontee deux fois par la pagination passerait la verification `known` et serait inseree
        // en double.
        let mut seen: HashSet<String> = HashSet::new();
        let fresh: Vec<JobOffer> = offers
            .into_iter()
            .filter(|offer| match offer.source_id.as_deref().map(str::trim) {
                Some(id) if !id.is_empty() => {
                    !known.contains(id) && seen.insert(id.to_string())
                }
                // Sans `source_id`, la deduplication par lot est impossible : on laisse
                // `persist_all` trancher offre par offre sur `apply_url`.
                _ => true,
            })
            .collect();

        if fresh.is_empty() {
            return Ok(0);
        }

        // `persist_all` porte la construction de `search_text`, le TTL d'expiration et le repli de
        // deduplication sur `apply_url`. Le reecrire ici dupliquerait ces trois regles, avec la
        // certitude qu'elles divergeraient : le pre-filtrage ci-dessus ne fait que lui epargner
        // l'essentiel des recherches.
        let persisted = JobSearchService::persist_offers(conn, fresh)?;
        Ok(persisted)
    }

    /// Couples `(source, partition_key)` ayant abouti assez recemment pour etre sautes.
    fn completed_recently(pool: &DbPool) -> Result<HashSet<(String, String)>, AppError> {
        let cutoff = Utc::now().naive_utc() - ChronoDuration::hours(COMPLETED_PARTITION_TTL_HOURS);
        let mut conn = pool.get().map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(ingestion_run::table
            .filter(ingestion_run::status.eq(INGESTION_STATUS_COMPLETED))
            .filter(ingestion_run::started_at.gt(cutoff))
            .select((ingestion_run::source, ingestion_run::partition_key))
            .load::<(String, String)>(&mut conn)?
            .into_iter()
            .collect())
    }

    fn open(pool: &DbPool, run_id: Uuid, partition: &Partition) -> Result<Uuid, AppError> {
        let mut conn = pool.get().map_err(|e| AppError::Internal(e.to_string()))?;
        let entry = NewIngestionRun {
            id: Uuid::new_v4(),
            run_id,
            source: partition.source.clone(),
            partition_key: partition.key.clone(),
            status: INGESTION_STATUS_RUNNING.to_string(),
            read_count: 0,
            written_count: 0,
            skipped_count: 0,
            error: None,
            started_at: Utc::now().naive_utc(),
            finished_at: None,
        };
        let id = entry.id;
        diesel::insert_into(ingestion_run::table)
            .values(&entry)
            .execute(&mut conn)?;
        Ok(id)
    }

    fn close(
        pool: &DbPool,
        entry_id: Uuid,
        status: &str,
        outcome: PartitionOutcome,
        error: Option<String>,
    ) -> Result<(), AppError> {
        let mut conn = pool.get().map_err(|e| AppError::Internal(e.to_string()))?;
        diesel::update(ingestion_run::table.find(entry_id))
            .set(&FinishIngestionRun {
                status: status.to_string(),
                read_count: clamp_count(outcome.read),
                written_count: clamp_count(outcome.written),
                skipped_count: clamp_count(outcome.skipped),
                // Tronque a la largeur de la colonne : un message plus long ferait echouer la
                // cloture, et perdre la trace de l'echec par-dessus l'echec lui-meme.
                error: error.map(|e| truncate(&e, 1024)),
                finished_at: Some(Utc::now().naive_utc()),
            })
            .execute(&mut conn)?;
        Ok(())
    }
}

/// Compte borne a `i32`, largeur de la colonne.
///
/// Aucune partition n'approche les deux milliards — le plafond de l'API est a 3150 — mais un
/// `as i32` silencieux sur un compte aberrant ecrirait un nombre negatif dans le journal.
fn clamp_count(value: usize) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

/// Tronque sur une frontiere de caractere.
///
/// `&s[..max]` paniquerait au milieu d'un caractere multi-octet, ce qu'un message d'erreur
/// contenant un accent suffit a produire.
fn truncate(value: &str, max: usize) -> String {
    if value.len() <= max {
        return value.to_string();
    }
    value.chars().take(max).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn les_comptes_sont_bornes_a_la_largeur_de_la_colonne() {
        assert_eq!(clamp_count(0), 0);
        assert_eq!(clamp_count(3150), 3150);
        assert_eq!(clamp_count(usize::MAX), i32::MAX);
    }

    #[test]
    fn la_troncature_ne_coupe_pas_un_caractere_accentue() {
        // Le cas qui paniquerait avec `&s[..max]` : la limite tombe au milieu du « e accent ».
        let message = "e\u{0301}chec repete".repeat(200);
        let truncated = truncate(&message, 10);

        assert_eq!(truncated.chars().count(), 10);
        assert!(message.starts_with(&truncated));
    }

    #[test]
    fn un_message_court_nest_pas_modifie() {
        assert_eq!(truncate("timeout", 1024), "timeout");
    }

    #[test]
    fn le_bilan_par_defaut_est_neutre() {
        let summary = IngestionSummary::default();
        assert_eq!(summary.partitions_total, 0);
        assert_eq!(summary.written, 0);
    }
}
