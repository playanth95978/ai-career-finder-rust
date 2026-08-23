//! Normalisations communes a tous les connecteurs, portage de `AbstractAtsConnector`.
//!
//! Rust n'a pas de classe abstraite : ce que la hierarchie Java portait en methodes protegees est
//! ici un jeu de fonctions libres. Les connecteurs restent des types independants, ce qui evite la
//! contrainte `sealed` du Java tout en gardant une seule implementation de chaque normalisation.

use scraper::Html;
use sha2::{Digest, Sha256};

/// Mots qui marquent une offre comme teletravaillable, dans les langues des sources.
const REMOTE_MARKERS: [&str; 5] = [
    "remote",
    "télétravail",
    "teletravail",
    "work from home",
    "anywhere",
];

/// Longueur de l'identifiant stable. Alignee sur le Java (32 caracteres hexadecimaux) pour que les
/// deux backends produisent la meme identite et dedupliquent donc les memes offres.
const STABLE_ID_LEN: usize = 32;

/// Transforme une description HTML en texte brut.
///
/// Un parseur reel plutot qu'un retrait de balises par expression reguliere : les descriptions des
/// boards contiennent des entites (`&amp;`, `&#39;`), des balises non fermees et des commentaires,
/// que seul un parseur tolerant traite correctement.
///
/// Les blocs sont separes par des espaces pour ne pas coller la fin d'un paragraphe au debut du
/// suivant, ce qui creerait des mots inexistants dans le texte vectorise.
pub fn strip_html(html: Option<&str>) -> String {
    let Some(html) = html.map(str::trim).filter(|value| !value.is_empty()) else {
        return String::new();
    };

    // Rien qui ressemble a du balisage : on evite le cout du parseur et on rend la valeur telle
    // quelle. C'est le cas courant des champs `descriptionPlain` deja fournis en texte.
    if !html.contains('<') && !html.contains('&') {
        return html.to_string();
    }

    let document = Html::parse_fragment(html);
    let text = document
        .root_element()
        .text()
        .collect::<Vec<&str>>()
        .join(" ");

    collapse_whitespace(&text)
}

/// Reduit toute suite d'espaces, tabulations et retours a la ligne a un espace simple.
fn collapse_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<&str>>().join(" ")
}

/// Detection heuristique du teletravail pour les sources sans drapeau booleen.
///
/// Les sources qui exposent un drapeau doivent l'utiliser : cette heuristique se declenche aussi
/// sur « no remote work » ou « remote office », donc elle ne remplace pas une donnee explicite.
pub fn detect_remote(fields: &[Option<&str>]) -> bool {
    let haystack = fields
        .iter()
        .flatten()
        .copied()
        .collect::<Vec<&str>>()
        .join(" ")
        .to_lowercase();

    REMOTE_MARKERS
        .iter()
        .any(|marker| haystack.contains(marker))
}

/// Filtrage par mots-cles cote client, pour les flux sans parametre de requete.
///
/// **Tous** les mots doivent apparaitre, et la recherche porte sur l'ensemble des champs fournis.
/// Chercher la requete entiere comme sous-chaine d'un seul champ ne remonterait rien des que
/// l'utilisateur saisit deux mots (« java spring »), puisqu'aucun intitule ne contient
/// litteralement cette suite.
pub fn matches_all_keywords(keywords: Option<&str>, fields: &[Option<&str>]) -> bool {
    let Some(keywords) = keywords.map(str::trim).filter(|value| !value.is_empty()) else {
        return true;
    };

    let haystack = fields
        .iter()
        .flatten()
        .copied()
        .collect::<Vec<&str>>()
        .join(" ")
        .to_lowercase();

    keywords
        .to_lowercase()
        .split_whitespace()
        .all(|word| haystack.contains(word))
}

/// Identite stable pour les sources sans identifiant — ou avec un identifiant instable, comme une
/// redirection signee qui change a chaque appel.
///
/// Hacher les invariants de l'annonce garde la deduplication fonctionnelle entre deux ingestions,
/// puisqu'elle porte sur `(source, source_id)`.
pub fn stable_id(parts: &[Option<&str>]) -> String {
    let joined = parts
        .iter()
        .map(|part| part.unwrap_or_default())
        .collect::<Vec<&str>>()
        .join("|");

    let digest = Sha256::digest(joined.as_bytes());
    hex::encode(digest)[..STABLE_ID_LEN].to_string()
}

/// Segment de chemin suivant `marker`, chaine de requete retiree (slug de board dans une URL).
pub fn slug_after(url: Option<&str>, marker: &str) -> String {
    let slug = super::ats_connector::board_slug(url, marker);
    slug.split('?').next().unwrap_or_default().to_string()
}

/// Valeur d'en-tete HTTP Basic. Un mot de passe vide est ce qu'attendent les API a cle seule.
pub fn basic_auth(user: &str, password: &str) -> String {
    use base64::Engine;
    let token = base64::engine::general_purpose::STANDARD
        .encode(format!("{user}:{password}").as_bytes());
    format!("Basic {token}")
}

/// Premiere valeur non blanche parmi celles fournies.
pub fn first_non_blank(candidates: &[Option<&str>]) -> Option<String> {
    candidates
        .iter()
        .flatten()
        .map(|value| value.trim())
        .find(|value| !value.is_empty())
        .map(str::to_owned)
}

/// Assemble les composantes non blanches d'un lieu (« Noumea, Nouvelle-Caledonie »).
pub fn join_location(parts: &[Option<&str>]) -> Option<String> {
    let joined = parts
        .iter()
        .flatten()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .collect::<Vec<&str>>()
        .join(", ");

    (!joined.is_empty()).then_some(joined)
}

/// Entier strictement positif, sinon `None`.
///
/// Les boards publient volontiers `0` pour « non renseigne » : le conserver ferait apparaitre des
/// offres a zero euro et fausserait le score de salaire du matching.
pub fn positive_or_none(value: Option<i32>) -> Option<i32> {
    value.filter(|amount| *amount > 0)
}

/// Serialise une liste de competences vers la colonne TEXT attendue par le reste du pipeline.
pub fn skills_json(tags: Option<&Vec<String>>) -> Option<String> {
    let tags = tags?;
    let cleaned: Vec<&String> = tags.iter().filter(|tag| !tag.trim().is_empty()).collect();
    if cleaned.is_empty() {
        return None;
    }
    serde_json::to_string(&cleaned).ok()
}

// ---------------------------------------------------------------------------
// HTTP
// ---------------------------------------------------------------------------

/// Delai maximum par appel a une source. Sans plafond, un board qui ne repond plus bloquerait la
/// recherche entiere le temps du timeout TCP par defaut.
const HTTP_TIMEOUT_SECS: u64 = 15;

/// GET puis decodage JSON.
///
/// Renvoie `None` en cas d'echec reseau, de statut non-2xx ou de JSON illisible, apres
/// journalisation. C'est le contrat du Java : les connecteurs sont interroges en boucle, et une
/// source indisponible doit degrader en « aucun resultat » plutot que faire echouer la recherche.
pub async fn get_json<T: serde::de::DeserializeOwned>(
    client: &reqwest::Client,
    source: &str,
    url: &str,
    headers: &[(&str, &str)],
) -> Option<T> {
    let mut request = client
        .get(url)
        .timeout(std::time::Duration::from_secs(HTTP_TIMEOUT_SECS));
    for (name, value) in headers {
        request = request.header(*name, *value);
    }

    let response = match request.send().await {
        Ok(response) => response,
        Err(e) => {
            tracing::warn!(source, url, error = %e, "Appel a la source echoue");
            return None;
        }
    };

    let status = response.status();
    if !status.is_success() {
        tracing::warn!(source, url, %status, "La source a refuse l'appel");
        return None;
    }

    // Le corps est lu en texte puis decode, pour pouvoir journaliser un extrait quand le JSON ne
    // correspond pas : `reqwest::json()` ne dirait que « expected value at line 1 ».
    let body = match response.text().await {
        Ok(body) => body,
        Err(e) => {
            tracing::warn!(source, url, error = %e, "Corps de reponse illisible");
            return None;
        }
    };

    match serde_json::from_str::<T>(&body) {
        Ok(payload) => Some(payload),
        Err(e) => {
            let excerpt: String = body.chars().take(200).collect();
            tracing::warn!(source, url, error = %e, excerpt, "Charge utile inattendue");
            None
        }
    }
}

/// GET renvoyant le corps en texte, pour les sources sans API JSON (pages HTML a analyser).
pub async fn get_text(
    client: &reqwest::Client,
    source: &str,
    url: &str,
    headers: &[(&str, &str)],
) -> Option<String> {
    let mut request = client
        .get(url)
        .timeout(std::time::Duration::from_secs(HTTP_TIMEOUT_SECS));
    for (name, value) in headers {
        request = request.header(*name, *value);
    }

    match request.send().await {
        Ok(response) if response.status().is_success() => response.text().await.ok(),
        Ok(response) => {
            tracing::warn!(source, url, status = %response.status(), "La source a refuse l'appel");
            None
        }
        Err(e) => {
            tracing::warn!(source, url, error = %e, "Appel a la source echoue");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_html_flattens_markup_and_decodes_entities() {
        assert_eq!(
            strip_html(Some("<p>Bonjour <b>le</b> monde</p>")),
            "Bonjour le monde"
        );
        // Les entites doivent etre decodees : un `&amp;` litteral polluerait le texte vectorise.
        assert_eq!(strip_html(Some("R&amp;D chez ACME")), "R&D chez ACME");
        assert_eq!(strip_html(Some("l&#39;equipe")), "l'equipe");
    }

    #[test]
    fn strip_html_separates_blocks_rather_than_gluing_them() {
        // Sans separateur, « missionProfil » deviendrait un mot inexistant dans l'embedding.
        let text = strip_html(Some("<p>Mission</p><p>Profil</p>"));
        assert_eq!(text, "Mission Profil");
    }

    #[test]
    fn strip_html_survives_unclosed_tags_and_comments() {
        // Cas reel des annonces scrapees : un retrait par regex laisserait passer le commentaire.
        let text = strip_html(Some("<div><p>Poste<!-- note interne --><span>ouvert"));
        assert!(text.contains("Poste"));
        assert!(text.contains("ouvert"));
        assert!(!text.contains("note interne"), "obtenu : {text}");
    }

    #[test]
    fn strip_html_passes_plain_text_through_untouched() {
        assert_eq!(strip_html(Some("Texte deja plat")), "Texte deja plat");
        assert_eq!(strip_html(None), "");
        assert_eq!(strip_html(Some("   ")), "");
    }

    #[test]
    fn detect_remote_reads_markers_in_both_languages() {
        assert!(detect_remote(&[Some("Full remote")]));
        assert!(detect_remote(&[Some("Poste en télétravail")]));
        assert!(detect_remote(&[None, Some("Work From Home")]));
        assert!(!detect_remote(&[Some("Sur site a Noumea")]));
        assert!(!detect_remote(&[]));
    }

    #[test]
    fn matches_all_keywords_requires_every_word_across_all_fields() {
        let fields = [Some("Developpeur Java"), Some("ACME"), Some("Spring Boot")];
        // Les deux mots sont presents, mais dans des champs differents : c'est le cas que le
        // Java documente comme casse par une recherche de sous-chaine unique.
        assert!(matches_all_keywords(Some("java spring"), &fields));
        assert!(matches_all_keywords(Some("JAVA"), &fields));
        assert!(!matches_all_keywords(Some("java cobol"), &fields));
    }

    #[test]
    fn matches_all_keywords_accepts_everything_without_a_query() {
        assert!(matches_all_keywords(None, &[Some("quoi que ce soit")]));
        assert!(matches_all_keywords(Some("   "), &[Some("quoi que ce soit")]));
    }

    #[test]
    fn stable_id_is_deterministic_and_sized_like_the_java_one() {
        let parts = [Some("Dev"), Some("ACME"), Some("Noumea")];
        let first = stable_id(&parts);
        assert_eq!(first.len(), STABLE_ID_LEN);
        assert_eq!(first, stable_id(&parts), "le hash doit etre reproductible");
        // Un champ different donne une identite differente, sinon deux offres fusionneraient.
        assert_ne!(first, stable_id(&[Some("Dev"), Some("ACME"), Some("Paris")]));
    }

    #[test]
    fn stable_id_treats_absent_and_empty_parts_consistently() {
        // Une source qui passe de `null` a `""` sur un champ ne doit pas re-creer toutes ses
        // offres a l'ingestion suivante.
        assert_eq!(
            stable_id(&[Some("Dev"), None, Some("Noumea")]),
            stable_id(&[Some("Dev"), Some(""), Some("Noumea")])
        );
    }

    #[test]
    fn slug_after_extracts_the_board_and_drops_the_query_string() {
        assert_eq!(
            slug_after(Some("https://boards.greenhouse.io/acme/jobs/42"), "greenhouse.io/"),
            "acme"
        );
        assert_eq!(
            slug_after(Some("https://jobs.lever.co/acme?utm=x"), "lever.co/"),
            "acme"
        );
    }

    #[test]
    fn basic_auth_encodes_an_empty_password() {
        // Les API a cle seule attendent « cle: » : omettre le deux-points renverrait un 401.
        assert_eq!(basic_auth("cle", ""), "Basic Y2xlOg==");
    }

    #[test]
    fn join_location_skips_absent_parts() {
        assert_eq!(
            join_location(&[Some("Noumea"), Some("Nouvelle-Caledonie")]).as_deref(),
            Some("Noumea, Nouvelle-Caledonie")
        );
        assert_eq!(join_location(&[Some("Noumea"), None]).as_deref(), Some("Noumea"));
        assert_eq!(join_location(&[None, Some("  ")]), None);
    }

    #[test]
    fn positive_or_none_rejects_zero_and_negatives() {
        // `0` est le « non renseigne » de plusieurs boards : le garder fausserait le score salaire.
        assert_eq!(positive_or_none(Some(0)), None);
        assert_eq!(positive_or_none(Some(-5)), None);
        assert_eq!(positive_or_none(Some(42)), Some(42));
        assert_eq!(positive_or_none(None), None);
    }

    #[test]
    fn skills_json_produces_the_array_the_pipeline_expects() {
        let tags = vec!["rust".to_string(), "  ".to_string(), "axum".to_string()];
        assert_eq!(skills_json(Some(&tags)).as_deref(), Some(r#"["rust","axum"]"#));
        assert_eq!(skills_json(None), None);
        assert_eq!(skills_json(Some(&vec![])), None);
    }

    #[test]
    fn first_non_blank_picks_the_first_usable_value() {
        assert_eq!(
            first_non_blank(&[None, Some("  "), Some("utile"), Some("ignore")]).as_deref(),
            Some("utile")
        );
        assert_eq!(first_non_blank(&[None, Some("   ")]), None);
    }
}
