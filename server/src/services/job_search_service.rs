//! Recherche d'offres : collecte live chez les sources, persistance dedupliquee, et classement
//! du corpus deja ingere.
//!
//! La recherche semantique passe par [`JobOfferVectorIndex`] (pgvector) ; le classement lexical
//! de ce module reste le repli quand une offre n'est pas encore vectorisee.
//!
//! Le pipeline complet est celui de l'application Spring : recuperation vectorielle et lexicale,
//! fusion par Reciprocal Rank Fusion ([`RrfFusionService`]), puis reclassement cross-encoder ONNX
//! ([`RerankerService`]).
//!
//! Ecart restant : le volet lexical est un score de correspondance maison (titre pondere,
//! competences, description) et non un vrai BM25 — `pg_search` est installe dans l'image
//! ParadeDB mais pas encore exploite.

use chrono::Utc;
use diesel::prelude::*;
use diesel::PgTextExpressionMethods;
use reqwest::Client;

use uuid::Uuid;

use crate::db::schema::job_offer;
use crate::db::DbConnection;
use crate::errors::AppError;
use crate::models::{JobOffer, NewJobOffer};
use crate::config::ats_config::AtsConfig;
use crate::services::connectors::aggregators::{
    AdzunaConnector, CareerjetConnector, FranceTravailConnector,
};
use crate::services::connectors::ats_connector::AtsConnector;
use crate::services::connectors::boards::{
    AshbyConnector, GreenhouseConnector, LeverConnector, RecruiteeConnector,
    SmartRecruitersConnector, WorkableConnector,
};
use crate::services::connectors::emploi_nc::EmploiNcConnector;
use crate::services::connectors::feeds::{JobicyConnector, RemoteOkConnector};
use crate::services::connectors::scrapers::{HelloWorkConnector, SeekConnector};
use crate::services::geo_service::GeoService;
use crate::services::job_offer_vector_index::JobOfferVectorIndex;
use crate::services::reranker_service::RerankerService;
use crate::services::rrf_service::RrfFusionService;

/// Statut d'embedding porte par les offres indexees, aligne sur l'enum Java `EmbeddingStatus`.
pub const EMBEDDING_STATUS_PENDING: &str = "PENDING";
pub const EMBEDDING_STATUS_COMPLETED: &str = "COMPLETED";
/// Offre dont la vectorisation a echoue trop de fois : elle n'est plus retentee par le poller.
pub const EMBEDDING_STATUS_FAILED: &str = "FAILED";

/// Plafond du nombre d'offres classees par la recherche lexicale. Sans borne, une requete d'un
/// seul mot courant ramenerait le corpus entier pour n'en afficher que trente lignes.
const CORPUS_SCAN_LIMIT: i64 = 500;

/// Nombre de candidats recuperes **par liste** avant fusion, independamment du nombre demande.
///
/// C'est le levier principal sur la qualite de la sortie RRF. Passer directement la limite de
/// l'appelant aux deux recuperations donnait un cas absurde : pour `limit = 5`, la fusion recevait
/// 5 vecteurs et 5 resultats BM25, soit 10 candidats au plus, dont il fallait deja rendre 5. Ni la
/// fusion ni le cross-encoder n'avaient de marge de manoeuvre.
///
/// Le rapport usuel en recherche hybride est de recuperer un ordre de grandeur de plus que ce qu'on
/// rend, puis de laisser le reclassement trancher.
const RETRIEVAL_DEPTH: i64 = 60;

/// Nombre maximum de candidats soumis au cross-encoder.
///
/// L'inference coute environ une dizaine de millisecondes par paire : reclasser les 120 candidats
/// que peut produire la fusion ajouterait plus d'une seconde a la recherche pour departager des
/// offres qui ne seront jamais affichees.
const RERANK_DEPTH: usize = 40;

/// Nombre maximum d'offres partageant le meme intitule et la meme entreprise dans les resultats.
///
/// Mesure sur le corpus : « Applied AI Engineer » chez openai existe en cinq exemplaires — Abu
/// Dhabi, Delhi, Londres, Seoul, Sydney. Ce sont cinq postes reels, donc les dedupliquer serait une
/// perte d'information ; mais les laisser occuper cinq des dix places affichees prive l'utilisateur
/// de toute diversite. On en garde deux et on laisse la place aux autres roles.
const MAX_PER_ROLE: usize = 2;

/// Duree de vie par defaut, en jours, estampillee sur une offre ingeree qui n'annonce pas sa propre
/// date d'expiration.
///
/// Sans cela la colonne `expires_at` reste NULL sur toutes les offres, le filtre d'expiration ne se
/// declenche jamais et le corpus accumule indefiniment des annonces mortes. Meme variable
/// d'environnement que l'application Spring (`OFFER_TTL_DAYS`), pour que les deux backends purgent
/// au meme rythme.
const DEFAULT_OFFER_TTL_DAYS: i64 = 30;

fn offer_ttl_days() -> i64 {
    std::env::var("OFFER_TTL_DAYS")
        .ok()
        .and_then(|value| value.trim().parse::<i64>().ok())
        .filter(|days| *days > 0)
        .unwrap_or(DEFAULT_OFFER_TTL_DAYS)
}

// Garde-fous de compilation sur le levier principal du RRF : recuperer autant que ce qu'on rend
// priverait la fusion et le reclassement de toute marge de manoeuvre. Verifie ici plutot que dans
// un test, ou l'assertion porterait sur des constantes et ne s'evaluerait jamais a l'execution.
const _: () = assert!(RETRIEVAL_DEPTH >= 30, "profondeur de recuperation trop faible");
const _: () = assert!(RERANK_DEPTH <= (RETRIEVAL_DEPTH as usize) * 2);
const _: () = assert!(MAX_PER_ROLE >= 1, "un plafond nul viderait les resultats");

// `unaccent()` de Postgres (extension activee par migration).
//
// Sans elle, « developpeur » ne trouve aucune offre intitulee « Developpeur » avec accents, ce
// qui rend la quasi-totalite du corpus emploi.nc introuvable des que l'utilisateur tape sans
// accents. La fonction est STABLE et non IMMUTABLE : elle ne peut pas servir d'index, d'ou le
// plafond de balayage `CORPUS_SCAN_LIMIT`.
//
// Commentaire non-doc : rustdoc ne documente pas les invocations de macro.
diesel::sql_function! {
    fn unaccent(text: diesel::sql_types::Nullable<diesel::sql_types::Text>)
        -> diesel::sql_types::Nullable<diesel::sql_types::Text>;
}

/// Identifiant renvoye par la requete BM25, avant chargement des offres.
#[derive(diesel::QueryableByName)]
struct Bm25Hit {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
}

/// Predicat `WHERE` compose dynamiquement sur `job_offer`, un terme de recherche a la fois.
type BoxedPredicate = Box<
    dyn BoxableExpression<
        job_offer::table,
        diesel::pg::Pg,
        SqlType = diesel::sql_types::Nullable<diesel::sql_types::Bool>,
    >,
>;

pub struct JobSearchService;

impl JobSearchService {
    /// Connecteurs actifs, portage complet de la liste `permits` de `AbstractAtsConnector`.
    ///
    /// Le client HTTP est partage : chaque `reqwest::Client` porte son propre pool de connexions,
    /// en construire un par connecteur multiplierait les handshakes TLS vers les memes hotes.
    ///
    /// Les connecteurs a identifiants restent dans la liste meme sans configuration : ils
    /// renvoient une liste vide en le journalisant une fois, ce qui se diagnostique mieux qu'une
    /// source absente de la liste. Seul Seek est conditionnel, comme cote Spring : sa mitigation
    /// anti-bot repond 403 au scraping serveur, donc l'activer par defaut ne produirait que du
    /// bruit dans les journaux.
    fn connectors() -> Vec<Box<dyn AtsConnector>> {
        let client = Client::new();
        let config = AtsConfig::from_env();

        let mut connectors: Vec<Box<dyn AtsConnector>> = vec![
            // Agregateurs : les seuls a offrir une recherche transverse.
            Box::new(EmploiNcConnector::new(client.clone())),
            Box::new(AdzunaConnector::new(client.clone(), config.adzuna.clone())),
            Box::new(CareerjetConnector::new(client.clone(), config.careerjet.clone())),
            Box::new(FranceTravailConnector::new(
                client.clone(),
                config.france_travail.clone(),
            )),
            // Flux publics sans cle, filtres en memoire.
            Box::new(RemoteOkConnector::new(client.clone())),
            Box::new(JobicyConnector::new(client.clone(), config.jobicy.clone())),
            // Boards ATS : pas de recherche transverse, mais indispensables a `fetch_from_url`
            // et au sondage d'un slug d'entreprise.
            Box::new(GreenhouseConnector::new(client.clone())),
            Box::new(LeverConnector::new(client.clone())),
            Box::new(SmartRecruitersConnector::new(client.clone())),
            Box::new(AshbyConnector::new(client.clone(), config.ashby_boards.clone())),
            Box::new(WorkableConnector::new(
                client.clone(),
                config.workable_accounts.clone(),
            )),
            Box::new(RecruiteeConnector::new(
                client.clone(),
                config.recruitee_companies.clone(),
            )),
            // Sans API publique : analyse de la page de recherche.
            Box::new(HelloWorkConnector::new(client.clone())),
        ];

        if config.seek_enabled {
            connectors.push(Box::new(SeekConnector::new(client)));
        }

        connectors
    }

    /// Recherche live chez les sources externes, puis persiste ce qui est nouveau et renvoie les
    /// offres telles qu'elles sont en base (donc avec leur identifiant, indispensable au front
    /// pour candidater ou demander un positionnement).
    ///
    /// L'echec d'un connecteur est journalise et ignore : une source indisponible ne doit pas
    /// vider la recherche entiere.
    pub async fn search_all(
        conn: &mut DbConnection,
        keywords: Option<&str>,
        location: Option<&str>,
        source: Option<&str>,
        country: Option<&str>,
    ) -> Result<Vec<JobOffer>, AppError> {
        let wanted_source = source.map(str::trim).filter(|s| !s.is_empty());

        let selected: Vec<Box<dyn AtsConnector>> = Self::connectors()
            .into_iter()
            .filter(|connector| match wanted_source {
                Some(wanted) => connector.source().as_str().eq_ignore_ascii_case(wanted),
                None => true,
            })
            .collect();

        // Les sources sont interrogees en parallele : en serie, sept appels reseau a quinze
        // secondes de delai maximum plafonneraient la recherche a pres de deux minutes. En
        // parallele, le temps total est celui de la source la plus lente.
        let results = futures::future::join_all(
            selected
                .iter()
                .map(|connector| async move {
                    (
                        connector.source().as_str(),
                        connector
                            .fetch_jobs_by_country(keywords, location, country)
                            .await,
                    )
                }),
        )
        .await;

        let mut fetched: Vec<JobOffer> = Vec::new();
        for (source, result) in results {
            match result {
                Ok(offers) => fetched.extend(offers),
                // Une source indisponible degrade en « aucun resultat » : elle ne doit pas vider
                // la recherche entiere.
                Err(e) => tracing::warn!(
                    source,
                    error = %e,
                    "Source indisponible, ignoree pour cette recherche"
                ),
            }
        }

        // Les connecteurs filtrent le lieu de facon inegale : certains l'ignorent, d'autres le
        // rapprochent grossierement. On reordonne donc les resultats agreges par proximite au lieu
        // demande, et on ecarte les erreurs de pays certaines.
        let persisted = Self::persist_all(conn, fetched)?;
        Ok(Self::rerank_by_location(persisted, location, country))
    }

    /// Reordonne par proximite geographique et ecarte les offres certainement dans un autre pays.
    ///
    /// Deux mecanismes distincts, volontairement asymetriques :
    ///
    ///  - le **tri** repose sur une comparaison textuelle des libelles, donc approximative — mais
    ///    se tromper d'ordre est sans gravite ;
    ///  - l'**exclusion** exige une preuve structuree : un pays demande par l'appelant ET un code
    ///    pays renseigne sur l'offre. Ecarter sur une simple divergence de libelles supprimerait
    ///    des offres valides, « Paris » et « Ile-de-France » n'ayant aucun mot commun.
    ///
    /// Sans lieu ni pays demandes, la fonction ne fait rien.
    fn rerank_by_location(
        offers: Vec<JobOffer>,
        search_location: Option<&str>,
        search_country: Option<&str>,
    ) -> Vec<JobOffer> {
        let location = search_location.map(str::trim).filter(|v| !v.is_empty());
        let country = search_country.map(str::trim).filter(|v| !v.is_empty());

        if location.is_none() && country.is_none() {
            return offers;
        }

        let before = offers.len();
        let mut kept: Vec<JobOffer> = offers
            .into_iter()
            .filter(|offer| {
                !GeoService::is_confident_country_mismatch(country, offer.country.as_deref())
            })
            .collect();

        let dropped = before - kept.len();
        if dropped > 0 {
            tracing::info!(
                requested_country = country,
                dropped,
                kept = kept.len(),
                "Reclassement geographique : offres d'un autre pays ecartees"
            );
        }

        if let Some(location) = location {
            // Tri stable : a proximite egale, l'ordre d'arrivee des connecteurs est conserve.
            kept.sort_by(|a, b| {
                let score_of = |offer: &JobOffer| {
                    GeoService::match_level(
                        location,
                        offer.location.as_deref(),
                        offer.remote.unwrap_or(false),
                    )
                    .score()
                };
                score_of(b).total_cmp(&score_of(a))
            });
        }

        kept
    }

    /// Recupere les offres publiees sur une URL de board supportee.
    pub async fn fetch_from_url(
        conn: &mut DbConnection,
        url: &str,
    ) -> Result<Vec<JobOffer>, AppError> {
        let url = url.trim();
        if url.is_empty() {
            return Err(AppError::BadRequest("URL manquante".into()));
        }

        for connector in Self::connectors() {
            if connector.supports(url) {
                let offers = connector.fetch_from_url(url).await?;
                return Self::persist_all(conn, offers);
            }
        }

        Err(AppError::BadRequest(format!(
            "Aucun connecteur ne gere cette URL : {url}"
        )))
    }

    /// Insere les offres inconnues et renvoie la version persistee de chacune.
    ///
    /// La deduplication se fait sur `(source, source_id)` quand la source fournit un identifiant
    /// stable, sinon sur `apply_url`. Sans cela, chaque recherche reinsererait tout le catalogue.
    fn persist_all(
        conn: &mut DbConnection,
        offers: Vec<JobOffer>,
    ) -> Result<Vec<JobOffer>, AppError> {
        let now = Utc::now().naive_utc();
        let mut persisted = Vec::with_capacity(offers.len());

        for offer in offers {
            if let Some(existing) = Self::find_duplicate(conn, &offer)? {
                persisted.push(existing);
                continue;
            }

            let new_offer = NewJobOffer {
                title: offer.title.clone(),
                company: offer.company.clone(),
                location: offer.location.clone(),
                country: offer.country.clone(),
                remote: offer.remote,
                description: offer.description.clone(),
                // `search_text` alimente le classement lexical : on le construit a l'insertion
                // pour ne pas avoir a concatener trois colonnes a chaque requete.
                search_text: Some(Self::build_search_text(&offer)),
                skills: offer.skills.clone(),
                metadata: offer.metadata.clone(),
                raw_payload: offer.raw_payload.clone(),
                content_hash: offer.content_hash.clone(),
                embedding_status: Some(EMBEDDING_STATUS_PENDING.to_string()),
                embedding_model: None,
                reindex_version: offer.reindex_version,
                retry_count: Some(0),
                indexing_error: None,
                source: offer.source.clone(),
                source_id: offer.source_id.clone(),
                apply_url: offer.apply_url.clone(),
                salary_min: offer.salary_min,
                salary_max: offer.salary_max,
                salary_currency: offer.salary_currency.clone(),
                contract_type: offer.contract_type.clone(),
                experience_level: offer.experience_level.clone(),
                category: offer.category.clone(),
                source_category: offer.source_category.clone(),
                published_at: offer.published_at,
                created_at: Some(now),
                indexed_at: None,
                updated_at: Some(now),
                // Expiration annoncee par la source si elle en donne une, sinon TTL par defaut :
                // c'est ce qui rend le filtre d'expiration operant, et c'est ce que fait
                // l'ingestion Spring.
                expires_at: offer
                    .expires_at
                    .or_else(|| now.checked_add_signed(chrono::Duration::days(offer_ttl_days()))),
                last_checked_at: Some(now),
                created_by: Some("system".to_string()),
                created_date: Some(now),
                last_modified_by: Some("system".to_string()),
                last_modified_date: Some(now),
            };

            let inserted: JobOffer = diesel::insert_into(job_offer::table)
                .values(&new_offer)
                .returning(JobOffer::as_returning())
                .get_result(conn)?;
            persisted.push(inserted);
        }

        Ok(persisted)
    }

    fn find_duplicate(
        conn: &mut DbConnection,
        offer: &JobOffer,
    ) -> Result<Option<JobOffer>, AppError> {
        if let (Some(source), Some(source_id)) = (
            offer.source.as_deref().filter(|s| !s.trim().is_empty()),
            offer.source_id.as_deref().filter(|s| !s.trim().is_empty()),
        ) {
            let found = job_offer::table
                .filter(job_offer::source.eq(source))
                .filter(job_offer::source_id.eq(source_id))
                .select(JobOffer::as_select())
                .first(conn)
                .optional()?;
            if found.is_some() {
                return Ok(found);
            }
        }

        let Some(apply_url) = offer.apply_url.as_deref().filter(|s| !s.trim().is_empty()) else {
            return Ok(None);
        };

        Ok(job_offer::table
            .filter(job_offer::apply_url.eq(apply_url))
            .select(JobOffer::as_select())
            .first(conn)
            .optional()?)
    }

    fn build_search_text(offer: &JobOffer) -> String {
        [
            Some(offer.title.as_str()),
            offer.company.as_deref(),
            offer.location.as_deref(),
            offer.skills.as_deref(),
            offer.description.as_deref(),
        ]
        .iter()
        .flatten()
        .copied()
        .collect::<Vec<&str>>()
        .join(" ")
    }

    /// Classement lexical du corpus deja ingere, en remplacement provisoire de la recherche
    /// hybride vectorielle.
    ///
    /// Chaque terme de la requete est cherche dans le titre, les competences et la description.
    /// Le score est le nombre de termes trouves, pondere : un terme dans le titre compte double,
    /// parce qu'un intitule qui contient le mot cherche est presque toujours plus pertinent
    /// qu'une mention noyee dans le corps de l'annonce.
    /// Recherche BM25 via l'index `pg_search`.
    ///
    /// Remplace le classement lexical maison : frontieres de mots, ponderation IDF des termes,
    /// saturation de frequence et normalisation par longueur de document — voir la migration
    /// `add_job_offer_bm25_index` pour les mesures qui ont motive la configuration du tokenizer.
    ///
    /// Renvoie `Err` quand l'index est absent (migration non appliquee) pour que l'appelant puisse
    /// se rabattre sur [`Self::search_lexical`].
    pub fn search_bm25(
        conn: &mut DbConnection,
        query: &str,
        source: Option<&str>,
        limit: i64,
    ) -> Result<Vec<JobOffer>, AppError> {
        let terms = Self::tokenize(query);
        if terms.is_empty() {
            return Ok(Vec::new());
        }

        // Requete en OR sur les termes : BM25 se charge de classer, un ET exclurait toute offre a
        // laquelle il manque un seul mot de la requete.
        //
        // Les termes viennent de `tokenize`, qui ne garde qu'alphanumerique plus `+` et `#`. Ils ne
        // peuvent donc pas contenir les metacaracteres de la syntaxe de requete (`:`, `"`, `(`,
        // `^`, `~`), ce qui rend l'interpolation sure ici — mais on les passe malgre tout en
        // parametre lie, pour que cette surete ne depende pas d'une invariante distante.
        let expression = terms.join(" OR ");

        // Les offres expirees sont exclues des resultats mais conservees en base : celles liees a
        // une candidature font partie de l'historique de l'utilisateur.
        let mut sql = String::from(
            "SELECT id FROM job_offer \
             WHERE search_text @@@ $1 AND (expires_at IS NULL OR expires_at > now())",
        );
        if source.map(str::trim).is_some_and(|s| !s.is_empty()) {
            sql.push_str(" AND upper(source) = upper($3)");
        }
        sql.push_str(" ORDER BY paradedb.score(id) DESC LIMIT $2");

        let ids: Vec<Uuid> = {
            let base = diesel::sql_query(&sql)
                .bind::<diesel::sql_types::Text, _>(expression)
                .bind::<diesel::sql_types::BigInt, _>(limit.clamp(1, CORPUS_SCAN_LIMIT));

            match source.map(str::trim).filter(|s| !s.is_empty()) {
                Some(source) => base
                    .bind::<diesel::sql_types::Text, _>(source.to_string())
                    .load::<Bm25Hit>(conn),
                None => base.load::<Bm25Hit>(conn),
            }
            .map_err(|e| AppError::Internal(e.to_string()))?
            .into_iter()
            .map(|hit| hit.id)
            .collect()
        };

        if ids.is_empty() {
            return Ok(Vec::new());
        }

        // Deuxieme requete pour charger les offres : `sql_query` ne sait pas materialiser un
        // `JobOffer` (la table porte une colonne `vector` que le modele n'expose pas), et le
        // faire en une passe demanderait de lister les trente-cinq colonnes a la main.
        let mut offers: Vec<JobOffer> = job_offer::table
            .filter(job_offer::id.eq_any(&ids))
            .select(JobOffer::as_select())
            .load(conn)?;

        // `eq_any` perd l'ordre du classement : on le restaure depuis la position dans `ids`.
        let rank_of: std::collections::HashMap<Uuid, usize> =
            ids.iter().enumerate().map(|(i, id)| (*id, i)).collect();
        offers.sort_by_key(|offer| rank_of.get(&offer.id).copied().unwrap_or(usize::MAX));

        Ok(offers)
    }

    pub fn search_lexical(
        conn: &mut DbConnection,
        query: &str,
        source: Option<&str>,
        limit: i64,
    ) -> Result<Vec<JobOffer>, AppError> {
        let terms = Self::tokenize(query);
        let limit = limit.clamp(1, CORPUS_SCAN_LIMIT);

        let mut sql = job_offer::table
            .into_boxed()
            // Meme exclusion que le volet BM25 : sans elle, les deux listes fusionnees par RRF
            // n'auraient pas le meme perimetre.
            .filter(
                job_offer::expires_at
                    .is_null()
                    .or(job_offer::expires_at.gt(diesel::dsl::now)),
            );
        if let Some(source) = source.map(str::trim).filter(|s| !s.is_empty()) {
            sql = sql.filter(job_offer::source.eq(source.to_uppercase()));
        }

        // Sans terme exploitable, la « recherche » degenere en « offres les plus recentes »,
        // ce qui reste une reponse utile pour une page de resultats vide.
        if terms.is_empty() {
            return Ok(sql
                .order(job_offer::published_at.desc().nulls_last())
                .limit(limit)
                .select(JobOffer::as_select())
                .load(conn)?);
        }

        // Un OR sur les termes en base, puis le scoring en memoire : le nombre de candidats est
        // borne par CORPUS_SCAN_LIMIT, donc le tri cote serveur applicatif reste negligeable, et
        // cela evite de construire une expression SQL de score par terme.
        let mut any_term = sql;
        {
            let mut predicate: Option<BoxedPredicate> = None;
            for term in &terms {
                let pattern = format!("%{term}%");
                // `Nullable<Bool>` et non `Bool` : `skills` et `search_text` sont nullables, donc
                // le `ILIKE` qui les vise peut valoir NULL, ce que Diesel reflete dans le type.
                // Seule la colonne passe par `unaccent` : le motif vient de `tokenize`, qui a
                // deja replie les accents cote Rust.
                let clause: BoxedPredicate = Box::new(
                    unaccent(job_offer::title.nullable())
                        .ilike(pattern.clone())
                        .or(unaccent(job_offer::skills).ilike(pattern.clone()))
                        .or(unaccent(job_offer::search_text).ilike(pattern)),
                );
                predicate = Some(match predicate {
                    None => clause,
                    Some(previous) => Box::new(previous.or(clause)),
                });
            }
            if let Some(predicate) = predicate {
                any_term = any_term.filter(predicate);
            }
        }

        let candidates: Vec<JobOffer> = any_term
            .order(job_offer::published_at.desc().nulls_last())
            .limit(CORPUS_SCAN_LIMIT)
            .select(JobOffer::as_select())
            .load(conn)?;

        let mut scored: Vec<(i32, JobOffer)> = candidates
            .into_iter()
            .map(|offer| (Self::lexical_score(&offer, &terms), offer))
            .collect();

        // Tri stable : a score egal on conserve l'ordre de publication decroissant deja etabli
        // par la requete, plutot que de reordonner arbitrairement.
        scored.sort_by_key(|(score, _)| std::cmp::Reverse(*score));

        Ok(scored
            .into_iter()
            .take(limit as usize)
            .map(|(_, offer)| offer)
            .collect())
    }

    fn lexical_score(offer: &JobOffer, terms: &[String]) -> i32 {
        // Les termes arrivent deja replies par `tokenize` : on replie donc aussi les valeurs de
        // l'offre, sinon le score en memoire contredirait le filtre `unaccent` fait en base.
        let title = fold_accents(&offer.title);
        let skills = fold_accents(offer.skills.as_deref().unwrap_or_default());
        let body = fold_accents(
            offer
                .search_text
                .as_deref()
                .or(offer.description.as_deref())
                .unwrap_or_default(),
        );

        terms
            .iter()
            .map(|term| {
                if title.contains(term) {
                    2
                } else if skills.contains(term) || body.contains(term) {
                    1
                } else {
                    0
                }
            })
            .sum()
    }

    /// Decoupe la requete en termes normalises, en ecartant le bruit.
    ///
    /// Les termes d'un seul caractere sont jetes : ils correspondent a presque tout et diluent le
    /// score au point de le rendre inutile pour ordonner les resultats.
    fn tokenize(query: &str) -> Vec<String> {
        query
            .split(|c: char| !c.is_alphanumeric() && c != '+' && c != '#')
            .map(fold_accents)
            .filter(|t| t.chars().count() > 1)
            .collect()
    }

    /// Recherche semantique avec repli lexical.
    ///
    /// C'est le chemin utilise par `search-smart` et le pre-filtrage du matching. Le repli n'est
    /// pas une precaution decorative : tant que le poller n'a pas vectorise le corpus — et pour
    /// toute offre dont l'embedding a echoue — la recherche vectorielle ne renvoie rien. Repondre
    /// « aucun resultat » alors que des offres pertinentes existent serait pire qu'un classement
    /// lexical approximatif.
    pub async fn search_semantic(
        index: &JobOfferVectorIndex,
        conn: &mut DbConnection,
        query: &str,
        source: Option<&str>,
        limit: i64,
    ) -> Result<Vec<JobOffer>, AppError> {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return Self::search_lexical(conn, query, source, limit);
        }

        // Le seuil n'est PAS applique en SQL : il faut pouvoir distinguer « le corpus n'a aucun
        // vecteur » (repli lexical legitime) de « aucune offre n'est assez proche » (resultat vide
        // correct). Filtrer en base confondrait les deux en une seule liste vide.
        // Profondeur de recuperation independante du nombre demande : c'est ce qui donne a la
        // fusion et au reclassement de quoi travailler.
        let depth = RETRIEVAL_DEPTH.max(limit);

        let hits = match index.search(trimmed, depth, None).await {
            Ok(hits) => hits,
            Err(e) => {
                // Modele d'embedding injoignable : on degrade au lexical plutot que de renvoyer
                // une erreur a l'utilisateur, dont la recherche fonctionnait la minute d'avant.
                tracing::warn!(error = %e, "Recherche vectorielle indisponible, repli lexical");
                return Self::search_lexical(conn, query, source, limit);
            }
        };

        if hits.is_empty() {
            // Rien de vectorise (poller pas encore passe, ou embeddings en echec) : le lexical
            // reste la seule facon de repondre quelque chose d'utile.
            tracing::debug!("Aucune offre vectorisee, repli lexical");
            return Self::search_lexical(conn, query, source, limit);
        }

        let floor = similarity_floor();
        let offers: Vec<JobOffer> = hits
            .into_iter()
            .filter(|(similarity, _)| *similarity >= floor)
            .map(|(_, offer)| offer)
            // Le filtrage par source se fait apres coup : l'index ne gere pas les filtres par
            // metadonnees, et le faire en SQL avant le tri vectoriel exigerait de dupliquer la
            // requete. Sur ces volumes, filtrer un top-N est negligeable.
            .filter(|offer| matches_source(offer, source))
            .collect();

        // Pipeline hybride complet, comme la version Spring : vectoriel + lexical fusionnes par
        // RRF, puis reclasses par le cross-encoder.
        //
        // Le seuil de similarite est applique AVANT la fusion, sur la seule liste vectorielle :
        // c'est lui qui ecarte le hors-sujet franc (« boulanger patissier » sur un corpus
        // informatique). La liste lexicale n'y est pas soumise — une correspondance litterale de
        // mot est une preuve directe, pas une ressemblance a seuiller.
        // Volet BM25, avec repli sur le classement lexical maison si l'index n'existe pas encore
        // (migration non appliquee). Un echec ici ne doit pas faire tomber une recherche
        // vectorielle qui a deja abouti : le lexical n'apporte que du rappel supplementaire.
        let lexical = match Self::search_bm25(conn, query, source, depth) {
            Ok(offers) => offers,
            Err(e) => {
                tracing::warn!(error = %e, "BM25 indisponible, repli sur le classement lexical");
                Self::search_lexical(conn, query, source, depth).unwrap_or_else(|e| {
                    tracing::warn!(error = %e, "Volet lexical indisponible, fusion sur le vectoriel seul");
                    Vec::new()
                })
            }
        };

        let fused = RrfFusionService::fuse(vec![offers, lexical], |offer: &JobOffer| offer.id);

        // Reclassement cross-encoder sur une profondeur bornee : le cosinus dit ce qui *ressemble*
        // a la requete, le cross-encoder ce qui y *repond*. Sans effet si le modele est absent.
        let candidates = fused.into_iter().take(RERANK_DEPTH).collect();
        let reranked = Self::rerank(trimmed, candidates, RERANK_DEPTH as i64).await;

        // Diversification en dernier : elle doit s'appliquer sur le classement final, sinon elle
        // ecarterait des offres que le reclassement aurait fait remonter.
        Ok(Self::diversify(reranked, limit.max(1) as usize))
    }

    /// Limite le nombre d'offres partageant le meme intitule et la meme entreprise, puis tronque.
    ///
    /// Les offres ecartees ne sont pas des doublons : ce sont le meme role dans des villes
    /// differentes, avec des URL de candidature distinctes. Les supprimer de la base serait une
    /// perte ; les laisser monopoliser la page de resultats en est une autre. On plafonne donc
    /// l'affichage sans toucher au corpus.
    fn diversify(offers: Vec<JobOffer>, limit: usize) -> Vec<JobOffer> {
        let mut seen: std::collections::HashMap<(String, String), usize> =
            std::collections::HashMap::new();
        let mut kept = Vec::with_capacity(limit.min(offers.len()));
        let mut overflow: Vec<JobOffer> = Vec::new();

        for offer in offers {
            let key = (
                offer.title.trim().to_lowercase(),
                offer.company.as_deref().unwrap_or_default().trim().to_lowercase(),
            );
            let count = seen.entry(key).or_insert(0);
            if *count < MAX_PER_ROLE {
                *count += 1;
                kept.push(offer);
            } else {
                // Conservees a part : si la diversification laissait la page a moitie vide, mieux
                // vaut la completer avec des offres ecartees que rendre moins que demande.
                overflow.push(offer);
            }
        }

        if kept.len() < limit {
            kept.extend(overflow.into_iter().take(limit - kept.len()));
        }
        kept.truncate(limit);
        kept
    }

    /// Reclasse les offres avec le cross-encoder ONNX.
    ///
    /// Le document soumis au modele est volontairement court — intitule, entreprise, lieu, debut
    /// de description : le cross-encoder tronque a 512 jetons, et lui donner une annonce entiere
    /// ferait juger la moitie de son pied de page plutot que le poste.
    async fn rerank(query: &str, offers: Vec<JobOffer>, limit: i64) -> Vec<JobOffer> {
        if offers.len() < 2 {
            return offers;
        }

        let documents: Vec<String> = offers.iter().map(Self::rerank_document).collect();
        let indices = RerankerService::rank_indices(query, documents).await;
        RerankerService::apply(offers, &indices, limit.max(1) as usize)
    }

    /// Representation textuelle d'une offre soumise au cross-encoder.
    fn rerank_document(offer: &JobOffer) -> String {
        // Part de description retenue, alignee sur la troncature du cross-encoder (128 jetons,
        // soit environ 400 caracteres au total). En envoyer davantage ne ferait que faire tronquer
        // le tokenizer, apres avoir paye la copie de la chaine.
        const DESCRIPTION_CHARS: usize = 280;

        let mut parts = vec![offer.title.clone()];
        if let Some(company) = offer.company.as_deref().filter(|v| !v.trim().is_empty()) {
            parts.push(company.to_string());
        }
        if let Some(location) = offer.location.as_deref().filter(|v| !v.trim().is_empty()) {
            parts.push(location.to_string());
        }
        if let Some(description) = offer.description.as_deref().filter(|v| !v.trim().is_empty()) {
            parts.push(description.chars().take(DESCRIPTION_CHARS).collect());
        }
        parts.join(" - ")
    }

    /// Nombre d'offres reellement indexees (embedding termine), donc interrogeables par l'IA.
    pub fn indexed_count(conn: &mut DbConnection) -> Result<i64, AppError> {
        Ok(job_offer::table
            .filter(job_offer::embedding_status.eq(EMBEDDING_STATUS_COMPLETED))
            .count()
            .get_result(conn)?)
    }

    /// Resout une offre depuis son URL de candidature. C'est ce que fait le front quand il tient
    /// un apercu d'offre sans identifiant et a besoin de l'entite persistee.
    pub fn find_by_apply_url(
        conn: &mut DbConnection,
        apply_url: &str,
        limit: i64,
    ) -> Result<(Vec<JobOffer>, i64), AppError> {
        let total: i64 = job_offer::table
            .filter(job_offer::apply_url.eq(apply_url))
            .count()
            .get_result(conn)?;

        let items = job_offer::table
            .filter(job_offer::apply_url.eq(apply_url))
            .order(job_offer::created_at.desc().nulls_last())
            .limit(limit.clamp(1, 100))
            .select(JobOffer::as_select())
            .load(conn)?;

        Ok((items, total))
    }
}

/// Similarite cosinus minimale pour qu'une offre soit consideree pertinente.
///
/// Mesure sur le corpus emploi.nc avec `nomic-embed-text` (768 dimensions) :
///
/// | requete                                         | meilleure similarite |
/// |-------------------------------------------------|----------------------|
/// | « developpeur full stack » (correspondance)     | 0.761                |
/// | « conteneurisation et automatisation »          | 0.595                |
/// | « ingenieur infrastructure cloud » (semantique) | 0.593                |
/// | « recette de cuisine »                          | 0.562                |
/// | « plombier chauffagiste »                       | 0.528                |
/// | « boulanger patissier »                         | 0.524                |
///
/// La fenetre entre le pertinent le plus faible (0.593) et le hors-sujet le plus fort (0.562) est
/// mince : ce modele compresse les scores. La valeur par defaut ecarte le hors-sujet franc sans
/// couper la recherche semantique, mais elle depend du modele et du corpus — d'ou la surcharge par
/// `JOB_SEARCH_SIMILARITY_FLOOR`.
///
/// Les prefixes de tache de nomic (`search_query:` / `search_document:`) ont ete essayes : ils
/// *reduisent* la separation sur ce corpus (+0.034 contre +0.080), ils ne sont donc pas utilises.
const DEFAULT_SIMILARITY_FLOOR: f64 = 0.55;

fn similarity_floor() -> f64 {
    std::env::var("JOB_SEARCH_SIMILARITY_FLOOR")
        .ok()
        .and_then(|raw| raw.trim().parse::<f64>().ok())
        // Une valeur hors de [0, 1] ne peut pas etre une similarite cosinus : on ignore la
        // surcharge plutot que de filtrer tout ou rien silencieusement.
        .filter(|value| (0.0..=1.0).contains(value))
        .unwrap_or(DEFAULT_SIMILARITY_FLOOR)
}

/// Vrai si l'offre appartient a la source demandee. Une source absente ne filtre rien.
fn matches_source(offer: &JobOffer, source: Option<&str>) -> bool {
    let Some(wanted) = source.map(str::trim).filter(|s| !s.is_empty()) else {
        return true;
    };
    offer
        .source
        .as_deref()
        .is_some_and(|actual| actual.eq_ignore_ascii_case(wanted))
}

/// Replie les diacritiques latins et passe en minuscules, pendant Rust d'`unaccent`.
///
/// Volontairement limite au latin etendu present dans les offres traitees (francais, et les
/// quelques titres anglais ou espagnols des boards) : une table de translitteration complete
/// serait une dependance de plus pour un gain nul ici.
fn fold_accents(value: &str) -> String {
    // Minuscules d'abord, puis repliement : sinon « À » tombe dans la branche par defaut et en
    // ressort « à », toujours accentue.
    value
        .trim()
        .to_lowercase()
        .chars()
        .flat_map(|c| {
            let folded = match c {
                'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' => "a",
                'ç' => "c",
                'è' | 'é' | 'ê' | 'ë' => "e",
                'ì' | 'í' | 'î' | 'ï' => "i",
                'ñ' => "n",
                'ò' | 'ó' | 'ô' | 'õ' | 'ö' => "o",
                'ù' | 'ú' | 'û' | 'ü' => "u",
                'ý' | 'ÿ' => "y",
                'æ' => "ae",
                'œ' => "oe",
                'ß' => "ss",
                _ => return vec![c],
            };
            folded.chars().collect::<Vec<char>>()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn offer_with(title: &str, skills: Option<&str>, body: Option<&str>) -> JobOffer {
        JobOffer {
            title: title.to_string(),
            skills: skills.map(str::to_owned),
            search_text: body.map(str::to_owned),
            ..Default::default()
        }
    }

    #[test]
    fn tokenize_keeps_technology_names_intact() {
        // `c++` et `c#` sont des noms de technologies : les decouper les rendrait introuvables.
        assert_eq!(
            JobSearchService::tokenize("Dev C++ / C#"),
            vec!["dev".to_string(), "c++".to_string(), "c#".to_string()]
        );
    }

    #[test]
    fn tokenize_folds_accents_so_unaccented_input_matches() {
        // Cas reel du corpus emploi.nc : les intitules portent « Developpeur » avec accents,
        // les utilisateurs tapent sans. Les deux saisies doivent produire le meme terme.
        assert_eq!(JobSearchService::tokenize("Développeur"), vec!["developpeur".to_string()]);
        assert_eq!(JobSearchService::tokenize("developpeur"), vec!["developpeur".to_string()]);
    }

    #[test]
    fn fold_accents_handles_ligatures_and_case() {
        assert_eq!(fold_accents("Cœur"), "coeur");
        assert_eq!(fold_accents("ÀÉÎÔÜ"), "aeiou");
        assert_eq!(fold_accents("Straße"), "strasse");
        // Le texte deja sans accent traverse inchange, en minuscules.
        assert_eq!(fold_accents("  Rust  "), "rust");
    }

    #[test]
    fn lexical_score_matches_across_accents() {
        // Le score en memoire doit concorder avec le filtre `unaccent` fait en base : sinon une
        // offre retenue par la requete ressortirait avec un score de zero.
        let terms = JobSearchService::tokenize("developpeur");
        let accented = offer_with("Développeur full-stack", None, None);
        assert_eq!(JobSearchService::lexical_score(&accented, &terms), 2);
    }

    #[test]
    fn tokenize_drops_single_character_noise() {
        // « a » et « l » matcheraient presque toutes les offres sans rien discriminer.
        assert_eq!(
            JobSearchService::tokenize("a developpeur l"),
            vec!["developpeur".to_string()]
        );
    }

    #[test]
    fn lexical_score_weights_the_title_above_the_body() {
        let terms = vec!["rust".to_string()];
        let in_title = offer_with("Developpeur Rust", None, Some("mission generique"));
        let in_body = offer_with("Developpeur", None, Some("un peu de rust a l'occasion"));

        assert_eq!(JobSearchService::lexical_score(&in_title, &terms), 2);
        assert_eq!(JobSearchService::lexical_score(&in_body, &terms), 1);
    }

    #[test]
    fn lexical_score_accumulates_across_terms() {
        let terms = vec!["rust".to_string(), "axum".to_string()];
        let both = offer_with("Developpeur Rust", Some(r#"["axum"]"#), None);
        // Titre (2) + competences (1) : une offre qui couvre les deux termes passe devant.
        assert_eq!(JobSearchService::lexical_score(&both, &terms), 3);

        let neither = offer_with("Comptable", None, Some("saisie"));
        assert_eq!(JobSearchService::lexical_score(&neither, &terms), 0);
    }

    #[test]
    fn similarity_floor_stays_a_valid_cosine_similarity() {
        // Ce test lit une variable de processus sans la modifier : muter l'environnement rendrait
        // les tests dependants de leur ordre d'execution.
        let floor = similarity_floor();
        assert!((0.0..=1.0).contains(&floor));
        if std::env::var("JOB_SEARCH_SIMILARITY_FLOOR").is_err() {
            assert_eq!(floor, DEFAULT_SIMILARITY_FLOOR);
        }
    }

    #[test]
    fn matches_source_ignores_case_and_treats_absent_as_no_filter() {
        let mut offer = offer_with("Dev", None, None);
        offer.source = Some("EMPLOI_NC".to_string());

        assert!(matches_source(&offer, None));
        assert!(matches_source(&offer, Some("")));
        assert!(matches_source(&offer, Some("emploi_nc")));
        assert!(!matches_source(&offer, Some("ADZUNA")));

        // Une offre sans source ne correspond a aucune source demandee explicitement.
        let sourceless = offer_with("Dev", None, None);
        assert!(matches_source(&sourceless, None));
        assert!(!matches_source(&sourceless, Some("EMPLOI_NC")));
    }

    fn offer_named(title: &str, company: &str) -> JobOffer {
        JobOffer {
            title: title.to_string(),
            company: Some(company.to_string()),
            ..Default::default()
        }
    }

    fn offer_at(location: &str, country: Option<&str>, remote: bool) -> JobOffer {
        JobOffer {
            title: format!("Poste a {location}"),
            location: Some(location.to_string()),
            country: country.map(str::to_owned),
            remote: Some(remote),
            ..Default::default()
        }
    }

    #[test]
    fn rerank_by_location_is_a_noop_without_location_or_country() {
        let offers = vec![offer_at("Paris", Some("FR"), false), offer_at("Noumea", Some("NC"), false)];
        let kept = JobSearchService::rerank_by_location(offers, None, None);
        // Sans critere geographique, ni tri ni exclusion : l'ordre des connecteurs est conserve.
        assert_eq!(kept.len(), 2);
        assert_eq!(kept[0].location.as_deref(), Some("Paris"));
    }

    #[test]
    fn rerank_by_location_orders_by_proximity() {
        let offers = vec![
            offer_at("Berlin, Germany", None, false),
            offer_at("Noumea", None, false),
            offer_at("Dumbea, Noumea Sud", None, false),
        ];
        let kept = JobSearchService::rerank_by_location(offers, Some("Noumea"), None);
        // La correspondance exacte passe devant l'inclusion, qui passe devant l'absence de lien.
        assert_eq!(kept[0].location.as_deref(), Some("Noumea"));
        assert_eq!(kept.last().unwrap().location.as_deref(), Some("Berlin, Germany"));
    }

    #[test]
    fn rerank_by_location_never_drops_on_a_textual_mismatch_alone() {
        // Point central : « Berlin » n'a aucun rapport textuel avec « Noumea », mais sans code pays
        // demande on ne peut pas l'affirmer — l'offre doit etre reclassee, jamais supprimee.
        let offers = vec![offer_at("Berlin, Germany", Some("DE"), false)];
        let kept = JobSearchService::rerank_by_location(offers, Some("Noumea"), None);
        assert_eq!(kept.len(), 1, "aucune exclusion sans pays demande");
    }

    #[test]
    fn rerank_by_location_drops_only_on_a_proven_country_mismatch() {
        let offers = vec![
            offer_at("Paris", Some("FR"), false),
            offer_at("New York", Some("US"), false),
            // Pays inconnu : on ne conclut pas, elle reste.
            offer_at("Quelque part", None, false),
        ];
        let kept = JobSearchService::rerank_by_location(offers, None, Some("fr"));
        let countries: Vec<Option<&str>> = kept.iter().map(|o| o.country.as_deref()).collect();
        assert!(countries.contains(&Some("FR")));
        assert!(countries.contains(&None), "un pays inconnu ne suffit pas a exclure");
        assert!(!countries.contains(&Some("US")), "le pays preuve doit exclure");
    }

    #[test]
    fn rerank_by_location_keeps_remote_offers_wherever_they_are() {
        let offers = vec![
            offer_at("Berlin, Germany", None, true),
            offer_at("Berlin, Germany", None, false),
        ];
        let kept = JobSearchService::rerank_by_location(offers, Some("Noumea"), None);
        // Le teletravail est tenable depuis Noumea : il passe devant le meme poste sur site.
        assert_eq!(kept[0].remote, Some(true));
    }

    #[test]
    fn diversify_caps_the_same_role_at_the_same_company() {
        // Cas reel : « Applied AI Engineer » chez openai existe en cinq villes. Ce sont cinq postes
        // distincts, mais ils ne doivent pas monopoliser la page.
        let offers = vec![
            offer_named("Applied AI Engineer", "openai"),
            offer_named("Applied AI Engineer", "openai"),
            offer_named("Applied AI Engineer", "openai"),
            offer_named("Software Engineer", "openai"),
            offer_named("Data Engineer", "scaleway"),
        ];
        // Limite a 4 : il y a assez d'autres roles pour remplir la page, donc le repli ne se
        // declenche pas et le plafond s'observe seul. (Le repli a son propre test.)
        let kept = JobSearchService::diversify(offers, 4);
        let applied = kept.iter().filter(|o| o.title == "Applied AI Engineer").count();
        assert_eq!(applied, MAX_PER_ROLE, "le plafond par role doit s'appliquer");
        // Les places liberees profitent aux autres roles.
        assert!(kept.iter().any(|o| o.title == "Software Engineer"));
        assert!(kept.iter().any(|o| o.title == "Data Engineer"));
    }

    #[test]
    fn diversify_distinguishes_companies() {
        // Meme intitule mais entreprises differentes : ce ne sont pas les memes offres, le plafond
        // ne doit pas les confondre.
        let offers = vec![
            offer_named("Developpeur full-stack", "acme"),
            offer_named("Developpeur full-stack", "globex"),
            offer_named("Developpeur full-stack", "initech"),
        ];
        assert_eq!(JobSearchService::diversify(offers, 5).len(), 3);
    }

    #[test]
    fn diversify_backfills_rather_than_returning_less_than_asked() {
        // Si le plafond vidait la page, mieux vaut la completer avec les offres ecartees que
        // rendre trois resultats quand l'utilisateur en demande cinq.
        let offers = vec![
            offer_named("Meme poste", "acme"),
            offer_named("Meme poste", "acme"),
            offer_named("Meme poste", "acme"),
            offer_named("Meme poste", "acme"),
            offer_named("Meme poste", "acme"),
        ];
        assert_eq!(JobSearchService::diversify(offers, 4).len(), 4);
    }

    #[test]
    fn diversify_is_case_and_whitespace_insensitive() {
        // Les sources ne normalisent pas : « openai » et « OpenAI » sont la meme entreprise.
        let offers = vec![
            offer_named("Applied AI Engineer", "openai"),
            offer_named("  applied ai engineer  ", "OpenAI"),
            offer_named("Applied AI Engineer", "openai"),
        ];
        assert_eq!(JobSearchService::diversify(offers, 3).len(), 3, "le repli complete la page");
        // Mais le plafond a bien vu les trois comme un seul groupe : sans cela, aucune ne serait
        // passee par la branche de repli.
        let kept = JobSearchService::diversify(
            vec![
                offer_named("Role", "acme"),
                offer_named("ROLE", "ACME"),
                offer_named("role", "Acme"),
                offer_named("Autre", "acme"),
            ],
            3,
        );
        assert_eq!(kept.iter().filter(|o| o.title.eq_ignore_ascii_case("role")).count(), 2);
        assert!(kept.iter().any(|o| o.title == "Autre"));
    }

    #[test]
    fn diversify_preserves_relevance_order() {
        // La diversification s'applique apres le reclassement : elle ne doit pas reordonner.
        let offers = vec![
            offer_named("Premier", "a"),
            offer_named("Deuxieme", "b"),
            offer_named("Troisieme", "c"),
        ];
        let kept = JobSearchService::diversify(offers, 3);
        let titles: Vec<&str> = kept.iter().map(|o| o.title.as_str()).collect();
        assert_eq!(titles, vec!["Premier", "Deuxieme", "Troisieme"]);
    }

    #[test]
    fn build_search_text_concatenates_available_fields_only() {
        let offer = JobOffer {
            title: "Dev".to_string(),
            company: Some("ACME".to_string()),
            location: None,
            description: Some("Mission".to_string()),
            ..Default::default()
        };
        // Les champs absents ne laissent pas de separateur vide derriere eux.
        assert_eq!(JobSearchService::build_search_text(&offer), "Dev ACME Mission");
    }
}
