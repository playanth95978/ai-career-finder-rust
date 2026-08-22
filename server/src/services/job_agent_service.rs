//! Agent conversationnel Job Copilot, portage du `JobCopilotAgentService` de l'app Spring.
//!
//! Tout ce qui etait a construire a la main cote Java est ici assure par rig :
//!
//!  - la memoire de conversation, via [`PostgresConversationMemory`] passe a `.memory()` : rig
//!    charge l'historique avant le prompt et ecrit le tour complet apres succes, une seule fois
//!    par run meme quand le tour a comporte plusieurs appels au modele, et rien du tout en cas
//!    d'echec ;
//!  - la boucle d'appels d'outils.
//!
//! # Pourquoi la recuperation est declenchee par outil et non par `dynamic_context`
//!
//! Le premier branchement utilisait `.dynamic_context(3, index)`, qui injecte les offres les plus
//! proches **a chaque appel modele**. Mesure faite : sur le message « Retiens que je m'appelle
//! Tryphon Tournesol », l'agent a repondu « Je vois que vous avez partage des offres d'emploi ».
//! Les documents injectes arrivent sans cadrage, le modele les a donc attribues a l'utilisateur.
//!
//! La recuperation reste vectorielle et passe par le meme [`JobOfferVectorIndex`], mais via les
//! outils `search_job_offers` et `match_offers_to_profile` : c'est le modele qui decide quand
//! chercher. On y gagne trois choses — plus de contexte parasite dans les conversations qui ne
//! parlent pas d'emploi, aucun jeton depense quand la question n'appelle pas de recherche, et une
//! provenance explicite (le modele sait que les offres viennent d'un outil qu'il a appele).
//!
//! L'agent est reconstruit a chaque requete : ses outils portent le login de l'utilisateur
//! connecte, donc il ne peut pas etre partage entre utilisateurs. Le cout est negligeable — le
//! builder n'ouvre aucune connexion — et l'isolation devient structurelle.

use rig::agent::Agent;
use rig::client::{AgentClientExt, ProviderClient};
use rig::providers::mistral;

use crate::db::connection::DbPool;
use crate::errors::AppError;
use crate::services::ai_service::AiService;
use crate::services::conversation_memory::PostgresConversationMemory;
use crate::services::job_agent_tools::{
    AgentToolContext, GenerateCoverLetterTool, GetCandidateProfileTool, ListApplicationsTool,
    MatchOffersToProfileTool, SearchJobOffersTool,
};
use crate::services::job_offer_vector_index::JobOfferVectorIndex;

/// Budget d'appels modele pour un tour, appels d'outils compris.
///
/// Sans plafond, un modele qui boucle sur un outil consommerait indefiniment. Six laisse la place
/// a deux ou trois allers-retours d'outils suivis de la reponse finale.
const MAX_TURNS: usize = 6;

/// Prompt systeme, transcrit de la version Java. Les noms d'outils cites doivent rester alignes
/// sur les constantes `NAME` de [`crate::services::job_agent_tools`].
const SYSTEM_PROMPT: &str = "\
Tu es Job Copilot, un assistant pour le marche de l'emploi en Nouvelle-Caledonie (et pour les \
postes en teletravail ou a l'international).

Tu aides l'utilisateur connecte a chercher des offres, a comparer les offres avec son profil de \
CV, a rediger des lettres de motivation et a suivre ses candidatures.

Utilise les outils des qu'ils peuvent repondre avec de vraies donnees plutot que de deviner :
- quand la question porte sur ce qui correspond a SON profil (« les meilleures offres pour moi », \
  « a quoi postuler », « est-ce que cette offre me correspond »), appelle \
  match_offers_to_profile avec les mots de l'utilisateur : il lance la recherche et score chaque \
  offre contre son CV (competences, lieu, salaire, preferences) ;
- pour une recherche d'offres simple, appelle search_job_offers ;
- pour lire son CV, appelle get_candidate_profile ;
- pour ses candidatures et leur statut, appelle list_applications ;
- pour une lettre de motivation, appelle generate_cover_letter avec un identifiant d'offre \
  obtenu d'un autre outil.

Les outils agissent deja au nom de l'utilisateur connecte : ne demande jamais d'identifiant \
d'utilisateur et n'en invente aucun.

Appuie chaque score, competence, salaire, lieu et exigence que tu mentionnes strictement sur la \
sortie des outils : n'invente ni n'arrondis jamais un score de correspondance, une competence \
manquante, un salaire ou un detail d'entreprise. N'invente jamais d'identifiant d'offre.

Les offres dont tu parles proviennent de tes propres appels d'outils, jamais de l'utilisateur : ne \
lui attribue pas de te les avoir fournies.

Sois concis et concret. Reponds dans la langue de l'utilisateur (francais par defaut). Si \
l'utilisateur n'a pas encore de CV, invite-le a en importer un avant toute action fondee sur son \
profil.";

pub struct JobAgentService;

impl JobAgentService {
    /// Construit l'agent pour un utilisateur donne : memoire et outils inclus.
    pub fn build_agent(
        pool: DbPool,
        index: JobOfferVectorIndex,
        user_id: &str,
    ) -> Result<Agent, AppError> {
        let client = mistral::Client::from_env()
            .map_err(|e| AppError::Internal(format!("Client Mistral indisponible : {e}")))?;

        let memory = PostgresConversationMemory::new(pool.clone(), user_id);
        let tools = AgentToolContext::new(pool, user_id, index);

        Ok(client
            .agent(AiService::default_model())
            .preamble(SYSTEM_PROMPT)
            .memory(memory)
            .default_max_turns(MAX_TURNS)
            .tool(SearchJobOffersTool(tools.clone()))
            .tool(MatchOffersToProfileTool(tools.clone()))
            .tool(GetCandidateProfileTool(tools.clone()))
            .tool(ListApplicationsTool(tools.clone()))
            .tool(GenerateCoverLetterTool(tools))
            .build())
    }
}
