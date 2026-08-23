//! Outils que l'agent Job Copilot peut appeler, portage du `JobAgentToolbox` de l'app Spring.
//!
//! Chaque outil porte le pool **et le login de l'utilisateur connecte**. C'est le point de
//! conception important : la signature vue par le modele n'expose aucun identifiant
//! d'utilisateur, donc le modele ne peut ni en inventer un ni agir pour quelqu'un d'autre.
//! L'isolation est structurelle, pas dependante du prompt systeme.
//!
//! Les sorties sont du JSON compact plutot que de la prose : le modele les relit, et une prose
//! generee ici serait a la fois plus chere en jetons et plus facile a mal interpreter.

use std::collections::HashMap;
use std::sync::Arc;

use diesel::prelude::*;
use rig::tool::{Tool, ToolContext};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::db::connection::DbPool;
use crate::db::schema::{candidate_profile, job_application, job_offer};
use crate::dto::job_copilot_dto::JobOfferSummaryDto;
use crate::dto::parse_string_list;
use crate::errors::AppError;
use crate::models::{CandidateProfile, JobApplication, JobOffer};
use crate::services::cover_letter_service::CoverLetterService;
use crate::services::job_matching_service::JobMatchingService;
use crate::services::job_offer_vector_index::JobOfferVectorIndex;
use crate::services::job_search_service::JobSearchService;

/// Nombre d'offres renvoyees par defaut a l'agent.
///
/// Volontairement bas : chaque offre rendue au modele coute des jetons, et une liste de trente
/// annonces lui fait perdre le fil autant qu'a un lecteur humain.
const DEFAULT_TOOL_LIMIT: i64 = 5;
const MAX_TOOL_LIMIT: i64 = 10;

/// Schemas de parametres, definis une seule fois.
///
/// `parameters()` les renvoie et les tests les inspectent : sans cette factorisation, un test qui
/// recopie le schema ne verifierait qu'une copie, et laisserait passer l'ajout d'un parametre
/// `userId` sur l'outil reel.
pub mod schemas {
    use serde_json::json;

    pub fn search() -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Ce que cherche l'utilisateur, en langage naturel"
                },
                "limit": {
                    "type": "integer",
                    "description": "Nombre d'offres a renvoyer"
                }
            },
            "required": ["query"]
        })
    }

    pub fn match_profile() -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "La demande de l'utilisateur, avec ses propres mots"
                },
                "limit": {
                    "type": "integer",
                    "description": "Nombre d'offres a scorer"
                }
            },
            "required": ["query"]
        })
    }

    pub fn empty() -> serde_json::Value {
        json!({ "type": "object", "properties": {} })
    }

    pub fn cover_letter() -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "jobOfferId": {
                    "type": "string",
                    "description": "Identifiant (UUID) de l'offre, issu d'un autre outil"
                },
                "language": {
                    "type": "string",
                    "description": "Langue de la lettre (defaut: fr)"
                }
            },
            "required": ["jobOfferId"]
        })
    }

    /// Tous les schemas exposes au modele, pour les verifications transverses.
    pub fn all() -> Vec<serde_json::Value> {
        vec![search(), match_profile(), empty(), cover_letter()]
    }
}

/// Erreur d'outil. Le modele voit ce message : il doit etre exploitable par lui (« importe un CV
/// d'abord ») et jamais fuir de detail d'infrastructure.
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct ToolError(String);

impl From<AppError> for ToolError {
    fn from(error: AppError) -> Self {
        match error {
            // Un message metier est utile au modele, il peut se corriger.
            AppError::BadRequest(message)
            | AppError::NotFound(message)
            | AppError::Validation(message) => Self(message),
            // Tout le reste est une panne : on ne detaille pas.
            _ => Self("Outil momentanement indisponible".to_string()),
        }
    }
}

/// Contexte partage par tous les outils : de quoi lire la base et savoir pour qui on agit.
#[derive(Clone)]
pub struct AgentToolContext {
    pub pool: DbPool,
    pub user_id: Arc<str>,
    pub index: JobOfferVectorIndex,
}

impl AgentToolContext {
    pub fn new(pool: DbPool, user_id: &str, index: JobOfferVectorIndex) -> Self {
        Self {
            pool,
            user_id: Arc::from(user_id),
            index,
        }
    }

    fn conn(&self) -> Result<crate::db::DbConnection, ToolError> {
        self.pool
            .get()
            .map_err(|_| ToolError("Base de donnees indisponible".to_string()))
    }

    fn profile(&self) -> Result<CandidateProfile, ToolError> {
        let mut conn = self.conn()?;
        candidate_profile::table
            .filter(candidate_profile::user_id.eq(self.user_id.as_ref()))
            .order(candidate_profile::created_at.desc().nulls_last())
            .select(CandidateProfile::as_select())
            .first(&mut conn)
            .optional()
            .map_err(|_| ToolError("Lecture du profil impossible".to_string()))?
            .ok_or_else(|| {
                ToolError(
                    "Cet utilisateur n'a pas encore de profil : il doit importer son CV."
                        .to_string(),
                )
            })
    }
}

// ---------------------------------------------------------------------------
// Recherche d'offres
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct SearchArgs {
    /// Requete en langage naturel.
    pub query: String,
    #[serde(default)]
    pub limit: Option<i64>,
}

/// Recherche semantique dans le corpus indexe.
pub struct SearchJobOffersTool(pub AgentToolContext);

impl Tool for SearchJobOffersTool {
    const NAME: &'static str = "search_job_offers";
    type Args = SearchArgs;
    type Output = serde_json::Value;
    type Error = ToolError;

    fn description(&self) -> String {
        "Recherche des offres d'emploi dans la base indexee par similarite semantique. \
         Utiliser pour toute demande d'offres correspondant a un metier, une technologie ou un \
         lieu. Renvoie l'identifiant de chaque offre, necessaire aux autres outils."
            .to_string()
    }

    fn parameters(&self) -> serde_json::Value {
        schemas::search()
    }

    async fn call(
        &self,
        _context: &mut ToolContext,
        args: Self::Args,
    ) -> Result<Self::Output, Self::Error> {
        let limit = args
            .limit
            .unwrap_or(DEFAULT_TOOL_LIMIT)
            .clamp(1, MAX_TOOL_LIMIT);

        let offers =
            JobSearchService::search_semantic(&self.0.index, &self.0.pool, &args.query, None, limit)
                .await?;

        let results: Vec<JobOfferSummaryDto> =
            offers.iter().map(JobOfferSummaryDto::from).collect();

        Ok(json!({ "count": results.len(), "offers": results }))
    }
}

// ---------------------------------------------------------------------------
// Matching profil <-> offres
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct MatchArgs {
    /// Requete decrivant ce que cherche l'utilisateur, avec ses propres mots.
    pub query: String,
    #[serde(default)]
    pub limit: Option<i64>,
}

/// Score les offres contre le CV de l'utilisateur.
pub struct MatchOffersToProfileTool(pub AgentToolContext);

impl Tool for MatchOffersToProfileTool {
    const NAME: &'static str = "match_offers_to_profile";
    type Args = MatchArgs;
    type Output = serde_json::Value;
    type Error = ToolError;

    fn description(&self) -> String {
        "Score les offres d'emploi contre le CV de l'utilisateur connecte et renvoie, pour \
         chacune, un score et les raisons (competences retrouvees ou manquantes, lieu, salaire). \
         Utiliser des que la question porte sur ce qui correspond a SON profil : « les meilleures \
         offres pour moi », « a quoi postuler », « est-ce que cette offre me correspond »."
            .to_string()
    }

    fn parameters(&self) -> serde_json::Value {
        schemas::match_profile()
    }

    async fn call(
        &self,
        _context: &mut ToolContext,
        args: Self::Args,
    ) -> Result<Self::Output, Self::Error> {
        let profile = self.0.profile()?;
        let limit = args
            .limit
            .unwrap_or(DEFAULT_TOOL_LIMIT)
            .clamp(1, MAX_TOOL_LIMIT);

        let mut conn = self.0.conn()?;
        let offers =
            JobSearchService::search_semantic(&self.0.index, &self.0.pool, &args.query, None, limit)
                .await?;

        let preferences = crate::db::schema::user_preference::table
            .filter(crate::db::schema::user_preference::user_id.eq(self.0.user_id.as_ref()))
            .select(crate::models::UserPreference::as_select())
            .first(&mut conn)
            .optional()
            .map_err(|_| ToolError("Lecture des preferences impossible".to_string()))?;

        let results =
            JobMatchingService::match_jobs(&profile, &offers, preferences.as_ref(), None);

        Ok(json!({ "count": results.len(), "matches": results }))
    }
}

// ---------------------------------------------------------------------------
// Profil candidat
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct NoArgs {}

/// Renvoie le profil extrait du CV.
pub struct GetCandidateProfileTool(pub AgentToolContext);

/// Vue du profil transmise au modele. Le markdown brut du CV en est exclu : plusieurs milliers de
/// jetons pour une information deja resumee par les champs structures.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProfileView {
    full_name: Option<String>,
    location: Option<String>,
    years_of_experience: Option<i32>,
    skills: Vec<String>,
    preferred_roles: Vec<String>,
}

impl Tool for GetCandidateProfileTool {
    const NAME: &'static str = "get_candidate_profile";
    type Args = NoArgs;
    type Output = serde_json::Value;
    type Error = ToolError;

    fn description(&self) -> String {
        "Renvoie le profil candidat de l'utilisateur connecte, extrait de son CV : identite, \
         lieu, annees d'experience, competences, postes recherches."
            .to_string()
    }

    fn parameters(&self) -> serde_json::Value {
        schemas::empty()
    }

    async fn call(
        &self,
        _context: &mut ToolContext,
        _args: Self::Args,
    ) -> Result<Self::Output, Self::Error> {
        let profile = self.0.profile()?;
        let view = ProfileView {
            full_name: profile.full_name.clone(),
            location: profile.location.clone(),
            years_of_experience: profile.years_of_experience,
            skills: parse_string_list(profile.skills.as_deref()),
            preferred_roles: parse_string_list(profile.preferred_roles.as_deref()),
        };
        serde_json::to_value(view).map_err(|_| ToolError("Profil illisible".to_string()))
    }
}

// ---------------------------------------------------------------------------
// Candidatures
// ---------------------------------------------------------------------------

/// Liste les candidatures de l'utilisateur, avec leur statut.
pub struct ListApplicationsTool(pub AgentToolContext);

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ApplicationView {
    id: Uuid,
    status: String,
    job_title: Option<String>,
    company: Option<String>,
    applied_at: Option<String>,
}

impl Tool for ListApplicationsTool {
    const NAME: &'static str = "list_applications";
    type Args = NoArgs;
    type Output = serde_json::Value;
    type Error = ToolError;

    fn description(&self) -> String {
        "Liste les candidatures de l'utilisateur connecte avec leur statut, l'intitule du poste \
         et l'entreprise, plus le nombre de candidatures par statut."
            .to_string()
    }

    fn parameters(&self) -> serde_json::Value {
        schemas::empty()
    }

    async fn call(
        &self,
        _context: &mut ToolContext,
        _args: Self::Args,
    ) -> Result<Self::Output, Self::Error> {
        let mut conn = self.0.conn()?;

        let rows: Vec<(JobApplication, Option<JobOffer>)> = job_application::table
            .left_join(job_offer::table.on(job_offer::id.nullable().eq(job_application::jobOffer_id)))
            .filter(job_application::user_id.eq(self.0.user_id.as_ref()))
            .order(job_application::created_at.desc().nulls_last())
            .select((JobApplication::as_select(), Option::<JobOffer>::as_select()))
            .load(&mut conn)
            .map_err(|_| ToolError("Lecture des candidatures impossible".to_string()))?;

        let mut by_status: HashMap<String, usize> = HashMap::new();
        let applications: Vec<ApplicationView> = rows
            .iter()
            .map(|(application, offer)| {
                let status = application
                    .status
                    .clone()
                    .unwrap_or_else(|| "DRAFT".to_string());
                *by_status.entry(status.clone()).or_insert(0) += 1;
                ApplicationView {
                    id: application.id,
                    status,
                    job_title: offer.as_ref().map(|o| o.title.clone()),
                    company: offer.as_ref().and_then(|o| o.company.clone()),
                    applied_at: application.applied_at.map(|d| d.to_string()),
                }
            })
            .collect();

        Ok(json!({
            "total": applications.len(),
            "byStatus": by_status,
            "applications": applications
        }))
    }
}

// ---------------------------------------------------------------------------
// Lettre de motivation
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverLetterArgs {
    /// Identifiant de l'offre, tel que renvoye par `search_job_offers`.
    pub job_offer_id: String,
    #[serde(default)]
    pub language: Option<String>,
}

/// Redige une lettre de motivation pour une offre.
pub struct GenerateCoverLetterTool(pub AgentToolContext);

impl Tool for GenerateCoverLetterTool {
    const NAME: &'static str = "generate_cover_letter";
    type Args = CoverLetterArgs;
    type Output = serde_json::Value;
    type Error = ToolError;

    fn description(&self) -> String {
        "Redige une lettre de motivation personnalisee pour une offre donnee, a partir du CV de \
         l'utilisateur connecte. L'identifiant d'offre doit venir de search_job_offers ou de \
         match_offers_to_profile : ne jamais l'inventer."
            .to_string()
    }

    fn parameters(&self) -> serde_json::Value {
        schemas::cover_letter()
    }

    async fn call(
        &self,
        _context: &mut ToolContext,
        args: Self::Args,
    ) -> Result<Self::Output, Self::Error> {
        // Un identifiant mal forme est une erreur du modele, pas une panne : on le lui dit
        // clairement pour qu'il rappelle l'outil de recherche.
        let offer_id = Uuid::parse_str(args.job_offer_id.trim()).map_err(|_| {
            ToolError(format!(
                "'{}' n'est pas un identifiant d'offre valide. Utiliser search_job_offers pour en obtenir un.",
                args.job_offer_id
            ))
        })?;

        let (profile, offer) = {
            let mut conn = self.0.conn()?;
            let profile = self.0.profile()?;
            let offer: JobOffer = job_offer::table
                .find(offer_id)
                .select(JobOffer::as_select())
                .first(&mut conn)
                .optional()
                .map_err(|_| ToolError("Lecture de l'offre impossible".to_string()))?
                .ok_or_else(|| ToolError("Offre inconnue".to_string()))?;
            (profile, offer)
        };

        let letter =
            CoverLetterService::generate(&profile, &offer, args.language.as_deref()).await?;

        Ok(json!({ "jobOfferId": offer_id, "coverLetter": letter }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_names_are_stable_and_distinct() {
        // Le prompt systeme cite ces noms : les renommer sans le mettre a jour rendrait les
        // consignes inoperantes.
        let names = [
            SearchJobOffersTool::NAME,
            MatchOffersToProfileTool::NAME,
            GetCandidateProfileTool::NAME,
            ListApplicationsTool::NAME,
            GenerateCoverLetterTool::NAME,
        ];
        let unique: std::collections::HashSet<&&str> = names.iter().collect();
        assert_eq!(unique.len(), names.len(), "noms d'outils en collision");
        assert_eq!(SearchJobOffersTool::NAME, "search_job_offers");
        assert_eq!(GenerateCoverLetterTool::NAME, "generate_cover_letter");
    }

    #[test]
    fn no_tool_exposes_a_user_identifier_parameter() {
        // Garantie centrale : le modele ne peut pas agir pour le compte d'un autre utilisateur,
        // parce qu'aucun outil ne lui offre de parametre pour le designer. Le test lit les
        // schemas reellement renvoyes par `parameters()`, pas une recopie.
        for schema in schemas::all() {
            let properties = schema["properties"]
                .as_object()
                .expect("un schema de parametres est un objet");
            for forbidden in ["userId", "user_id", "login", "user", "owner"] {
                assert!(
                    !properties.contains_key(forbidden),
                    "{forbidden} ne doit pas etre un parametre d'outil : schema {schema}"
                );
            }
        }
    }

    #[test]
    fn every_schema_is_a_valid_json_schema_object() {
        // Un schema mal forme fait rejeter la definition d'outil par le provider, avec une erreur
        // opaque cote appel modele : autant le voir ici.
        for schema in schemas::all() {
            assert_eq!(schema["type"], "object");
            assert!(schema["properties"].is_object());
            if let Some(required) = schema.get("required") {
                let properties = schema["properties"].as_object().unwrap();
                for name in required.as_array().unwrap() {
                    let name = name.as_str().unwrap();
                    assert!(
                        properties.contains_key(name),
                        "{name} est requis mais absent des proprietes"
                    );
                }
            }
        }
    }

    #[test]
    fn app_error_is_translated_without_leaking_infrastructure() {
        // Un message metier remonte tel quel : le modele peut s'en servir.
        let business = ToolError::from(AppError::BadRequest("importe un CV".into()));
        assert_eq!(business.to_string(), "importe un CV");

        // Une panne interne est masquee : pas de detail de base ni de trace dans le contexte.
        let internal = ToolError::from(AppError::Internal(
            "connection refused on 127.0.0.1:5438".into(),
        ));
        assert_eq!(internal.to_string(), "Outil momentanement indisponible");
        assert!(!internal.to_string().contains("5438"));
    }

    #[test]
    fn limits_are_clamped_to_a_token_affordable_range() {
        // Un `limit` venu du modele ne doit pas pouvoir faire exploser le contexte.
        for (asked, expected) in [(0i64, 1i64), (3, 3), (999, MAX_TOOL_LIMIT)] {
            assert_eq!(asked.clamp(1, MAX_TOOL_LIMIT), expected);
        }
    }
}
