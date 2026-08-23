//! Sources sans API publique : HelloWork et Seek, lues en analysant leur page de recherche.
//!
//! Regroupees a part parce qu'elles ne partagent rien avec les connecteurs a API : pas de contrat
//! de charge utile, donc des selecteurs CSS qui cassent des que le site change son balisage. C'est
//! le compromis assume de la version Java, et la raison pour laquelle les deux tolerent l'absence
//! de resultat sans la signaler comme une panne.
//!
//! Seek est de plus protege par une mitigation anti-bot (Kasada) qui repond 403 au scraping
//! serveur : le connecteur n'est enregistre que si `SEEK_ENABLED=true`, exactement comme le
//! `@ConditionalOnProperty` cote Spring.

use async_trait::async_trait;
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};

use super::ats_connector::{AtsConnector, JobSource};
use super::support::{detect_remote, get_text, stable_id};
use crate::errors::AppError;
use crate::models::JobOffer;

/// Navigateur usurpe. Les deux sites renvoient une page vide, voire un 403, a un client qui
/// s'annonce comme un robot.
const BROWSER_USER_AGENT: &str =
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0 Safari/537.36";

/// Plafond d'annonces extraites d'une page de resultats.
const MAX_CARDS: usize = 50;

lazy_static! {
    /// Identifiant d'annonce Seek : /job/{id}.
    static ref SEEK_JOB_ID: Regex = Regex::new(r"/job/(\d+)").unwrap();
}

/// Premier texte non vide correspondant a l'un des selecteurs, dans l'ordre.
///
/// Plusieurs selecteurs par champ : les sites font cohabiter plusieurs generations de balisage, et
/// un selecteur unique ne remonterait qu'une partie des cartes.
fn first_text(element: &scraper::ElementRef, selectors: &[&str]) -> Option<String> {
    for raw in selectors {
        let Ok(selector) = Selector::parse(raw) else {
            continue;
        };
        if let Some(found) = element.select(&selector).next() {
            let text = found.text().collect::<Vec<&str>>().join(" ");
            let text = text.split_whitespace().collect::<Vec<&str>>().join(" ");
            if !text.is_empty() {
                return Some(text);
            }
        }
    }
    None
}

/// Premiere URL absolue trouvee, resolue contre `base` quand le lien est relatif.
fn first_href(element: &scraper::ElementRef, base: &str) -> Option<String> {
    let href = if element.value().name() == "a" {
        element.value().attr("href").map(str::to_owned)
    } else {
        Selector::parse("a[href]")
            .ok()
            .and_then(|selector| element.select(&selector).next())
            .and_then(|link| link.value().attr("href").map(str::to_owned))
    }?;

    let href = href.trim();
    if href.is_empty() {
        return None;
    }
    Some(if href.starts_with("http") {
        href.to_string()
    } else {
        format!("{}{}", base.trim_end_matches('/'), href)
    })
}

/// Selectionne les cartes d'annonce, avec un repli sur les liens quand aucun selecteur structurel
/// ne correspond plus.
fn select_cards<'a>(document: &'a Html, primary: &[&str], fallback: &str) -> Vec<scraper::ElementRef<'a>> {
    for raw in primary {
        let Ok(selector) = Selector::parse(raw) else {
            continue;
        };
        let found: Vec<scraper::ElementRef<'a>> = document.select(&selector).collect();
        if !found.is_empty() {
            return found;
        }
    }
    Selector::parse(fallback)
        .map(|selector| document.select(&selector).collect())
        .unwrap_or_default()
}

// ===========================================================================
// HelloWork
// ===========================================================================

const HELLOWORK_SEARCH: &str = "https://www.hellowork.com/fr-fr/emploi/recherche.html";
const HELLOWORK_BASE: &str = "https://www.hellowork.com";

/// Connecteur HelloWork : le site n'a pas d'API publique, la page de recherche est analysee.
pub struct HelloWorkConnector {
    client: Client,
}

impl HelloWorkConnector {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    fn search_url(keywords: Option<&str>, location: Option<&str>) -> String {
        let mut url = format!("{HELLOWORK_SEARCH}?");
        if let Some(keywords) = keywords.map(str::trim).filter(|v| !v.is_empty()) {
            url.push_str(&format!("k={}", urlencoding::encode(keywords)));
        }
        if let Some(location) = location.map(str::trim).filter(|v| !v.is_empty()) {
            url.push_str(&format!("&l={}", urlencoding::encode(location)));
        }
        url
    }

    fn parse_page(html: &str) -> Vec<JobOffer> {
        let document = Html::parse_document(html);
        let cards = select_cards(
            &document,
            &[
                "[data-testid='offer-card']",
                ".offer-card",
                "article.offer",
                ".tw-card",
            ],
            "a[href*='/emploi/']",
        );

        cards
            .iter()
            .filter_map(|card| {
                let title = first_text(
                    card,
                    &["h3", "h2", "[data-testid='offer-title']", ".offer-title"],
                )?;
                let company = first_text(
                    card,
                    &[
                        "[data-testid='offer-company']",
                        ".company-name",
                        ".offer-company",
                    ],
                );
                let location = first_text(
                    card,
                    &[
                        "[data-testid='offer-location']",
                        ".offer-location",
                        ".location",
                    ],
                );
                let apply_url = first_href(card, HELLOWORK_BASE);

                Some(JobOffer {
                    remote: Some(detect_remote(&[location.as_deref(), Some(&title)])),
                    // Aucun identifiant dans le balisage : l'identite est hachee depuis les
                    // invariants, sinon chaque passage recreerait toutes les annonces.
                    source_id: Some(stable_id(&[
                        Some(&title),
                        company.as_deref(),
                        location.as_deref(),
                    ])),
                    title,
                    company,
                    location,
                    apply_url,
                    source: Some(JobSource::HelloWork.as_str().to_string()),
                    ..Default::default()
                })
            })
            .take(MAX_CARDS)
            .collect()
    }
}

#[async_trait]
impl AtsConnector for HelloWorkConnector {
    fn supports(&self, url: &str) -> bool {
        url.contains("hellowork.com")
    }

    fn source(&self) -> JobSource {
        JobSource::HelloWork
    }

    async fn fetch_jobs(
        &self,
        keywords: Option<&str>,
        location: Option<&str>,
    ) -> Result<Vec<JobOffer>, AppError> {
        let url = Self::search_url(keywords, location);
        let Some(html) = get_text(
            &self.client,
            "HELLOWORK",
            &url,
            &[("User-Agent", BROWSER_USER_AGENT)],
        )
        .await
        else {
            return Ok(Vec::new());
        };

        let offers = Self::parse_page(&html);
        if offers.is_empty() {
            // Un balisage modifie ne se distingue pas d'une recherche sans resultat : on le
            // journalise pour que la panne soit visible sans faire echouer la recherche.
            tracing::info!("HelloWork : aucune annonce extraite (balisage modifie ?)");
        }
        Ok(offers)
    }

    async fn fetch_from_url(&self, _url: &str) -> Result<Vec<JobOffer>, AppError> {
        // Une URL hellowork.com nue designe une recherche par defaut.
        self.fetch_jobs(None, None).await
    }
}

// ===========================================================================
// Seek
// ===========================================================================

const SEEK_BASE: &str = "https://www.seek.com.au";

/// Connecteur Seek (Australie).
///
/// Desactive par defaut : le site est protege par Kasada, qui repond 403 a toute requete serveur.
/// Le connecteur existe pour la parite avec la version Java, mais il ne faut pas s'attendre a ce
/// qu'il produise des resultats depuis une IP de datacentre.
pub struct SeekConnector {
    client: Client,
}

impl SeekConnector {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// Seek encode la recherche dans le chemin : `/jobs-in-{slug}` puis `?where=`.
    fn search_url(keywords: Option<&str>, location: Option<&str>) -> String {
        let slug = keywords
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(|k| {
                k.to_lowercase()
                    .split_whitespace()
                    .collect::<Vec<&str>>()
                    .join("-")
            })
            .unwrap_or_default();

        let mut url = if slug.is_empty() {
            format!("{SEEK_BASE}/jobs")
        } else {
            format!("{SEEK_BASE}/{}-jobs", urlencoding::encode(&slug))
        };
        if let Some(location) = location.map(str::trim).filter(|v| !v.is_empty()) {
            url.push_str(&format!("?where={}", urlencoding::encode(location)));
        }
        url
    }

    fn parse_page(html: &str) -> Vec<JobOffer> {
        let document = Html::parse_document(html);
        let cards = select_cards(
            &document,
            &["article[data-card-type='JobCard']", "article"],
            "a[href*='/job/']",
        );

        cards
            .iter()
            .filter_map(|card| {
                let title = first_text(card, &["[data-automation='jobTitle']", "h3", "h2"])?;
                let company = first_text(card, &["[data-automation='jobCompany']"]);
                let location = first_text(card, &["[data-automation='jobLocation']"]);
                let teaser = first_text(card, &["[data-automation='jobShortDescription']"]);
                let apply_url = first_href(card, SEEK_BASE);

                Some(JobOffer {
                    remote: Some(detect_remote(&[location.as_deref(), Some(&title)])),
                    // L'identifiant est dans l'URL ; a defaut, on hache les invariants.
                    source_id: apply_url
                        .as_deref()
                        .and_then(|url| {
                            SEEK_JOB_ID
                                .captures(url)
                                .map(|captures| captures[1].to_string())
                        })
                        .or_else(|| {
                            Some(stable_id(&[
                                Some(&title),
                                company.as_deref(),
                                location.as_deref(),
                            ]))
                        }),
                    // La page de resultats ne porte qu'un resume : l'offre est marquee STUB pour
                    // que l'enrichissement aille chercher la description complete plus tard.
                    embedding_status: Some(EMBEDDING_STATUS_STUB.to_string()),
                    description: teaser,
                    title,
                    company,
                    location,
                    apply_url,
                    source: Some(JobSource::Seek.as_str().to_string()),
                    ..Default::default()
                })
            })
            .take(MAX_CARDS)
            .collect()
    }
}

/// Offre dont seule l'accroche est connue : la description complete reste a recuperer.
const EMBEDDING_STATUS_STUB: &str = "STUB";

#[async_trait]
impl AtsConnector for SeekConnector {
    fn supports(&self, url: &str) -> bool {
        url.contains("seek.com")
    }

    fn source(&self) -> JobSource {
        JobSource::Seek
    }

    async fn fetch_jobs(
        &self,
        keywords: Option<&str>,
        location: Option<&str>,
    ) -> Result<Vec<JobOffer>, AppError> {
        let url = Self::search_url(keywords, location);
        let Some(html) = get_text(
            &self.client,
            "SEEK",
            &url,
            &[("User-Agent", BROWSER_USER_AGENT)],
        )
        .await
        else {
            return Ok(Vec::new());
        };

        let offers = Self::parse_page(&html);
        if offers.is_empty() {
            tracing::info!("Seek : aucune annonce extraite (mitigation anti-bot attendue)");
        }
        Ok(offers)
    }

    async fn fetch_from_url(&self, _url: &str) -> Result<Vec<JobOffer>, AppError> {
        self.fetch_jobs(None, None).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- HelloWork ------------------------------------------------------------------------

    #[test]
    fn hellowork_builds_its_search_url() {
        let url = HelloWorkConnector::search_url(Some("developpeur rust"), Some("Paris"));
        assert!(url.contains("k=developpeur%20rust"), "obtenu : {url}");
        assert!(url.contains("&l=Paris"));
        // Sans critere, l'URL reste valide (recherche par defaut).
        assert!(HelloWorkConnector::search_url(None, None).starts_with(HELLOWORK_SEARCH));
    }

    #[test]
    fn hellowork_extracts_offers_from_the_structured_markup() {
        let html = r#"
            <html><body>
              <div data-testid="offer-card">
                <h3>Developpeur Rust</h3>
                <span data-testid="offer-company">ACME</span>
                <span data-testid="offer-location">Noumea</span>
                <a href="/fr-fr/emploi/12345.html">voir</a>
              </div>
            </body></html>"#;

        let offers = HelloWorkConnector::parse_page(html);
        assert_eq!(offers.len(), 1);
        let offer = &offers[0];
        assert_eq!(offer.title, "Developpeur Rust");
        assert_eq!(offer.company.as_deref(), Some("ACME"));
        assert_eq!(offer.location.as_deref(), Some("Noumea"));
        // Le lien relatif doit etre resolu, sinon le candidat ne peut pas postuler.
        assert_eq!(
            offer.apply_url.as_deref(),
            Some("https://www.hellowork.com/fr-fr/emploi/12345.html")
        );
        assert_eq!(offer.source.as_deref(), Some("HELLOWORK"));
        assert_eq!(offer.source_id.as_ref().map(|id| id.len()), Some(32));
    }

    #[test]
    fn hellowork_identity_is_stable_across_two_passes() {
        let html = r#"<div data-testid="offer-card"><h3>Dev</h3>
            <span data-testid="offer-company">ACME</span></div>"#;
        // Sans identite stable, chaque passage de scraping recreerait toutes les annonces.
        let first = HelloWorkConnector::parse_page(html);
        let second = HelloWorkConnector::parse_page(html);
        assert_eq!(first[0].source_id, second[0].source_id);
    }

    #[test]
    fn hellowork_falls_back_to_links_when_the_markup_changed() {
        // Plus aucune carte structurelle : le repli sur les liens doit encore remonter quelque
        // chose, sinon un changement de balisage viderait la source en silence.
        let html = r#"<html><body>
            <a href="/fr-fr/emploi/999.html"><h3>Technicien reseau</h3></a>
        </body></html>"#;
        let offers = HelloWorkConnector::parse_page(html);
        assert_eq!(offers.len(), 1);
        assert_eq!(offers[0].title, "Technicien reseau");
    }

    #[test]
    fn hellowork_skips_cards_without_a_title() {
        // Une carte sans titre n'est pas une annonce (bandeau publicitaire, entete de section).
        let html = r#"<div data-testid="offer-card"><span>rien d'utile</span></div>"#;
        assert!(HelloWorkConnector::parse_page(html).is_empty());
    }

    #[test]
    fn hellowork_detects_remote_from_the_location() {
        let html = r#"<div data-testid="offer-card"><h3>Dev</h3>
            <span data-testid="offer-location">Télétravail</span></div>"#;
        assert_eq!(HelloWorkConnector::parse_page(html)[0].remote, Some(true));
    }

    // --- Seek -----------------------------------------------------------------------------

    #[test]
    fn seek_encodes_the_query_in_the_path() {
        let url = SeekConnector::search_url(Some("software engineer"), Some("Sydney"));
        assert!(url.contains("software-engineer-jobs"), "obtenu : {url}");
        assert!(url.contains("?where=Sydney"));
        // Sans mot-cle, on tombe sur la liste generale plutot que sur une URL malformee.
        assert_eq!(SeekConnector::search_url(None, None), "https://www.seek.com.au/jobs");
    }

    #[test]
    fn seek_reads_the_job_id_from_the_url_and_marks_the_offer_stub() {
        let html = r#"
            <article data-card-type="JobCard">
              <a data-automation="jobTitle" href="/job/78901234">Senior Engineer</a>
              <span data-automation="jobCompany">ACME</span>
              <span data-automation="jobLocation">Sydney NSW</span>
              <span data-automation="jobShortDescription">Great role</span>
            </article>"#;

        let offers = SeekConnector::parse_page(html);
        assert_eq!(offers.len(), 1);
        let offer = &offers[0];
        assert_eq!(offer.title, "Senior Engineer");
        // L'identifiant vient de l'URL, pas d'un hash : il est plus fiable.
        assert_eq!(offer.source_id.as_deref(), Some("78901234"));
        // Seule l'accroche est connue : STUB declenche l'enrichissement ulterieur.
        assert_eq!(offer.embedding_status.as_deref(), Some("STUB"));
        assert_eq!(offer.description.as_deref(), Some("Great role"));
    }

    #[test]
    fn seek_hashes_the_identity_when_the_url_carries_no_id() {
        let html = r#"<article data-card-type="JobCard">
            <span data-automation="jobTitle">Engineer</span>
            <span data-automation="jobCompany">ACME</span>
        </article>"#;
        let offers = SeekConnector::parse_page(html);
        assert_eq!(offers[0].source_id.as_ref().map(|id| id.len()), Some(32));
    }

    #[test]
    fn scrapers_tolerate_an_empty_or_broken_page() {
        // Kasada renvoie une page d'interstitiel : aucune annonce, mais pas de panique non plus.
        for html in ["", "<html><body>Access denied</body></html>", "<<<not html"] {
            assert!(SeekConnector::parse_page(html).is_empty(), "entree {html:?}");
            assert!(HelloWorkConnector::parse_page(html).is_empty(), "entree {html:?}");
        }
    }

    #[test]
    fn scrapers_recognise_their_hosts() {
        assert!(HelloWorkConnector::new(Client::new()).supports("https://www.hellowork.com/x"));
        assert!(SeekConnector::new(Client::new()).supports("https://www.seek.com.au/job/1"));
        assert!(!SeekConnector::new(Client::new()).supports("https://www.hellowork.com/x"));
    }
}
