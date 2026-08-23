//! Comparaison de lieux, pour reordonner les resultats d'une recherche live.
//!
//! # Ecart assume avec l'application Spring, et ce qu'il change
//!
//! Le `GeoLocationService` Java resout un lieu en texte libre vers une structure
//! (ville / region / pays / remote) **via un agent LLM**. Il connait donc les agglomerations et les
//! regions, ce qui lui permet d'affirmer « Paris » et « Noumea » sont dans des pays differents, et
//! donc d'**ecarter** une offre.
//!
//! Ici la comparaison est textuelle. Elle sait dire « ces deux libelles se ressemblent », jamais
//! « ces deux lieux sont dans des pays differents » : « Paris » et « Ile-de-France » n'ont aucun
//! mot commun tout en designant le meme endroit. Conclusion de conception :
//!
//!  - **reordonner** par proximite textuelle est sans risque, et c'est fait ;
//!  - **ecarter** sur une divergence textuelle ne l'est pas, et n'est donc pas fait — cela
//!    supprimerait des offres valides.
//!
//! L'exclusion n'a lieu que sur une preuve structuree : le code pays porte par l'offre
//! (`job_offer.country`, renseigne par plusieurs connecteurs) compare au pays explicitement demande
//! par l'appelant. Deux donnees factuelles, aucune deduction.

/// Niveau de correspondance entre un lieu recherche et le lieu d'une offre.
///
/// L'ordre des variantes suit la confiance decroissante ; il sert au tri.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeoMatchLevel {
    /// Libelles identiques apres normalisation.
    Exact,
    /// L'un des deux libelles contient l'autre (« Noumea » dans « Noumea, Nouvelle-Caledonie »).
    Contains,
    /// Au moins un mot significatif en commun.
    SharedToken,
    /// Offre en teletravail : le lieu ne la disqualifie pas.
    Remote,
    /// Lieu absent de l'offre : on ne sait pas, ce n'est pas un motif de deconsideration.
    Unknown,
    /// Aucun rapprochement textuel possible. **Ne vaut pas preuve d'un autre pays.**
    NoTextualMatch,
}

impl GeoMatchLevel {
    /// Poids utilise pour le tri, de 1.0 (identique) a 0.15 (aucun rapprochement).
    ///
    /// « Unknown » est place au-dessus de « NoTextualMatch » : une offre sans lieu reste plus
    /// plausible qu'une offre situee ailleurs.
    pub fn score(self) -> f64 {
        match self {
            Self::Exact => 1.0,
            Self::Contains => 0.8,
            Self::SharedToken => 0.6,
            Self::Remote => 0.85,
            Self::Unknown => 0.4,
            Self::NoTextualMatch => 0.15,
        }
    }
}

/// Mots trop courts ou trop generiques pour porter un signal geographique.
///
/// Sans ce filtre, « Saint-Louis » et « Saint-Denis » partageraient un mot et passeraient pour
/// proches, et « de » rapprocherait a peu pres n'importe quoi.
const STOP_TOKENS: [&str; 8] = ["de", "du", "des", "la", "le", "les", "saint", "sainte"];

pub struct GeoService;

impl GeoService {
    /// Compare un lieu recherche au lieu d'une offre.
    ///
    /// `remote` a la priorite sur le libelle : une offre en teletravail est tenable ou que soit le
    /// candidat, c'est tout l'interet du teletravail.
    pub fn match_level(
        search_location: &str,
        offer_location: Option<&str>,
        remote: bool,
    ) -> GeoMatchLevel {
        let reference = normalize(search_location);
        if reference.is_empty() {
            return GeoMatchLevel::Unknown;
        }

        let Some(offer) = offer_location.map(normalize).filter(|v| !v.is_empty()) else {
            // Pas de lieu : le teletravail reste un signal, sinon on ne sait pas.
            return if remote {
                GeoMatchLevel::Remote
            } else {
                GeoMatchLevel::Unknown
            };
        };

        if offer == reference {
            return GeoMatchLevel::Exact;
        }
        if offer.contains(&reference) || reference.contains(&offer) {
            return GeoMatchLevel::Contains;
        }
        if shares_significant_token(&reference, &offer) {
            return GeoMatchLevel::SharedToken;
        }
        // Le teletravail rattrape un lieu qui ne correspond pas — mais seulement apres avoir
        // cherche une correspondance, pour ne pas masquer une offre reellement sur place.
        if remote {
            return GeoMatchLevel::Remote;
        }
        GeoMatchLevel::NoTextualMatch
    }

    /// Vrai si l'offre est certainement dans un autre pays que celui demande.
    ///
    /// Exige les deux faits : un pays demande par l'appelant, et un pays renseigne sur l'offre.
    /// Sans les deux, on ne conclut pas — c'est ce qui distingue cette exclusion d'une deduction
    /// hasardeuse a partir de libelles.
    ///
    /// # Pourquoi une table d'equivalences et pas une comparaison de sous-chaines
    ///
    /// Le front envoie un code marche (`fr`, `nc`, `us`), les connecteurs stockent un nom complet
    /// (« France », « Nouvelle-Caledonie », « United States »). Une tolerance par inclusion marche
    /// par accident pour `fr` dans `france`, et **echoue en s'inversant** pour `nc` : « nc » n'est
    /// pas une sous-chaine de « nouvelle caledonie », donc `country=NC` ecartait precisement les
    /// offres caledoniennes qu'il devait garder. Mesure faite avant correction : 23 offres NC
    /// supprimees, 100 offres au pays inconnu conservees.
    ///
    /// D'ou une table explicite. Un code absent de la table ne permet **aucune** conclusion : mieux
    /// vaut ne pas filtrer que filtrer a l'envers.
    pub fn is_confident_country_mismatch(
        requested_country: Option<&str>,
        offer_country: Option<&str>,
    ) -> bool {
        let Some(requested) = requested_country.map(normalize).filter(|v| !v.is_empty()) else {
            return false;
        };
        let Some(offer) = offer_country.map(normalize).filter(|v| !v.is_empty()) else {
            return false;
        };

        if offer == requested {
            return false;
        }

        // Les deux valeurs sont resolues vers le meme jeu d'alias : peu importe laquelle porte le
        // code et laquelle porte le nom.
        match (country_aliases(&requested), country_aliases(&offer)) {
            // Les deux pays sont connus : la comparaison est fiable dans les deux sens.
            (Some(wanted), Some(actual)) => wanted != actual,
            // Le pays demande est connu, celui de l'offre non : on ne peut pas affirmer qu'ils
            // diffèrent, un nom non reference n'etant pas la preuve d'un autre pays.
            _ => false,
        }
    }
}

/// Pays reconnus, sous leurs formes rencontrees dans les donnees.
///
/// La premiere entree de chaque ligne est le code marche envoye par le front ; les suivantes sont
/// les libelles que les connecteurs ecrivent reellement en base (releves sur le corpus). La liste
/// couvre les marches Adzuna et Careerjet plus les pays effectivement presents ; l'etendre est un
/// ajout de ligne.
const COUNTRY_ALIASES: [&[&str]; 14] = [
    &["nc", "nouvelle caledonie", "new caledonia"],
    &["fr", "france"],
    &["us", "usa", "united states", "united states of america"],
    &["gb", "uk", "united kingdom", "great britain"],
    &["au", "australia"],
    &["de", "germany", "deutschland"],
    &["ca", "canada"],
    &["ie", "ireland"],
    &["sg", "singapore"],
    &["jp", "japan"],
    &["in", "india"],
    &["kr", "south korea", "korea republic of"],
    &["nz", "new zealand"],
    &["es", "spain", "espana"],
];

/// Jeu d'alias auquel appartient une valeur normalisee, ou `None` si le pays n'est pas reference.
///
/// L'identite du groupe est son premier element (le code marche), ce qui permet de comparer deux
/// valeurs de formes differentes.
fn country_aliases(value: &str) -> Option<&'static str> {
    COUNTRY_ALIASES
        .iter()
        .find(|aliases| aliases.contains(&value))
        .map(|aliases| aliases[0])
}

/// Minuscules, accents replies, ponctuation reduite a des espaces.
fn normalize(value: &str) -> String {
    let folded: String = value
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| match c {
            'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' => 'a',
            'ç' => 'c',
            'è' | 'é' | 'ê' | 'ë' => 'e',
            'ì' | 'í' | 'î' | 'ï' => 'i',
            'ñ' => 'n',
            'ò' | 'ó' | 'ô' | 'õ' | 'ö' => 'o',
            'ù' | 'ú' | 'û' | 'ü' => 'u',
            other if other.is_alphanumeric() => other,
            _ => ' ',
        })
        .collect();

    folded.split_whitespace().collect::<Vec<&str>>().join(" ")
}

fn shares_significant_token(reference: &str, offer: &str) -> bool {
    let reference_tokens: Vec<&str> = significant_tokens(reference);
    if reference_tokens.is_empty() {
        return false;
    }
    let offer_tokens = significant_tokens(offer);
    reference_tokens
        .iter()
        .any(|token| offer_tokens.contains(token))
}

fn significant_tokens(value: &str) -> Vec<&str> {
    value
        .split_whitespace()
        // Deux caracteres au minimum, et pas de mot de liaison : « de » ou « la » rapprocheraient
        // n'importe quels deux libelles.
        .filter(|token| token.chars().count() > 2 && !STOP_TOKENS.contains(token))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_labels_match_exactly_despite_accents_and_case() {
        assert_eq!(
            GeoService::match_level("Nouméa", Some("NOUMEA"), false),
            GeoMatchLevel::Exact
        );
    }

    #[test]
    fn a_label_contained_in_the_other_is_close() {
        assert_eq!(
            GeoService::match_level("Noumea", Some("Noumea, Nouvelle-Caledonie"), false),
            GeoMatchLevel::Contains
        );
        assert_eq!(
            GeoService::match_level("Paris, France", Some("Paris"), false),
            GeoMatchLevel::Contains
        );
    }

    #[test]
    fn a_shared_significant_token_is_a_weaker_signal() {
        // Ni l'un ni l'autre ne contient l'autre, mais « noumea » leur est commun.
        assert_eq!(
            GeoService::match_level("Noumea Sud", Some("Dumbea Nord, Noumea"), false),
            GeoMatchLevel::SharedToken
        );
        // Quand un libelle contient l'autre, `Contains` est plus informatif et doit primer.
        assert_eq!(
            GeoService::match_level("Noumea", Some("Dumbea, Grand Noumea"), false),
            GeoMatchLevel::Contains
        );
    }

    #[test]
    fn linking_words_do_not_create_false_proximity() {
        // Sans le filtre de mots vides, « de » suffirait a rapprocher ces deux lieux.
        assert_eq!(
            GeoService::match_level("Ile de France", Some("Cote de Nacre"), false),
            GeoMatchLevel::NoTextualMatch
        );
        // « saint » est trop repandu dans les toponymes pour signifier quoi que ce soit.
        assert_eq!(
            GeoService::match_level("Saint-Louis", Some("Saint-Denis"), false),
            GeoMatchLevel::NoTextualMatch
        );
    }

    #[test]
    fn remote_offers_are_never_penalised_by_their_location() {
        assert_eq!(
            GeoService::match_level("Noumea", Some("Berlin, Germany"), true),
            GeoMatchLevel::Remote
        );
        assert_eq!(GeoService::match_level("Noumea", None, true), GeoMatchLevel::Remote);
        // Mais le teletravail ne doit pas masquer une correspondance exacte, plus informative.
        assert_eq!(
            GeoService::match_level("Noumea", Some("Noumea"), true),
            GeoMatchLevel::Exact
        );
    }

    #[test]
    fn an_offer_without_a_location_is_unknown_not_rejected() {
        // Ne pas savoir n'est pas une raison de deconsiderer : `Unknown` est mieux classe que
        // `NoTextualMatch`.
        assert_eq!(
            GeoService::match_level("Noumea", None, false),
            GeoMatchLevel::Unknown
        );
        assert!(GeoMatchLevel::Unknown.score() > GeoMatchLevel::NoTextualMatch.score());
    }

    #[test]
    fn scores_are_ordered_by_decreasing_confidence() {
        let ordered = [
            GeoMatchLevel::Exact,
            GeoMatchLevel::Remote,
            GeoMatchLevel::Contains,
            GeoMatchLevel::SharedToken,
            GeoMatchLevel::Unknown,
            GeoMatchLevel::NoTextualMatch,
        ];
        for pair in ordered.windows(2) {
            assert!(
                pair[0].score() > pair[1].score(),
                "{:?} doit passer devant {:?}",
                pair[0],
                pair[1]
            );
        }
    }

    // --- Exclusion par pays ---------------------------------------------------------------

    #[test]
    fn country_mismatch_needs_both_facts_to_be_known() {
        // Aucun pays demande : on ne conclut pas.
        assert!(!GeoService::is_confident_country_mismatch(None, Some("US")));
        // Offre sans pays : on ne conclut pas non plus.
        assert!(!GeoService::is_confident_country_mismatch(Some("fr"), None));
        assert!(!GeoService::is_confident_country_mismatch(Some("  "), Some("US")));
    }

    #[test]
    fn country_mismatch_maps_market_codes_to_the_names_stored_in_the_data() {
        // Formes reellement presentes en base, relevees sur le corpus.
        assert!(!GeoService::is_confident_country_mismatch(Some("fr"), Some("France")));
        assert!(!GeoService::is_confident_country_mismatch(Some("us"), Some("United States")));
        assert!(!GeoService::is_confident_country_mismatch(Some("gb"), Some("United Kingdom")));
        assert!(!GeoService::is_confident_country_mismatch(Some("sg"), Some("Singapore")));
        // Et dans l'autre sens : le nom demande, le code stocke.
        assert!(!GeoService::is_confident_country_mismatch(Some("France"), Some("fr")));
    }

    #[test]
    fn nc_matches_nouvelle_caledonie_the_case_that_was_inverted() {
        // Regression : « nc » n'etant pas une sous-chaine de « nouvelle caledonie », l'ancienne
        // comparaison par inclusion ecartait exactement les offres caledoniennes que
        // `country=NC` devait conserver.
        assert!(!GeoService::is_confident_country_mismatch(
            Some("NC"),
            Some("Nouvelle-Calédonie")
        ));
        assert!(!GeoService::is_confident_country_mismatch(Some("nc"), Some("New Caledonia")));
        // Et le filtre doit bien ecarter les autres.
        assert!(GeoService::is_confident_country_mismatch(
            Some("NC"),
            Some("United States")
        ));
    }

    #[test]
    fn a_genuinely_different_country_is_rejected() {
        assert!(GeoService::is_confident_country_mismatch(Some("fr"), Some("US")));
        assert!(GeoService::is_confident_country_mismatch(Some("France"), Some("Australia")));
        assert!(GeoService::is_confident_country_mismatch(Some("us"), Some("Japan")));
    }

    #[test]
    fn an_unreferenced_country_never_triggers_a_rejection() {
        // Principe de conception : mieux vaut ne pas filtrer que filtrer a l'envers. Un libelle
        // absent de la table n'est pas la preuve d'un autre pays.
        assert!(!GeoService::is_confident_country_mismatch(Some("fr"), Some("Ruritanie")));
        assert!(!GeoService::is_confident_country_mismatch(Some("zz"), Some("France")));
        assert!(!GeoService::is_confident_country_mismatch(Some("zz"), Some("Ruritanie")));
    }

    #[test]
    fn country_alias_groups_are_disjoint() {
        // Un libelle present dans deux groupes rendrait la comparaison non deterministe.
        let mut seen: Vec<&str> = Vec::new();
        for aliases in COUNTRY_ALIASES {
            for alias in aliases {
                assert!(
                    !seen.contains(alias),
                    "'{alias}' apparait dans deux groupes de pays"
                );
                seen.push(alias);
            }
        }
    }

    #[test]
    fn country_aliases_are_stored_normalised() {
        // Les valeurs de la table sont comparees a du texte normalise : une majuscule ou un accent
        // dans la table la rendrait inatteignable, sans erreur visible.
        for aliases in COUNTRY_ALIASES {
            for alias in aliases {
                assert_eq!(
                    &normalize(alias),
                    alias,
                    "'{alias}' n'est pas sous forme normalisee"
                );
            }
        }
    }

    #[test]
    fn a_textual_mismatch_alone_never_rejects() {
        // Le point central de ce module : « Paris » et « Ile-de-France » ne partagent aucun mot,
        // mais designent le meme endroit. Une divergence textuelle ne doit jamais suffire.
        assert_eq!(
            GeoService::match_level("Paris", Some("Ile-de-France"), false),
            GeoMatchLevel::NoTextualMatch
        );
        assert!(!GeoService::is_confident_country_mismatch(None, None));
    }
}
