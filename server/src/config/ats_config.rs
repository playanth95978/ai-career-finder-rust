//! Configuration des connecteurs de job boards, lue depuis l'environnement.
//!
//! Les noms de variables sont **identiques** a ceux de l'application Spring (`ADZUNA_APP_ID`,
//! `CAREERJET_API_KEY`, `FRANCE_TRAVAIL_CLIENT_ID`…), et les valeurs par defaut sont celles de son
//! `application.yml`. Un meme `.env` alimente donc les deux backends, ce qui est le seul moyen
//! d'obtenir des corpus comparables entre eux.

use std::time::Duration;

/// Requetes d'amorçage Adzuna, valeur par defaut du `application.yml` Spring.
const DEFAULT_ADZUNA_QUERIES: &str = "développeur,architecte,data engineer,devops,software engineer,cloud,cybersécurité,product manager";

/// Configuration complete des sources d'offres.
#[derive(Debug, Clone)]
pub struct AtsConfig {
    pub adzuna: AdzunaConfig,
    pub careerjet: CareerjetConfig,
    pub france_travail: FranceTravailConfig,
    pub jobicy: JobicyConfig,
    pub workable_accounts: Vec<String>,
    pub ashby_boards: Vec<String>,
    pub recruitee_companies: Vec<String>,
    /// Seek est protege par une mitigation anti-bot (Kasada) qui renvoie 403 au scraping
    /// serveur : desactive par defaut, exactement comme cote Spring.
    pub seek_enabled: bool,
}

#[derive(Debug, Clone)]
pub struct AdzunaConfig {
    pub app_id: Option<String>,
    pub app_key: Option<String>,
    /// Marche Adzuna (fr, gb, us, de, au…). Pas d'endpoint Nouvelle-Caledonie : emploi.nc couvre
    /// le local.
    pub country: String,
    pub where_filter: Option<String>,
    pub results_per_page: i64,
    pub max_pages: i64,
    pub queries: Vec<String>,
}

impl AdzunaConfig {
    /// Les deux identifiants sont necessaires : avec un seul, l'API repond 401 a chaque appel.
    pub fn is_configured(&self) -> bool {
        self.app_id.is_some() && self.app_key.is_some()
    }
}

#[derive(Debug, Clone)]
pub struct CareerjetConfig {
    pub api_key: Option<String>,
    /// Careerjet partitionne son index par locale : fr_FR, en_AU, en_GB, en_US…
    pub locale_code: String,
    /// `user_ip` est obligatoire cote API. En ingestion planifiee il n'y a pas d'utilisateur
    /// final : renseigner l'IP publique du serveur.
    pub user_ip: Option<String>,
    pub user_agent: String,
    /// En-tete `Referer` exige par l'API (403 « Undeclared referrer » sinon) ; il doit correspondre
    /// au site declare sur le compte partenaire.
    pub referer: Option<String>,
    pub page_size: i64,
    pub max_pages: i64,
}

impl CareerjetConfig {
    /// Les trois valeurs sont exigees par l'API. En annoncer le connecteur sans elles produirait un
    /// 403 par requete au lieu d'une source proprement inactive.
    pub fn is_configured(&self) -> bool {
        self.api_key.is_some() && self.user_ip.is_some() && self.referer.is_some()
    }
}

#[derive(Debug, Clone)]
pub struct FranceTravailConfig {
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    /// Codes departement INSEE croises avec les partitions ROME. Vide = partitions nationales.
    pub departements: Vec<String>,
}

impl FranceTravailConfig {
    pub fn is_configured(&self) -> bool {
        self.client_id.is_some() && self.client_secret.is_some()
    }
}

#[derive(Debug, Clone)]
pub struct JobicyConfig {
    pub count: i64,
    /// Les conditions de Jobicy plafonnent l'appel a un par heure : le flux entier est mis en
    /// cache et filtre en memoire, quel que soit le trafic de recherche.
    pub cache_ttl: Duration,
}

impl AtsConfig {
    pub fn from_env() -> Self {
        Self {
            adzuna: AdzunaConfig {
                app_id: env_opt("ADZUNA_APP_ID"),
                app_key: env_opt("ADZUNA_APP_KEY"),
                country: env_or("ADZUNA_COUNTRY", "fr"),
                where_filter: env_opt("ADZUNA_WHERE"),
                results_per_page: env_num("ADZUNA_RESULTS_PER_PAGE", 50),
                max_pages: env_num("ADZUNA_MAX_PAGES", 5),
                queries: env_csv("ADZUNA_QUERIES", DEFAULT_ADZUNA_QUERIES),
            },
            careerjet: CareerjetConfig {
                api_key: env_opt("CAREERJET_API_KEY"),
                locale_code: env_or("CAREERJET_LOCALE", "fr_FR"),
                user_ip: env_opt("CAREERJET_USER_IP"),
                user_agent: env_or(
                    "CAREERJET_USER_AGENT",
                    "Mozilla/5.0 (compatible; JobSearchAI/1.0)",
                ),
                referer: env_opt("CAREERJET_REFERER"),
                page_size: env_num("CAREERJET_PAGE_SIZE", 50),
                max_pages: env_num("CAREERJET_MAX_PAGES", 5),
            },
            france_travail: FranceTravailConfig {
                client_id: env_opt("FRANCE_TRAVAIL_CLIENT_ID"),
                client_secret: env_opt("FRANCE_TRAVAIL_CLIENT_SECRET"),
                departements: env_csv("FRANCE_TRAVAIL_DEPARTEMENTS", ""),
            },
            jobicy: JobicyConfig {
                count: env_num("JOBICY_COUNT", 100),
                cache_ttl: env_duration("JOBICY_CACHE_TTL", Duration::from_secs(3600)),
            },
            workable_accounts: env_csv("WORKABLE_ACCOUNTS", ""),
            ashby_boards: env_csv("ASHBY_BOARDS", ""),
            recruitee_companies: env_csv("RECRUITEE_COMPANIES", ""),
            seek_enabled: env_bool("SEEK_ENABLED", false),
        }
    }
}

impl Default for AtsConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

/// Variable presente et non blanche, sinon `None`.
///
/// Une variable definie a la chaine vide vaut « absente » : c'est ainsi que le `application.yml`
/// Spring exprime « connecteur inactif » (`${ADZUNA_APP_ID:}`).
fn env_opt(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
            .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn env_or(name: &str, default: &str) -> String {
    env_opt(name).unwrap_or_else(|| default.to_string())
}

/// Entier positif, sinon la valeur par defaut. Une valeur illisible ou nulle est ignoree plutot
/// que d'annuler la pagination en silence.
fn env_num(name: &str, default: i64) -> i64 {
    env_opt(name)
        .and_then(|value| value.parse::<i64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(default)
}

fn env_bool(name: &str, default: bool) -> bool {
    env_opt(name)
        .map(|value| matches!(value.to_ascii_lowercase().as_str(), "true" | "1" | "yes"))
        .unwrap_or(default)
}

/// Liste separee par des virgules, comme les valeurs CSV du `application.yml`.
fn env_csv(name: &str, default: &str) -> Vec<String> {
    csv(&env_or(name, default))
}

/// Duree au format Spring (`1h`, `30m`, `45s`) ou en secondes nues.
///
/// Le `application.yml` ecrit `1h` : accepter uniquement des secondes obligerait a diverger des
/// valeurs Spring, donc a maintenir deux configurations.
fn env_duration(name: &str, default: Duration) -> Duration {
    let Some(raw) = env_opt(name) else {
        return default;
    };
    let raw = raw.to_ascii_lowercase();
    let (digits, multiplier) = match raw.chars().last() {
        Some('h') => (&raw[..raw.len() - 1], 3600),
        Some('m') => (&raw[..raw.len() - 1], 60),
        Some('s') => (&raw[..raw.len() - 1], 1),
        _ => (raw.as_str(), 1),
    };
    digits
        .trim()
        .parse::<u64>()
        .ok()
        .filter(|value| *value > 0)
        .map(|value| Duration::from_secs(value * multiplier))
        .unwrap_or(default)
}

/// Decoupe une valeur de configuration separee par des virgules (slugs de boards, requetes…).
pub fn csv(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(|part| part.trim().to_string())
        .filter(|part| !part.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_trims_and_drops_empty_entries() {
        assert_eq!(
            csv(" a , b ,, c,"),
            vec!["a".to_string(), "b".to_string(), "c".to_string()]
        );
        assert!(csv("").is_empty());
        assert!(csv("  ,  ").is_empty());
    }

    #[test]
    fn csv_preserves_accented_default_queries() {
        // Les requetes Adzuna par defaut contiennent des accents : les perdre changerait le corpus
        // ingere par rapport a la version Spring.
        let queries = csv(DEFAULT_ADZUNA_QUERIES);
        assert!(queries.contains(&"développeur".to_string()));
        assert!(queries.contains(&"cybersécurité".to_string()));
        assert_eq!(queries.len(), 8);
    }

    #[test]
    fn env_duration_reads_the_spring_shorthand() {
        // Fonction interne testee via son parseur : `1h` est ce qu'ecrit le application.yml.
        for (input, expected_secs) in [("1h", 3600), ("30m", 1800), ("45s", 45), ("90", 90)] {
            let parsed = parse_duration_for_test(input);
            assert_eq!(
                parsed,
                Some(Duration::from_secs(expected_secs)),
                "entree {input}"
            );
        }
        // Une valeur illisible ou nulle doit etre rejetee, pas silencieusement ramenee a zero :
        // un TTL de cache nul rappellerait l'API a chaque requete, ce que Jobicy interdit.
        for input in ["", "abc", "0h", "-1"] {
            assert_eq!(parse_duration_for_test(input), None, "entree {input}");
        }
    }

    /// Reprend la logique de [`env_duration`] sans passer par l'environnement, pour ne pas rendre
    /// les tests dependants de leur ordre d'execution.
    fn parse_duration_for_test(raw: &str) -> Option<Duration> {
        let raw = raw.trim().to_ascii_lowercase();
        if raw.is_empty() {
            return None;
        }
        let (digits, multiplier) = match raw.chars().last() {
            Some('h') => (&raw[..raw.len() - 1], 3600),
            Some('m') => (&raw[..raw.len() - 1], 60),
            Some('s') => (&raw[..raw.len() - 1], 1),
            _ => (raw.as_str(), 1),
        };
        digits
            .trim()
            .parse::<u64>()
            .ok()
            .filter(|value| *value > 0)
            .map(|value| Duration::from_secs(value * multiplier))
    }

    #[test]
    fn adzuna_needs_both_credentials() {
        let mut config = AdzunaConfig {
            app_id: Some("id".into()),
            app_key: None,
            country: "fr".into(),
            where_filter: None,
            results_per_page: 50,
            max_pages: 5,
            queries: vec![],
        };
        assert!(!config.is_configured(), "une cle seule ne suffit pas");
        config.app_key = Some("key".into());
        assert!(config.is_configured());
    }

    #[test]
    fn careerjet_needs_key_ip_and_referer() {
        let mut config = CareerjetConfig {
            api_key: Some("key".into()),
            locale_code: "fr_FR".into(),
            user_ip: None,
            user_agent: "ua".into(),
            referer: None,
            page_size: 50,
            max_pages: 5,
        };
        // L'API exige les trois : sans IP ni Referer elle repond 403 a chaque appel.
        assert!(!config.is_configured());
        config.user_ip = Some("10.0.0.1".into());
        assert!(!config.is_configured());
        config.referer = Some("https://example.com".into());
        assert!(config.is_configured());
    }
}
