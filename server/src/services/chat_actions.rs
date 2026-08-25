//! Actions cliquables proposees a la fin d'un tour d'assistant.
//!
//! Quand l'assistant a repondu, le front affiche des boutons contextuels : ouvrir une offre citee,
//! affiner la recherche, importer son CV. Ces boutons sont decides **par le code**, a partir de ce
//! que les outils de l'agent ont reellement produit — jamais par le modele. Un modele qui choisit
//! ses propres boutons invente des identifiants d'offres et des routes inexistantes ; le code, lui,
//! ne peut proposer que ce qui vient de se passer.
//!
//! Voir `docs/plan-actions-chatbot.md` pour le plan d'ensemble.
//!
//! # Trois types, et pourquoi seulement trois
//!
//! Les cas d'usage recenses se ramenent a trois comportements de clic : renvoyer un texte dans la
//! conversation, ouvrir une page, appeler une API. Un quatrieme type par cas d'usage aurait
//! multiplie les branches du template Angular sans rien ajouter.
//!
//! # Le tour est termine
//!
//! Ces actions sont un raccourci pour le tour **suivant**. Elles ne suspendent ni l'agent ni le flux
//! SSE : l'historique de conversation etant en base, le tour suivant retrouve tout le contexte sans
//! qu'aucun etat n'ait besoin d'etre conserve entre les deux. C'est ce qui permet de repliquer le
//! serveur sans session collante.

use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

/// Nombre maximum d'actions attachees a un message.
///
/// Au-dela, le fil de conversation cesse de se lire comme une conversation pour ressembler a un
/// formulaire. La valeur tient exactement le cas le plus charge — trois offres citees, un « voir
/// tout », deux filtres — ce qui est aussi une facon de dire qu'on ne veut pas plus riche que ca.
const MAX_ACTIONS: usize = 6;

/// Nombre d'offres citees individuellement.
///
/// Trois : au-dela, le bouton « voir la liste » fait mieux le travail, et une pile de boutons
/// d'offres pousse la reponse textuelle hors de l'ecran.
const MAX_CITED_OFFERS: usize = 3;

// Routes de l'application Angular.
//
// Rassemblees ici plutot qu'ecrites en place : ce sont des chaines que le front passe a
// `navigateByUrl`, donc une faute de frappe produit une page blanche sans erreur. Les avoir au meme
// endroit rend aussi visible ce que le chatbot est autorise a ouvrir.
const ROUTE_SEARCH: &str = "/job-copilot/search";
const ROUTE_MATCHES: &str = "/job-copilot/matches";
const ROUTE_APPLICATIONS: &str = "/job-copilot/applications";
const ROUTE_ONBOARDING: &str = "/job-copilot/onboarding";
const ROUTE_SETTINGS: &str = "/job-copilot/settings";
const ROUTE_CV_BUILDER: &str = "/job-copilot/cv-builder";

/// Libelle d'une action : une **cle i18n** et ses parametres, jamais une phrase.
///
/// Le backend ne construit aucun texte affichable, par coherence avec le reste de l'application qui
/// passe partout par des cles. Les parametres servent aux libelles qui portent une valeur — « Voir
/// les 12 offres » est la cle `viewAllOffers` avec `{ "count": 12 }`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct ChatLabel {
    /// Cle du bundle Angular, p. ex. `chat.action.viewAllOffers`.
    pub key: String,
    /// Parametres d'interpolation. `None` quand la cle n'en attend pas.
    #[schema(value_type = Option<Object>)]
    pub params: Option<serde_json::Value>,
}

impl ChatLabel {
    fn plain(key: &str) -> Self {
        Self { key: format!("chat.action.{key}"), params: None }
    }

    fn with(key: &str, params: serde_json::Value) -> Self {
        Self { key: format!("chat.action.{key}"), params: Some(params) }
    }
}

/// Operations que le front sait declencher.
///
/// Enum ferme et non chaine libre : c'est ce qui garantit qu'une operation recue correspond a un
/// appel que le front sait faire. Une chaine imposerait une branche par defaut dans le template,
/// donc un bouton qui ne fait rien.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ChatMutation {
    GenerateCoverLetter,
    Apply,
}

/// Une action cliquable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ChatAction {
    /// Relance la conversation avec `text`.
    Prompt { label: ChatLabel, text: String },
    /// Ouvre une route de l'application. Toujours un chemin interne, jamais une URL externe.
    Navigate { label: ChatLabel, route: String },
    /// Appelle une API. `confirm` impose une validation de l'utilisateur avant l'appel.
    Mutate {
        label: ChatLabel,
        operation: ChatMutation,
        #[serde(rename = "offerId", skip_serializing_if = "Option::is_none")]
        offer_id: Option<Uuid>,
        confirm: bool,
    },
}

/// Une offre citee par un outil, reduite a ce qu'un bouton a besoin d'afficher.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CitedOffer {
    pub id: Uuid,
    pub title: String,
}

/// Ce qu'un outil de l'agent a produit pendant le tour.
///
/// Rempli par les outils, lu en fin de tour. C'est la seule source d'information de ce module :
/// aucune action n'est proposee sans un fait correspondant dans cette liste.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolOutcome {
    /// `search_job_offers` a repondu. `total` peut depasser `offers`, qui est tronquee.
    OffersFound { offers: Vec<CitedOffer>, total: usize },
    /// `match_offers_to_profile` a repondu.
    MatchesFound { offers: Vec<CitedOffer>, total: usize },
    /// `get_candidate_profile` n'a trouve aucun profil : l'utilisateur n'a pas importe de CV.
    ProfileMissing,
    /// `list_applications` a repondu.
    ApplicationsListed { total: usize },
    /// `generate_cover_letter` a produit une lettre.
    CoverLetterGenerated { offer_id: Uuid },
}

/// Traduit ce que les outils ont fait en boutons a afficher.
///
/// Fonction pure : ni base, ni modele, ni horloge. C'est ce qui la rend testable exhaustivement, et
/// c'est la raison pour laquelle la decision des boutons vit ici plutot que dans le handler.
///
/// L'ordre de construction est l'ordre d'affichage, et il est deliberé : les actions les plus
/// engageantes d'abord (ouvrir une offre precise, agir sur elle), les invitations a reformuler
/// ensuite. Le plafond [`MAX_ACTIONS`] tronque donc la queue, jamais l'essentiel.
pub fn actions_for(outcomes: &[ToolOutcome]) -> Vec<ChatAction> {
    let mut actions: Vec<ChatAction> = Vec::new();

    for outcome in outcomes {
        match outcome {
            ToolOutcome::OffersFound { offers, total } if *total > 0 => {
                push_cited_offers(&mut actions, offers);
                actions.push(ChatAction::Navigate {
                    label: ChatLabel::with("viewAllOffers", serde_json::json!({ "count": total })),
                    route: ROUTE_SEARCH.to_string(),
                });
                // Les deux filtres les plus demandes. En proposer un par critere de l'offre
                // transformerait le fil en panneau de facettes — c'est le role de la page de
                // recherche, pas du chat.
                actions.push(ChatAction::Prompt {
                    label: ChatLabel::plain("onlyPermanent"),
                    text: "Uniquement les offres en CDI".to_string(),
                });
                actions.push(ChatAction::Prompt {
                    label: ChatLabel::plain("onlyRemote"),
                    text: "Uniquement les offres en teletravail".to_string(),
                });
            }
            // Recherche infructueuse : les deux issues sont d'elargir, ou de corriger des
            // preferences trop etroites. Ne rien proposer laisserait l'utilisateur devant un
            // « aucun resultat » sans porte de sortie.
            ToolOutcome::OffersFound { .. } => {
                actions.push(ChatAction::Prompt {
                    label: ChatLabel::plain("widenSearch"),
                    text: "Elargis la recherche".to_string(),
                });
                actions.push(ChatAction::Navigate {
                    label: ChatLabel::plain("openPreferences"),
                    route: ROUTE_SETTINGS.to_string(),
                });
            }
            ToolOutcome::MatchesFound { offers, total } if *total > 0 => {
                push_cited_offers(&mut actions, offers);
                actions.push(ChatAction::Navigate {
                    label: ChatLabel::with("viewAllMatches", serde_json::json!({ "count": total })),
                    route: ROUTE_MATCHES.to_string(),
                });
            }
            // Aucun match : c'est presque toujours un profil trop pauvre plutot qu'un corpus vide.
            ToolOutcome::MatchesFound { .. } => {
                actions.push(ChatAction::Navigate {
                    label: ChatLabel::plain("completeProfile"),
                    route: ROUTE_CV_BUILDER.to_string(),
                });
            }
            ToolOutcome::ProfileMissing => {
                actions.push(ChatAction::Navigate {
                    label: ChatLabel::plain("importCv"),
                    route: ROUTE_ONBOARDING.to_string(),
                });
            }
            ToolOutcome::ApplicationsListed { total } => {
                actions.push(ChatAction::Navigate {
                    label: ChatLabel::with("viewApplications", serde_json::json!({ "count": total })),
                    route: ROUTE_APPLICATIONS.to_string(),
                });
            }
            ToolOutcome::CoverLetterGenerated { offer_id } => {
                actions.push(ChatAction::Navigate {
                    label: ChatLabel::plain("openCoverLetter"),
                    route: offer_route(*offer_id),
                });
            }
        }
    }

    // Les actions qui modifient quelque chose ne sont proposees que si le tour porte sur **une**
    // offre identifiee. Sur trois offres, « Postuler » serait ambigu : l'utilisateur ne saurait pas
    // laquelle part.
    if let Some(offer) = single_subject_offer(outcomes) {
        actions.push(ChatAction::Mutate {
            label: ChatLabel::plain("generateCoverLetter"),
            operation: ChatMutation::GenerateCoverLetter,
            offer_id: Some(offer),
            // Generer une lettre est reversible : elle est enregistree, pas envoyee.
            confirm: false,
        });
        actions.push(ChatAction::Mutate {
            label: ChatLabel::plain("apply"),
            operation: ChatMutation::Apply,
            offer_id: Some(offer),
            // Postuler est irreversible du point de vue de l'utilisateur : la candidature part.
            confirm: true,
        });
    }

    actions.truncate(MAX_ACTIONS);
    actions
}

fn push_cited_offers(actions: &mut Vec<ChatAction>, offers: &[CitedOffer]) {
    for offer in offers.iter().take(MAX_CITED_OFFERS) {
        actions.push(ChatAction::Navigate {
            // L'intitule est une **donnee**, pas un libelle d'interface : il passe donc en
            // parametre d'une cle, et non comme texte du bouton.
            label: ChatLabel::with(
                "openOffer",
                serde_json::json!({ "title": offer.title.as_str() }),
            ),
            route: offer_route(offer.id),
        });
    }
}

/// Route ouvrant une offre precise.
///
/// L'application n'a **pas** de page de detail d'offre : les routes de `job-copilot` s'arretent aux
/// pages de liste. On passe donc par la page de recherche avec l'identifiant en parametre, a charge
/// pour elle de mettre l'offre en avant. Le jour ou une route dediee existera, seul ce point sera a
/// changer.
fn offer_route(offer_id: Uuid) -> String {
    format!("{ROUTE_SEARCH}?offerId={offer_id}")
}

/// L'offre unique dont parle le tour, s'il n'y en a qu'une.
///
/// Une lettre generee designe son offre sans ambiguite. Une recherche ou un matching ne le font que
/// s'ils n'ont rapporte qu'un seul resultat.
fn single_subject_offer(outcomes: &[ToolOutcome]) -> Option<Uuid> {
    let mut candidates = outcomes.iter().filter_map(|outcome| match outcome {
        ToolOutcome::CoverLetterGenerated { offer_id } => Some(*offer_id),
        ToolOutcome::OffersFound { offers, total } | ToolOutcome::MatchesFound { offers, total }
            if *total == 1 =>
        {
            offers.first().map(|offer| offer.id)
        }
        _ => None,
    });

    let first = candidates.next()?;
    // Deux outils ayant designe deux offres differentes : on ne choisit pas a la place de
    // l'utilisateur.
    candidates.all(|other| other == first).then_some(first)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn offer(title: &str) -> CitedOffer {
        CitedOffer { id: Uuid::new_v4(), title: title.to_string() }
    }

    fn keys(actions: &[ChatAction]) -> Vec<&str> {
        actions
            .iter()
            .map(|action| match action {
                ChatAction::Prompt { label, .. }
                | ChatAction::Navigate { label, .. }
                | ChatAction::Mutate { label, .. } => label.key.as_str(),
            })
            .collect()
    }

    #[test]
    fn sans_outil_appele_aucun_bouton() {
        // Une reponse purement conversationnelle ne doit pas faire apparaitre de boutons : ils
        // suggereraient une action que rien dans le tour ne justifie.
        assert!(actions_for(&[]).is_empty());
    }

    #[test]
    fn une_recherche_fructueuse_propose_les_offres_puis_la_liste_puis_les_filtres() {
        let actions = actions_for(&[ToolOutcome::OffersFound {
            offers: vec![offer("Developpeur"), offer("Devops"), offer("Data")],
            total: 12,
        }]);

        assert_eq!(
            keys(&actions),
            vec![
                "chat.action.openOffer",
                "chat.action.openOffer",
                "chat.action.openOffer",
                "chat.action.viewAllOffers",
                "chat.action.onlyPermanent",
                "chat.action.onlyRemote",
            ]
        );
    }

    #[test]
    fn le_nombre_total_est_passe_en_parametre_pas_dans_le_libelle() {
        let actions = actions_for(&[ToolOutcome::OffersFound {
            offers: vec![offer("Developpeur")],
            total: 12,
        }]);

        let view_all = actions
            .iter()
            .find(|a| matches!(a, ChatAction::Navigate { label, .. } if label.key.ends_with("viewAllOffers")))
            .expect("bouton de liste attendu");
        let ChatAction::Navigate { label, .. } = view_all else { unreachable!() };
        assert_eq!(label.params, Some(serde_json::json!({ "count": 12 })));
    }

    #[test]
    fn les_offres_citees_sont_plafonnees_a_trois() {
        let actions = actions_for(&[ToolOutcome::OffersFound {
            offers: vec![offer("a"), offer("b"), offer("c"), offer("d"), offer("e")],
            total: 5,
        }]);

        let cited = keys(&actions)
            .iter()
            .filter(|key| key.ends_with("openOffer"))
            .count();
        assert_eq!(cited, 3);
    }

    #[test]
    fn une_recherche_vide_offre_une_porte_de_sortie() {
        let actions = actions_for(&[ToolOutcome::OffersFound { offers: vec![], total: 0 }]);

        assert_eq!(
            keys(&actions),
            vec!["chat.action.widenSearch", "chat.action.openPreferences"]
        );
    }

    #[test]
    fn un_profil_absent_renvoie_vers_l_import_de_cv() {
        let actions = actions_for(&[ToolOutcome::ProfileMissing]);

        assert_eq!(keys(&actions), vec!["chat.action.importCv"]);
        let ChatAction::Navigate { route, .. } = &actions[0] else {
            panic!("navigation attendue")
        };
        assert_eq!(route, ROUTE_ONBOARDING);
    }

    #[test]
    fn une_offre_unique_autorise_les_actions_qui_modifient() {
        let target = offer("Developpeur backend");
        let id = target.id;
        let actions = actions_for(&[ToolOutcome::OffersFound {
            offers: vec![target],
            total: 1,
        }]);

        let mutations: Vec<(ChatMutation, bool)> = actions
            .iter()
            .filter_map(|action| match action {
                ChatAction::Mutate { operation, offer_id, confirm, .. } => {
                    assert_eq!(*offer_id, Some(id));
                    Some((*operation, *confirm))
                }
                _ => None,
            })
            .collect();

        assert_eq!(
            mutations,
            vec![
                (ChatMutation::GenerateCoverLetter, false),
                (ChatMutation::Apply, true),
            ]
        );
    }

    #[test]
    fn plusieurs_offres_n_autorisent_aucune_action_qui_modifie() {
        // « Postuler » sur trois offres ne veut rien dire : l'utilisateur ne saurait pas laquelle
        // part chez le recruteur.
        let actions = actions_for(&[ToolOutcome::OffersFound {
            offers: vec![offer("a"), offer("b")],
            total: 2,
        }]);

        assert!(!actions.iter().any(|a| matches!(a, ChatAction::Mutate { .. })));
    }

    #[test]
    fn deux_outils_designant_deux_offres_differentes_n_autorisent_rien() {
        let actions = actions_for(&[
            ToolOutcome::OffersFound { offers: vec![offer("a")], total: 1 },
            ToolOutcome::CoverLetterGenerated { offer_id: Uuid::new_v4() },
        ]);

        assert!(!actions.iter().any(|a| matches!(a, ChatAction::Mutate { .. })));
    }

    #[test]
    fn postuler_exige_toujours_une_confirmation() {
        let target = offer("Developpeur");
        let actions = actions_for(&[ToolOutcome::OffersFound {
            offers: vec![target],
            total: 1,
        }]);

        for action in &actions {
            if let ChatAction::Mutate { operation: ChatMutation::Apply, confirm, .. } = action {
                assert!(*confirm, "postuler est irreversible pour l'utilisateur");
            }
        }
    }

    #[test]
    fn le_nombre_d_actions_est_plafonne() {
        // Deux outils bavards : sans plafond on afficherait une dizaine de boutons.
        let actions = actions_for(&[
            ToolOutcome::OffersFound {
                offers: vec![offer("a"), offer("b"), offer("c")],
                total: 30,
            },
            ToolOutcome::MatchesFound {
                offers: vec![offer("d"), offer("e"), offer("f")],
                total: 30,
            },
        ]);

        assert_eq!(actions.len(), MAX_ACTIONS);
    }

    #[test]
    fn toutes_les_routes_restent_internes() {
        // Le front passe `route` a `navigateByUrl` : une URL absolue y ouvrirait une redirection
        // hors de l'application.
        let actions = actions_for(&[
            ToolOutcome::OffersFound { offers: vec![offer("a")], total: 1 },
            ToolOutcome::ProfileMissing,
            ToolOutcome::ApplicationsListed { total: 3 },
            ToolOutcome::CoverLetterGenerated { offer_id: Uuid::new_v4() },
        ]);

        for action in &actions {
            if let ChatAction::Navigate { route, .. } = action {
                assert!(route.starts_with("/job-copilot/"), "route suspecte : {route}");
                assert!(!route.contains("//"), "route suspecte : {route}");
            }
        }
    }

    #[test]
    fn la_forme_json_porte_le_discriminant_et_camel_case() {
        // Le front discrimine sur `type` et lit `offerId` : c'est le contrat, il doit etre teste.
        let id = Uuid::new_v4();
        let json = serde_json::to_value(ChatAction::Mutate {
            label: ChatLabel::plain("apply"),
            operation: ChatMutation::Apply,
            offer_id: Some(id),
            confirm: true,
        })
        .expect("serialisation");

        assert_eq!(json["type"], "mutate");
        assert_eq!(json["operation"], "APPLY");
        assert_eq!(json["offerId"], id.to_string());
        assert_eq!(json["confirm"], true);
        assert_eq!(json["label"]["key"], "chat.action.apply");
        assert!(json["label"]["params"].is_null());
    }

    #[test]
    fn un_libelle_sans_parametre_n_expose_pas_d_objet_vide() {
        let json = serde_json::to_value(ChatLabel::plain("importCv")).expect("serialisation");
        assert!(json["params"].is_null());
    }
}
