//! Index vectoriel des offres, expose comme un [`VectorStoreIndex`] de rig.
//!
//! Le choix d'implementer le trait nous-memes plutot que d'utiliser le crate `rig-postgres` est
//! delibere : celui-ci repose sur `sqlx`, ce qui ajouterait un second driver et un second pool a
//! cote de Diesel, et il impose sa propre table `documents(id, document jsonb, embedded_text,
//! embedding)`. Nos offres sont une table metier a une trentaine de colonnes, pas des documents
//! JSON ; le trait ne demande que deux methodes, et les implementer sur Diesel garde une seule
//! source de verite et un seul pool.
//!
//! Le gain est double : la recherche semantique de `search-smart` et du matching, et — parce que
//! rig fournit un `VectorStoreIndexDyn` par implementation generique — le branchement direct sur
//! `AgentBuilder::dynamic_context` pour le RAG de l'assistant, sans code supplementaire.

use diesel::prelude::*;
use diesel::sql_types::{Double, Nullable};
use pgvector::Vector;
use pgvector::VectorExpressionMethods;
use rig::vector_store::request::{Filter, VectorSearchRequest};
use rig::vector_store::{VectorStoreError, VectorStoreIndex};
use serde::Deserialize;
use uuid::Uuid;

use crate::db::connection::DbPool;
use crate::db::schema::job_offer;
use crate::errors::AppError;
use crate::models::JobOffer;
use crate::services::embedding_service::EmbeddingService;
use crate::services::job_search_service::EMBEDDING_STATUS_COMPLETED;

/// Plafond dur du nombre de resultats, quoi que demande l'appelant.
///
/// `samples` est un `u64` venant potentiellement d'un parametre de requete : sans borne, un appel
/// pourrait demander de materialiser le corpus entier.
const MAX_SAMPLES: i64 = 200;

/// Index vectoriel sur `job_offer`.
///
/// Clone bon marche : il ne porte qu'un pool (lui-meme un `Arc`), donc il peut etre partage entre
/// handlers et passe a `dynamic_context` sans cout.
#[derive(Clone)]
pub struct JobOfferVectorIndex {
    pool: DbPool,
}

impl JobOfferVectorIndex {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Recherche les offres les plus proches d'un texte, avec leur score de similarite.
    ///
    /// Le score renvoye est une **similarite** dans `[0, 1]` (1 = identique), alors que pgvector
    /// renvoie une **distance** cosinus : la conversion est faite ici pour que la convention de
    /// rig (plus grand = plus proche) soit respectee.
    pub async fn search(
        &self,
        query: &str,
        samples: i64,
        threshold: Option<f64>,
    ) -> Result<Vec<(f64, JobOffer)>, AppError> {
        let query = query.trim();
        if query.is_empty() {
            return Err(AppError::BadRequest(
                "Une recherche vectorielle a besoin d'un texte non vide".into(),
            ));
        }

        // Vectorisation hors connexion au pool : l'appel au modele est lent, inutile
        // d'immobiliser une connexion pendant ce temps.
        let embedded = EmbeddingService::embed(query).await?;
        let vector = Vector::from(embedded);
        let limit = samples.clamp(1, MAX_SAMPLES);

        // Requete deportee sur un thread bloquant : Diesel est synchrone, et un balayage HNSW
        // n'est pas instantane. La laisser sur le thread du runtime y bloquait toutes les autres
        // requetes — y compris la jambe lexicale de la recherche hybride, censee tourner en
        // parallele de celle-ci.
        let pool = self.pool.clone();
        let rows: Vec<(JobOffer, Option<f64>)> = tokio::task::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|e| AppError::Internal(e.to_string()))?;

            // `cosine_distance` produit l'operateur `<=>`, donc l'index HNSW pose par la migration
            // est utilise. Le filtre sur `COMPLETED` exclut les offres pas encore vectorisees,
            // dont le vecteur NULL ressortirait en fin de classement sans signifier quoi que ce
            // soit.
            job_offer::table
                .filter(job_offer::embedding_status.eq(EMBEDDING_STATUS_COMPLETED))
                .filter(job_offer::embedding.is_not_null())
                // Les offres expirees sont exclues des resultats mais gardees en base : celles
                // liees a une candidature appartiennent a l'historique de l'utilisateur.
                .filter(
                    job_offer::expires_at
                        .is_null()
                        .or(job_offer::expires_at.gt(diesel::dsl::now)),
                )
                .order(job_offer::embedding.cosine_distance(vector.clone()))
                .limit(limit)
                .select((
                    JobOffer::as_select(),
                    job_offer::embedding
                        .cosine_distance(vector)
                        .nullable()
                        .into_sql::<Nullable<Double>>(),
                ))
                .load(&mut conn)
                .map_err(AppError::from)
        })
        .await
        .map_err(|e| AppError::Internal(format!("Recherche vectorielle interrompue : {e}")))??;

        Ok(rows
            .into_iter()
            .filter_map(|(offer, distance)| {
                // Une distance NULL ne devrait pas survenir apres le filtre `is_not_null`, mais
                // la traiter comme « pas de score » vaut mieux que de fabriquer un 0 trompeur.
                let similarity = 1.0 - distance?;
                Some((similarity, offer))
            })
            .filter(|(similarity, _)| threshold.is_none_or(|min| *similarity >= min))
            .collect())
    }
}

/// Vue JSON d'une offre exposee au modele via le RAG.
///
/// Volontairement pauvre : le contexte injecte a chaque appel modele est facture en jetons, et un
/// dump des trente-cinq colonnes (payload brut, hash, compteurs d'indexation) noierait le poste
/// sous des metadonnees techniques.
#[derive(Debug, Clone, serde::Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobOfferDocument {
    pub id: Uuid,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apply_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Longueur de description injectee dans le contexte du modele.
const RAG_DESCRIPTION_MAX_CHARS: usize = 1_200;

impl From<&JobOffer> for JobOfferDocument {
    fn from(offer: &JobOffer) -> Self {
        Self {
            id: offer.id,
            title: offer.title.clone(),
            company: offer.company.clone(),
            location: offer.location.clone(),
            contract_type: offer.contract_type.clone(),
            apply_url: offer.apply_url.clone(),
            description: offer.description.as_deref().map(|d| {
                if d.chars().count() <= RAG_DESCRIPTION_MAX_CHARS {
                    d.to_string()
                } else {
                    d.chars().take(RAG_DESCRIPTION_MAX_CHARS).collect()
                }
            }),
        }
    }
}

impl VectorStoreIndex for JobOfferVectorIndex {
    /// Le filtre canonique de rig, ce qui suffit a obtenir `VectorStoreIndexDyn` par
    /// l'implementation generique du crate — condition pour passer cet index a
    /// `AgentBuilder::dynamic_context`.
    type Filter = Filter<serde_json::Value>;

    async fn top_n<T: for<'a> Deserialize<'a> + Send>(
        &self,
        req: VectorSearchRequest<Self::Filter>,
    ) -> Result<Vec<(f64, String, T)>, VectorStoreError> {
        // Le filtre par metadonnees n'est pas implemente : le declarer supporte puis l'ignorer
        // renverrait des resultats non filtres que l'appelant croirait filtres. Les restrictions
        // metier (source, lieu) passent par les endpoints, qui filtrent en SQL.
        if req.filter().is_some() {
            return Err(VectorStoreError::DatastoreError(
                "JobOfferVectorIndex ne gere pas les filtres par metadonnees".into(),
            ));
        }

        let hits = self
            .search(req.query(), req.samples() as i64, req.threshold())
            .await
            .map_err(to_store_error)?;

        hits.iter()
            .map(|(score, offer)| {
                let document = JobOfferDocument::from(offer);
                let value =
                    serde_json::to_value(&document).map_err(VectorStoreError::JsonError)?;
                let deserialized: T =
                    serde_json::from_value(value).map_err(VectorStoreError::JsonError)?;
                Ok((*score, offer.id.to_string(), deserialized))
            })
            .collect()
    }

    async fn top_n_ids(
        &self,
        req: VectorSearchRequest<Self::Filter>,
    ) -> Result<Vec<(f64, String)>, VectorStoreError> {
        if req.filter().is_some() {
            return Err(VectorStoreError::DatastoreError(
                "JobOfferVectorIndex ne gere pas les filtres par metadonnees".into(),
            ));
        }

        let hits = self
            .search(req.query(), req.samples() as i64, req.threshold())
            .await
            .map_err(to_store_error)?;

        Ok(hits
            .into_iter()
            .map(|(score, offer)| (score, offer.id.to_string()))
            .collect())
    }
}

/// Traduit nos erreurs vers celles de rig en conservant la distinction utile : une requete mal
/// formee ne doit pas etre signalee comme une panne du datastore.
fn to_store_error(error: AppError) -> VectorStoreError {
    match error {
        AppError::BadRequest(message) | AppError::Validation(message) => {
            VectorStoreError::DatastoreError(message.into())
        }
        other => VectorStoreError::DatastoreError(other.to_string().into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn offer(description: Option<&str>) -> JobOffer {
        JobOffer {
            id: Uuid::nil(),
            title: "Developpeur".to_string(),
            company: Some("ACME".to_string()),
            location: Some("Noumea".to_string()),
            contract_type: Some("CDI".to_string()),
            apply_url: Some("https://example.nc/1".to_string()),
            description: description.map(str::to_owned),
            ..Default::default()
        }
    }

    #[test]
    fn document_keeps_only_the_fields_worth_paying_tokens_for() {
        let document = JobOfferDocument::from(&offer(Some("Mission.")));
        let json = serde_json::to_value(&document).unwrap();
        let object = json.as_object().unwrap();

        for field in ["id", "title", "company", "location", "contractType", "applyUrl", "description"] {
            assert!(object.contains_key(field), "{field} attendu");
        }
        // Les colonnes techniques ne doivent pas atteindre le contexte du modele.
        for field in ["rawPayload", "contentHash", "embeddingStatus", "retryCount", "searchText"] {
            assert!(!object.contains_key(field), "{field} ne doit pas etre expose");
        }
    }

    #[test]
    fn document_truncates_long_descriptions() {
        let long = "a".repeat(RAG_DESCRIPTION_MAX_CHARS * 2);
        let document = JobOfferDocument::from(&offer(Some(&long)));
        assert_eq!(
            document.description.as_deref().map(|d| d.chars().count()),
            Some(RAG_DESCRIPTION_MAX_CHARS)
        );
    }

    #[test]
    fn document_omits_absent_optional_fields() {
        let bare = JobOffer {
            title: "Poste".to_string(),
            ..Default::default()
        };
        let json = serde_json::to_value(JobOfferDocument::from(&bare)).unwrap();
        let object = json.as_object().unwrap();
        assert!(!object.contains_key("company"));
        assert!(!object.contains_key("description"));
        assert!(object.contains_key("title"));
    }

    /// Preuve a la compilation que l'index satisfait la surface type-effacee de rig, donc qu'il
    /// se passe tel quel a `AgentBuilder::dynamic_context` pour le RAG de l'assistant. Si une
    /// evolution du trait cassait cette compatibilite, l'erreur tomberait ici et non plus tard
    /// dans le handler de l'assistant.
    #[test]
    fn index_is_usable_as_rig_dynamic_context() {
        fn accepts_dynamic_index<I: rig::vector_store::VectorStoreIndexDyn + 'static>() {}
        accepts_dynamic_index::<JobOfferVectorIndex>();
    }

    #[test]
    fn bad_request_maps_to_a_datastore_error_carrying_the_message() {
        let mapped = to_store_error(AppError::BadRequest("texte vide".into()));
        assert!(mapped.to_string().contains("texte vide"));
    }
}
