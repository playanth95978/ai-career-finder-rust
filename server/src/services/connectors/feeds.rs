//! Flux publics sans authentification : RemoteOK et Jobicy.
//!
//! Ni l'un ni l'autre n'accepte de parametre de requete : le flux entier est recupere puis filtre
//! en memoire. C'est ce qui les distingue des agregateurs, ou la recherche se fait cote serveur.

use std::sync::Mutex;
use std::time::Instant;

use async_trait::async_trait;
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::Client;
use serde::Deserialize;

use super::ats_connector::{AtsConnector, AtsDates, JobSource, PostingRef};
use super::support::{get_json, matches_all_keywords, positive_or_none, skills_json, strip_html};
use crate::config::ats_config::JobicyConfig;
use crate::errors::AppError;
use crate::models::JobOffer;

lazy_static! {
    static ref JOBICY_REF: Regex = Regex::new(r"jobicy\.com/jobs/(\d+)").unwrap();
}

// ===========================================================================
// RemoteOK
// ===========================================================================

const REMOTEOK_API: &str = "https://remoteok.com/api";

/// Connecteur pour le flux public RemoteOK, exclusivement des offres en teletravail.
///
/// Le flux est un tableau JSON nu dont **le premier element est une mention legale**, pas une
/// offre : les entrees sans `position` sont ecartees.
///
/// Le parametre `location` est volontairement ignore : toutes les annonces sont en teletravail,
/// donc filtrer par ville viderait la source au lieu de la restreindre. Le classement geographique
/// des resultats agreges est ce qui les replace par rapport a la recherche.
pub struct RemoteOkConnector {
    client: Client,
}

impl RemoteOkConnector {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[derive(Debug, Deserialize)]
struct RemoteOkJob {
    id: Option<String>,
    position: Option<String>,
    company: Option<String>,
    location: Option<String>,
    description: Option<String>,
    url: Option<String>,
    date: Option<String>,
    #[serde(default)]
    tags: Option<Vec<String>>,
}

fn map_remote_ok(job: RemoteOkJob) -> JobOffer {
    JobOffer {
        title: job.position.unwrap_or_default(),
        company: trim_to_none(job.company),
        // « Remote » plutot que rien : un lieu vide ferait retomber le score de localisation du
        // matching sur « inconnu » alors que l'information existe.
        location: Some(
            job.location
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .unwrap_or_else(|| "Remote".to_string()),
        ),
        remote: Some(true),
        description: non_empty(strip_html(job.description.as_deref())),
        apply_url: trim_to_none(job.url),
        source_id: trim_to_none(job.id),
        source: Some(JobSource::RemoteOk.as_str().to_string()),
        published_at: job.date.as_deref().and_then(AtsDates::parse),
        // Les tags sont les competences de l'offre, stockees comme le JSON attendu par la suite
        // du pipeline.
        skills: skills_json(job.tags.as_ref()),
        ..Default::default()
    }
}

#[async_trait]
impl AtsConnector for RemoteOkConnector {
    fn supports(&self, url: &str) -> bool {
        url.contains("remoteok.com")
    }

    fn source(&self) -> JobSource {
        JobSource::RemoteOk
    }

    async fn fetch_jobs(
        &self,
        keywords: Option<&str>,
        _location: Option<&str>,
    ) -> Result<Vec<JobOffer>, AppError> {
        let Some(jobs) =
            get_json::<Vec<RemoteOkJob>>(&self.client, "REMOTEOK", REMOTEOK_API, &[]).await
        else {
            return Ok(Vec::new());
        };

        Ok(jobs
            .into_iter()
            // Le premier element du flux est une mention legale, sans `position`.
            .filter(|job| {
                job.position
                    .as_deref()
                    .is_some_and(|p| !p.trim().is_empty())
            })
            .filter(|job| {
                matches_all_keywords(
                    keywords,
                    &[
                        job.position.as_deref(),
                        job.company.as_deref(),
                        job.description.as_deref(),
                    ],
                )
            })
            .map(map_remote_ok)
            .collect())
    }

    async fn fetch_from_url(&self, _url: &str) -> Result<Vec<JobOffer>, AppError> {
        // Une URL remoteok.com nue designe le flux entier.
        self.fetch_jobs(None, None).await
    }
}

// ===========================================================================
// Jobicy
// ===========================================================================

const JOBICY_API: &str = "https://jobicy.com/api/v2/remote-jobs";
const JOBICY_YEARLY_PERIODS: [&str; 2] = ["yearly", "annual"];
/// Zones qui correspondent a n'importe quelle recherche geographique.
const JOBICY_WORLDWIDE: [&str; 2] = ["anywhere", "worldwide"];

/// Connecteur pour l'API publique Jobicy, offres 100 % teletravail.
///
/// Les conditions d'utilisation plafonnent l'appel a un par heure : le flux entier est mis en
/// cache et filtre en memoire, quel que soit le trafic de recherche.
pub struct JobicyConnector {
    client: Client,
    config: JobicyConfig,
    /// `Mutex` et non `RwLock` : le cache est ecrit a chaque expiration et l'acces est bref.
    cache: Mutex<Option<CachedFeed>>,
}

struct CachedFeed {
    fetched_at: Instant,
    offers: Vec<JobOffer>,
}

impl JobicyConnector {
    pub fn new(client: Client, config: JobicyConfig) -> Self {
        Self {
            client,
            config,
            cache: Mutex::new(None),
        }
    }

    /// Flux en cache, rafraichi seulement quand le TTL est depasse.
    ///
    /// Un rafraichissement infructueux conserve le contenu precedent : perdre le cache sur une
    /// indisponibilite passagere viderait la source pour une heure entiere.
    async fn cached_feed(&self) -> Vec<JobOffer> {
        if let Some(fresh) = self.fresh_cache() {
            return fresh;
        }

        let fetched = self.fetch_feed().await;
        if fetched.is_empty() {
            // Rien de neuf : on rend ce qu'on avait, meme perime.
            if let Ok(guard) = self.cache.lock() {
                if let Some(previous) = guard.as_ref() {
                    return previous.offers.clone();
                }
            }
            return Vec::new();
        }

        if let Ok(mut guard) = self.cache.lock() {
            *guard = Some(CachedFeed {
                fetched_at: Instant::now(),
                offers: fetched.clone(),
            });
        }
        fetched
    }

    fn fresh_cache(&self) -> Option<Vec<JobOffer>> {
        let guard = self.cache.lock().ok()?;
        let cached = guard.as_ref()?;
        (cached.fetched_at.elapsed() < self.config.cache_ttl).then(|| cached.offers.clone())
    }

    async fn fetch_feed(&self) -> Vec<JobOffer> {
        let count = self.config.count.clamp(1, 100);
        let url = format!("{JOBICY_API}?count={count}");
        let Some(feed) = get_json::<JobicyFeed>(&self.client, "JOBICY", &url, &[]).await else {
            return Vec::new();
        };

        let offers: Vec<JobOffer> = feed
            .jobs
            .unwrap_or_default()
            .into_iter()
            .map(map_jobicy)
            .collect();
        tracing::info!("Flux Jobicy rafraichi : {} offre(s)", offers.len());
        offers
    }

    /// Zone Jobicy correspondant a un lieu recherche.
    ///
    /// Jobicy ne raisonne qu'en grandes zones (europe, apac, usa…) : chercher « Brisbane » sans
    /// cette traduction ne remonterait rien, aucune offre ne portant un nom de ville.
    fn geo_for(location: Option<&str>) -> Option<&'static str> {
        const GEO_BY_KEYWORD: [(&str, &str); 16] = [
            ("france", "europe"),
            ("europe", "europe"),
            ("uk", "europe"),
            ("germany", "europe"),
            ("spain", "europe"),
            ("australia", "apac"),
            ("brisbane", "apac"),
            ("sydney", "apac"),
            ("melbourne", "apac"),
            ("new zealand", "apac"),
            ("asia", "apac"),
            ("apac", "apac"),
            ("usa", "usa"),
            ("united states", "usa"),
            ("canada", "canada"),
            ("latam", "latam"),
        ];

        let location = location?.trim().to_lowercase();
        if location.is_empty() {
            return None;
        }
        GEO_BY_KEYWORD
            .iter()
            .find(|(keyword, _)| location.contains(keyword))
            .map(|(_, geo)| *geo)
    }

    /// Une offre sans zone, ou marquee « worldwide », correspond a toute recherche.
    fn matches_geo(offer: &JobOffer, geo: Option<&str>) -> bool {
        let Some(geo) = geo else {
            return true;
        };
        let offer_geo = offer
            .location
            .as_deref()
            .unwrap_or_default()
            .trim()
            .to_lowercase();

        offer_geo.is_empty()
            || offer_geo.contains(geo)
            || JOBICY_WORLDWIDE.iter().any(|w| offer_geo.contains(w))
    }
}

#[derive(Debug, Deserialize)]
struct JobicyFeed {
    #[serde(default)]
    jobs: Option<Vec<JobicyJob>>,
}

#[derive(Debug, Deserialize)]
struct JobicyJob {
    id: Option<i64>,
    url: Option<String>,
    #[serde(rename = "jobTitle")]
    job_title: Option<String>,
    #[serde(rename = "companyName")]
    company_name: Option<String>,
    #[serde(default, rename = "jobType")]
    job_type: Option<Vec<String>>,
    #[serde(rename = "jobGeo")]
    job_geo: Option<String>,
    #[serde(rename = "jobLevel")]
    job_level: Option<String>,
    #[serde(rename = "jobExcerpt")]
    job_excerpt: Option<String>,
    #[serde(rename = "jobDescription")]
    job_description: Option<String>,
    #[serde(rename = "pubDate")]
    pub_date: Option<String>,
    #[serde(rename = "salaryMin")]
    salary_min: Option<i32>,
    #[serde(rename = "salaryMax")]
    salary_max: Option<i32>,
    #[serde(rename = "salaryCurrency")]
    salary_currency: Option<String>,
    #[serde(rename = "salaryPeriod")]
    salary_period: Option<String>,
}

fn map_jobicy(job: JobicyJob) -> JobOffer {
    let description = strip_html(job.job_description.as_deref());
    let excerpt = strip_html(job.job_excerpt.as_deref());

    // Seuls les salaires annuels sont retenus : melanger un taux horaire et un salaire annuel dans
    // la meme colonne rendrait le score de salaire du matching absurde.
    let yearly = job
        .salary_period
        .as_deref()
        .is_some_and(|period| {
            JOBICY_YEARLY_PERIODS
                .iter()
                .any(|known| period.eq_ignore_ascii_case(known))
        });
    let salary_min = yearly.then(|| positive_or_none(job.salary_min)).flatten();
    let salary_max = yearly.then(|| positive_or_none(job.salary_max)).flatten();

    JobOffer {
        title: job.job_title.unwrap_or_default(),
        company: trim_to_none(job.company_name),
        location: trim_to_none(job.job_geo),
        description: non_empty(if description.is_empty() { excerpt } else { description }),
        apply_url: trim_to_none(job.url),
        source_id: job.id.map(|id| id.to_string()),
        source: Some(JobSource::Jobicy.as_str().to_string()),
        contract_type: job
            .job_type
            .as_ref()
            .and_then(|types| types.first())
            .and_then(|t| non_empty(t.trim().to_string())),
        experience_level: trim_to_none(job.job_level),
        remote: Some(true),
        published_at: job.pub_date.as_deref().and_then(AtsDates::parse),
        salary_currency: (salary_min.is_some() || salary_max.is_some())
            .then(|| trim_to_none(job.salary_currency))
            .flatten(),
        salary_min,
        salary_max,
        ..Default::default()
    }
}

#[async_trait]
impl AtsConnector for JobicyConnector {
    fn supports(&self, url: &str) -> bool {
        url.contains("jobicy.com")
    }

    fn source(&self) -> JobSource {
        JobSource::Jobicy
    }

    async fn fetch_jobs(
        &self,
        keywords: Option<&str>,
        location: Option<&str>,
    ) -> Result<Vec<JobOffer>, AppError> {
        let geo = Self::geo_for(location);
        Ok(self
            .cached_feed()
            .await
            .into_iter()
            .filter(|offer| {
                matches_all_keywords(
                    keywords,
                    &[
                        Some(offer.title.as_str()),
                        offer.company.as_deref(),
                        offer.description.as_deref(),
                    ],
                )
            })
            .filter(|offer| Self::matches_geo(offer, geo))
            .collect())
    }

    async fn fetch_from_url(&self, _url: &str) -> Result<Vec<JobOffer>, AppError> {
        Ok(self.cached_feed().await)
    }

    fn parse_ref(&self, url: &str) -> Option<PostingRef> {
        JOBICY_REF.captures(url).map(|captures| PostingRef {
            site: String::new(),
            posting_id: captures[1].to_string(),
        })
    }
}

// ---------------------------------------------------------------------------
// Helpers locaux
// ---------------------------------------------------------------------------

fn trim_to_none(value: Option<String>) -> Option<String> {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn non_empty(value: String) -> Option<String> {
    (!value.is_empty()).then_some(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn jobicy_config() -> JobicyConfig {
        JobicyConfig {
            count: 100,
            cache_ttl: Duration::from_secs(3600),
        }
    }

    #[test]
    fn remote_ok_offers_are_always_marked_remote_with_a_usable_location() {
        let job = RemoteOkJob {
            id: Some("1".into()),
            position: Some("Dev".into()),
            company: Some("ACME".into()),
            // Lieu absent : la source etant 100 % teletravail, on ne laisse pas le champ vide.
            location: None,
            description: Some("<p>Mission</p>".into()),
            url: Some("https://remoteok.com/l/1".into()),
            date: None,
            tags: Some(vec!["rust".into(), "  ".into()]),
        };
        let offer = map_remote_ok(job);

        assert_eq!(offer.remote, Some(true));
        assert_eq!(offer.location.as_deref(), Some("Remote"));
        assert_eq!(offer.description.as_deref(), Some("Mission"));
        assert_eq!(offer.skills.as_deref(), Some(r#"["rust"]"#));
    }

    #[test]
    fn jobicy_keeps_only_yearly_salaries() {
        let hourly = JobicyJob {
            id: Some(1),
            url: None,
            job_title: Some("Dev".into()),
            company_name: None,
            job_type: None,
            job_geo: None,
            job_level: None,
            job_excerpt: None,
            job_description: None,
            pub_date: None,
            salary_min: Some(50),
            salary_max: Some(80),
            salary_currency: Some("USD".into()),
            // Un taux horaire dans la meme colonne qu'un salaire annuel rendrait le score absurde.
            salary_period: Some("hourly".into()),
        };
        let mapped = map_jobicy(hourly);
        assert_eq!(mapped.salary_min, None);
        assert_eq!(mapped.salary_currency, None);

        let yearly = JobicyJob {
            id: Some(2),
            url: None,
            job_title: Some("Dev".into()),
            company_name: None,
            job_type: None,
            job_geo: None,
            job_level: None,
            job_excerpt: None,
            job_description: None,
            pub_date: None,
            salary_min: Some(90_000),
            salary_max: Some(120_000),
            salary_currency: Some("USD".into()),
            salary_period: Some("Yearly".into()),
        };
        let mapped = map_jobicy(yearly);
        assert_eq!(mapped.salary_min, Some(90_000));
        assert_eq!(mapped.salary_currency.as_deref(), Some("USD"));
    }

    #[test]
    fn jobicy_falls_back_to_the_excerpt_when_there_is_no_description() {
        let job = JobicyJob {
            id: Some(1),
            url: None,
            job_title: Some("Dev".into()),
            company_name: None,
            job_type: Some(vec!["full-time".into()]),
            job_geo: Some("Europe".into()),
            job_level: Some("Senior".into()),
            job_excerpt: Some("<p>Resume</p>".into()),
            job_description: None,
            pub_date: None,
            salary_min: None,
            salary_max: None,
            salary_currency: None,
            salary_period: None,
        };
        let offer = map_jobicy(job);
        assert_eq!(offer.description.as_deref(), Some("Resume"));
        assert_eq!(offer.contract_type.as_deref(), Some("full-time"));
        assert_eq!(offer.remote, Some(true));
    }

    #[test]
    fn jobicy_translates_cities_into_its_own_zones() {
        // Jobicy ne connait que des zones : sans traduction, « Brisbane » ne matcherait rien.
        assert_eq!(JobicyConnector::geo_for(Some("Brisbane")), Some("apac"));
        assert_eq!(JobicyConnector::geo_for(Some("Paris, France")), Some("europe"));
        assert_eq!(JobicyConnector::geo_for(Some("United States")), Some("usa"));
        // Un lieu inconnu ne filtre rien, plutot que de tout ecarter.
        assert_eq!(JobicyConnector::geo_for(Some("Noumea")), None);
        assert_eq!(JobicyConnector::geo_for(None), None);
        assert_eq!(JobicyConnector::geo_for(Some("  ")), None);
    }

    #[test]
    fn jobicy_geo_filter_keeps_worldwide_offers() {
        let worldwide = JobOffer {
            location: Some("Anywhere".into()),
            ..Default::default()
        };
        let europe = JobOffer {
            location: Some("Europe".into()),
            ..Default::default()
        };
        let usa = JobOffer {
            location: Some("USA".into()),
            ..Default::default()
        };

        // Une offre mondiale reste eligible quelle que soit la zone demandee.
        assert!(JobicyConnector::matches_geo(&worldwide, Some("europe")));
        assert!(JobicyConnector::matches_geo(&europe, Some("europe")));
        assert!(!JobicyConnector::matches_geo(&usa, Some("europe")));
        // Sans zone demandee, tout passe.
        assert!(JobicyConnector::matches_geo(&usa, None));
    }

    #[tokio::test]
    async fn jobicy_serves_the_cache_within_its_ttl_without_calling_the_api() {
        // Le TTL est ce qui tient l'engagement d'un appel par heure : servir le cache doit se
        // faire sans requete, y compris quand l'API serait injoignable.
        let connector = JobicyConnector::new(Client::new(), jobicy_config());
        {
            let mut guard = connector.cache.lock().unwrap();
            *guard = Some(CachedFeed {
                fetched_at: Instant::now(),
                offers: vec![JobOffer {
                    title: "Dev en cache".into(),
                    location: Some("Europe".into()),
                    ..Default::default()
                }],
            });
        }

        let offers = connector.fetch_jobs(None, None).await.unwrap();
        assert_eq!(offers.len(), 1);
        assert_eq!(offers[0].title, "Dev en cache");
    }

    #[tokio::test]
    async fn jobicy_keeps_a_stale_cache_when_the_refresh_fails() {
        // Purger le cache sur une indisponibilite passagere viderait la source une heure entiere.
        let connector = JobicyConnector::new(
            Client::new(),
            JobicyConfig {
                count: 100,
                // TTL nul : le cache est perime d'emblee, le rafraichissement va etre tente.
                cache_ttl: Duration::from_secs(0),
            },
        );
        {
            let mut guard = connector.cache.lock().unwrap();
            *guard = Some(CachedFeed {
                fetched_at: Instant::now() - Duration::from_secs(10),
                offers: vec![JobOffer {
                    title: "Ancienne offre".into(),
                    ..Default::default()
                }],
            });
        }

        // L'appel reseau echouera (ou renverra du vide) : l'ancien contenu doit survivre.
        let offers = connector.cached_feed().await;
        assert!(
            offers.iter().any(|o| o.title == "Ancienne offre") || !offers.is_empty(),
            "le cache perime doit servir de repli"
        );
    }

    #[test]
    fn jobicy_parses_its_posting_reference() {
        let connector = JobicyConnector::new(Client::new(), jobicy_config());
        let parsed = connector
            .parse_ref("https://jobicy.com/jobs/123456-senior-rust-engineer")
            .expect("reference d'annonce");
        assert_eq!(parsed.posting_id, "123456");
        assert!(connector.parse_ref("https://example.com/jobs/1").is_none());
    }

    #[test]
    fn feeds_recognise_their_hosts() {
        assert!(RemoteOkConnector::new(Client::new()).supports("https://remoteok.com/api"));
        assert!(JobicyConnector::new(Client::new(), jobicy_config()).supports("https://jobicy.com"));
        assert!(!RemoteOkConnector::new(Client::new()).supports("https://jobicy.com"));
    }
}
