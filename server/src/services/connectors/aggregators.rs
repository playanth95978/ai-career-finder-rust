//! Agregateurs multi-entreprises : Adzuna, Careerjet, France Travail.
//!
//! Contrairement aux boards ATS, ceux-ci exposent une recherche transverse : `fetch_jobs` est donc
//! leur point d'entree principal, et `fetch_by_slug` est sans objet.
//!
//! Tous les trois exigent des identifiants. Un connecteur non configure renvoie une liste vide
//! **apres l'avoir journalise une fois** : le declarer actif produirait un 401 ou un 403 par
//! requete, ce qui est plus difficile a diagnostiquer qu'une source proprement inactive.

use std::sync::Mutex;

use async_trait::async_trait;
use chrono::{Duration as ChronoDuration, Utc};
use reqwest::Client;
use serde::Deserialize;

use super::ats_connector::{AtsConnector, AtsDates, JobSource};
use super::support::{basic_auth, detect_remote, first_non_blank, get_json, stable_id, strip_html};
use crate::config::ats_config::{AdzunaConfig, CareerjetConfig, FranceTravailConfig};
use crate::errors::AppError;
use crate::models::JobOffer;

// ===========================================================================
// Adzuna
// ===========================================================================

const ADZUNA_API: &str = "https://api.adzuna.com/v1/api/jobs";

/// Connecteur pour l'API de recherche Adzuna, partitionnee par marche (pays).
pub struct AdzunaConnector {
    client: Client,
    config: AdzunaConfig,
}

impl AdzunaConnector {
    pub fn new(client: Client, config: AdzunaConfig) -> Self {
        Self { client, config }
    }

    /// Requetes d'amorçage paginees par l'ingestion planifiee.
    pub async fn fetch_configured_seeds(&self) -> Vec<JobOffer> {
        if !self.config.is_configured() {
            return Vec::new();
        }
        if self.config.queries.is_empty() {
            tracing::warn!("Adzuna configure mais sans requete d'amorcage : rien a ingerer");
            return Vec::new();
        }

        let mut all = Vec::new();
        for query in &self.config.queries {
            all.extend(
                self.fetch_paginated(Some(query), self.config.where_filter.as_deref())
                    .await,
            );
        }
        all
    }

    /// Parcourt les pages jusqu'a `max_pages`, en s'arretant sur une page vide ou incomplete.
    ///
    /// Une page plus courte que `results_per_page` est la derniere : continuer paierait un appel
    /// pour un resultat vide.
    async fn fetch_paginated(
        &self,
        keywords: Option<&str>,
        location: Option<&str>,
    ) -> Vec<JobOffer> {
        let mut all = Vec::new();
        for page in 1..=self.config.max_pages {
            let offers = self
                .fetch_page(keywords, location, page, &self.config.country)
                .await;
            let count = offers.len() as i64;
            all.extend(offers);
            if count < self.config.results_per_page {
                break;
            }
        }
        all
    }

    async fn fetch_page(
        &self,
        what: Option<&str>,
        location: Option<&str>,
        page: i64,
        market: &str,
    ) -> Vec<JobOffer> {
        let (Some(app_id), Some(app_key)) =
            (self.config.app_id.as_deref(), self.config.app_key.as_deref())
        else {
            return Vec::new();
        };

        let mut url = format!(
            "{ADZUNA_API}/{market}/search/{page}?app_id={}&app_key={}&results_per_page={}&content-type=application/json",
            urlencoding::encode(app_id),
            urlencoding::encode(app_key),
            self.config.results_per_page
        );
        if let Some(what) = what.map(str::trim).filter(|v| !v.is_empty()) {
            url.push_str(&format!("&what={}", urlencoding::encode(what)));
        }
        if let Some(location) = location.map(str::trim).filter(|v| !v.is_empty()) {
            url.push_str(&format!("&where={}", urlencoding::encode(location)));
        }

        let Some(search) = get_json::<AdzunaSearch>(&self.client, "ADZUNA", &url, &[]).await else {
            return Vec::new();
        };

        search
            .results
            .unwrap_or_default()
            .into_iter()
            .map(|job| map_adzuna(job, &self.config.country))
            .collect()
    }
}

#[derive(Debug, Deserialize)]
struct AdzunaSearch {
    #[serde(default)]
    results: Option<Vec<AdzunaJob>>,
}

#[derive(Debug, Deserialize)]
struct AdzunaJob {
    id: Option<String>,
    title: Option<String>,
    description: Option<String>,
    company: Option<AdzunaCompany>,
    location: Option<AdzunaLocation>,
    redirect_url: Option<String>,
    created: Option<String>,
    salary_min: Option<f64>,
    salary_max: Option<f64>,
    contract_type: Option<String>,
    contract_time: Option<String>,
    category: Option<AdzunaCategory>,
}

#[derive(Debug, Deserialize)]
struct AdzunaCompany {
    #[serde(rename = "display_name")]
    display_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AdzunaLocation {
    #[serde(rename = "display_name")]
    display_name: Option<String>,
    #[serde(default)]
    area: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct AdzunaCategory {
    tag: Option<String>,
}

fn map_adzuna(job: AdzunaJob, default_country: &str) -> JobOffer {
    let title = job.title.unwrap_or_default();
    let location = job
        .location
        .as_ref()
        .and_then(|l| trim_to_none(l.display_name.clone()));
    // `area` est hierarchique, du plus large au plus fin : le premier element est le pays.
    let country = job
        .location
        .as_ref()
        .and_then(|l| l.area.as_ref())
        .and_then(|area| area.first().cloned())
        .unwrap_or_else(|| default_country.to_uppercase());
    let description = strip_html(job.description.as_deref());

    JobOffer {
        title: title.clone(),
        company: job.company.and_then(|c| trim_to_none(c.display_name)),
        location: location.clone(),
        country: Some(country),
        remote: Some(detect_remote(&[
            Some(&title),
            location.as_deref(),
            Some(&description),
        ])),
        description: non_empty(description),
        apply_url: trim_to_none(job.redirect_url),
        source_id: trim_to_none(job.id),
        source: Some(JobSource::Adzuna.as_str().to_string()),
        salary_min: rounded_or_none(job.salary_min),
        salary_max: rounded_or_none(job.salary_max),
        contract_type: first_non_blank(&[
            job.contract_type.as_deref(),
            job.contract_time.as_deref(),
        ]),
        published_at: job.created.as_deref().and_then(AtsDates::parse),
        // Vocabulaire propre a Adzuna : l'ingestion le traduit vers la categorie commune.
        source_category: job.category.and_then(|c| trim_to_none(c.tag)),
        ..Default::default()
    }
}

#[async_trait]
impl AtsConnector for AdzunaConnector {
    fn supports(&self, url: &str) -> bool {
        url.contains("adzuna.com")
    }

    fn source(&self) -> JobSource {
        JobSource::Adzuna
    }

    async fn fetch_jobs(
        &self,
        keywords: Option<&str>,
        location: Option<&str>,
    ) -> Result<Vec<JobOffer>, AppError> {
        self.fetch_jobs_by_country(keywords, location, None).await
    }

    /// Adzuna porte le pays dans le chemin de l'URL : c'est l'un des rares connecteurs pour
    /// lesquels `country` a un sens.
    async fn fetch_jobs_by_country(
        &self,
        keywords: Option<&str>,
        location: Option<&str>,
        country: Option<&str>,
    ) -> Result<Vec<JobOffer>, AppError> {
        if !self.config.is_configured() {
            tracing::warn!("Adzuna inactif : ADZUNA_APP_ID et ADZUNA_APP_KEY sont requis");
            return Ok(Vec::new());
        }

        let market = country
            .map(str::trim)
            .filter(|c| !c.is_empty())
            .map(|c| c.to_lowercase())
            .unwrap_or_else(|| self.config.country.clone());
        let location = location
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(str::to_owned)
            .or_else(|| self.config.where_filter.clone());

        Ok(self
            .fetch_page(keywords, location.as_deref(), 1, &market)
            .await)
    }

    async fn fetch_jobs_paginated(
        &self,
        keywords: Option<&str>,
        location: Option<&str>,
    ) -> Result<Vec<JobOffer>, AppError> {
        if !self.config.is_configured() {
            return Ok(Vec::new());
        }
        Ok(self.fetch_paginated(keywords, location).await)
    }
}

// ===========================================================================
// Careerjet
// ===========================================================================

const CAREERJET_API: &str = "https://search.api.careerjet.net/v4/query";
/// L'API plafonne le parametre `page` a 10.
const CAREERJET_MAX_PAGE: i64 = 10;

/// Connecteur pour la Search API v4 de Careerjet.
///
/// Trois exigences non negociables cote API : l'authentification Basic construite depuis la cle,
/// un en-tete `Referer` correspondant au site declare sur le compte partenaire (403 « Undeclared
/// referrer » sinon), et `user_ip` — l'IP de l'utilisateur final, remplacee par celle du serveur
/// en ingestion planifiee.
pub struct CareerjetConnector {
    client: Client,
    config: CareerjetConfig,
}

impl CareerjetConnector {
    pub fn new(client: Client, config: CareerjetConfig) -> Self {
        Self { client, config }
    }

    /// Careerjet partitionne son index par locale : un code marche selectionne la bonne.
    fn locale_for(&self, country: Option<&str>) -> String {
        const LOCALE_BY_COUNTRY: [(&str, &str); 6] = [
            ("fr", "fr_FR"),
            ("gb", "en_GB"),
            ("uk", "en_GB"),
            ("us", "en_US"),
            ("au", "en_AU"),
            ("ca", "en_CA"),
        ];

        let Some(country) = country.map(str::trim).filter(|c| !c.is_empty()) else {
            return self.config.locale_code.clone();
        };
        let lower = country.to_lowercase();
        LOCALE_BY_COUNTRY
            .iter()
            .find(|(code, _)| *code == lower)
            .map(|(_, locale)| (*locale).to_string())
            .unwrap_or_else(|| self.config.locale_code.clone())
    }

    /// Une page de resultats, plus le nombre total de pages annonce par l'API.
    async fn fetch_page(
        &self,
        keywords: Option<&str>,
        location: Option<&str>,
        page: i64,
        locale: &str,
    ) -> (Vec<JobOffer>, i64) {
        let (Some(api_key), Some(user_ip), Some(referer)) = (
            self.config.api_key.as_deref(),
            self.config.user_ip.as_deref(),
            self.config.referer.as_deref(),
        ) else {
            return (Vec::new(), 0);
        };

        let mut url = format!(
            "{CAREERJET_API}?locale_code={}&user_ip={}&user_agent={}&page={page}&page_size={}",
            urlencoding::encode(locale),
            urlencoding::encode(user_ip),
            urlencoding::encode(&self.config.user_agent),
            self.config.page_size.clamp(1, 100)
        );
        if let Some(keywords) = keywords.map(str::trim).filter(|v| !v.is_empty()) {
            url.push_str(&format!("&keywords={}", urlencoding::encode(keywords)));
        }
        if let Some(location) = location.map(str::trim).filter(|v| !v.is_empty()) {
            url.push_str(&format!("&location={}", urlencoding::encode(location)));
        }

        let auth = basic_auth(api_key, "");
        let headers = [("Authorization", auth.as_str()), ("Referer", referer)];

        let Some(response) =
            get_json::<CareerjetResponse>(&self.client, "CAREERJET", &url, &headers).await
        else {
            return (Vec::new(), 0);
        };

        let offers = response
            .jobs
            .unwrap_or_default()
            .into_iter()
            .map(|job| map_careerjet(job, locale))
            .collect();
        (offers, response.pages.unwrap_or(0))
    }
}

#[derive(Debug, Deserialize)]
struct CareerjetResponse {
    #[serde(default)]
    pages: Option<i64>,
    #[serde(default)]
    jobs: Option<Vec<CareerjetJob>>,
}

#[derive(Debug, Deserialize)]
struct CareerjetJob {
    title: Option<String>,
    company: Option<String>,
    locations: Option<String>,
    description: Option<String>,
    url: Option<String>,
    date: Option<String>,
    salary_min: Option<f64>,
    salary_max: Option<f64>,
    #[serde(rename = "salary_currency_code")]
    salary_currency_code: Option<String>,
}

fn map_careerjet(job: CareerjetJob, locale: &str) -> JobOffer {
    let title = job.title.unwrap_or_default();
    let location = trim_to_none(job.locations.clone());
    let description = strip_html(job.description.as_deref());
    let salary_min = rounded_or_none(job.salary_min);
    let salary_max = rounded_or_none(job.salary_max);

    JobOffer {
        title: title.clone(),
        company: trim_to_none(job.company.clone()),
        location: location.clone(),
        // Careerjet n'expose pas d'identifiant, et son `url` est une redirection signee qui change
        // a chaque appel : hacher les invariants est le seul moyen de dedupliquer entre ingestions.
        source_id: Some(stable_id(&[
            Some(&title),
            job.company.as_deref(),
            job.locations.as_deref(),
            job.date.as_deref(),
        ])),
        remote: Some(detect_remote(&[
            Some(&title),
            location.as_deref(),
            Some(&description),
        ])),
        description: non_empty(description),
        apply_url: trim_to_none(job.url),
        source: Some(JobSource::Careerjet.as_str().to_string()),
        country: country_of_locale(locale),
        published_at: job.date.as_deref().and_then(AtsDates::parse),
        salary_currency: (salary_min.is_some() || salary_max.is_some())
            .then(|| trim_to_none(job.salary_currency_code))
            .flatten(),
        salary_min,
        salary_max,
        ..Default::default()
    }
}

/// Pays deduit de la locale (`fr_FR` -> `FR`).
fn country_of_locale(locale: &str) -> Option<String> {
    locale
        .split('_')
        .nth(1)
        .map(str::to_uppercase)
        .filter(|c| !c.is_empty())
}

#[async_trait]
impl AtsConnector for CareerjetConnector {
    fn supports(&self, url: &str) -> bool {
        url.contains("careerjet.")
    }

    fn source(&self) -> JobSource {
        JobSource::Careerjet
    }

    async fn fetch_jobs(
        &self,
        keywords: Option<&str>,
        location: Option<&str>,
    ) -> Result<Vec<JobOffer>, AppError> {
        self.fetch_jobs_by_country(keywords, location, None).await
    }

    async fn fetch_jobs_by_country(
        &self,
        keywords: Option<&str>,
        location: Option<&str>,
        country: Option<&str>,
    ) -> Result<Vec<JobOffer>, AppError> {
        if !self.config.is_configured() {
            tracing::warn!(
                "Careerjet inactif : CAREERJET_API_KEY, CAREERJET_USER_IP et CAREERJET_REFERER sont requis"
            );
            return Ok(Vec::new());
        }

        let locale = self.locale_for(country);
        let (offers, _) = self.fetch_page(keywords, location, 1, &locale).await;
        Ok(offers)
    }

    async fn fetch_jobs_paginated(
        &self,
        keywords: Option<&str>,
        location: Option<&str>,
    ) -> Result<Vec<JobOffer>, AppError> {
        if !self.config.is_configured() {
            return Ok(Vec::new());
        }

        let locale = self.config.locale_code.clone();
        let limit = self.config.max_pages.min(CAREERJET_MAX_PAGE);
        let mut all = Vec::new();

        for page in 1..=limit {
            let (offers, pages) = self.fetch_page(keywords, location, page, &locale).await;
            if offers.is_empty() {
                break;
            }
            all.extend(offers);
            // On s'arrete sur le nombre de pages annonce par la reponse : `page_size` est
            // documente mais ignore par l'API (toujours 20 par page), donc une page « courte »
            // n'indique rien sur la fin des resultats.
            if page >= pages {
                break;
            }
        }
        Ok(all)
    }
}

// ===========================================================================
// France Travail
// ===========================================================================

const FRANCE_TRAVAIL_API: &str =
    "https://api.francetravail.io/partenaire/offresdemploi/v2/offres/search";
const FRANCE_TRAVAIL_TOKEN_URL: &str =
    "https://entreprise.francetravail.fr/connexion/oauth2/access_token?realm=/partenaire";
const FRANCE_TRAVAIL_SCOPE: &str = "api_offresdemploiv2 o2dsoffre";
const FRANCE_TRAVAIL_PAGE_SIZE: i64 = 50;
/// Taille de page de l'ingestion de masse. Plus grande que celle des recherches interactives : le
/// batch veut le moins d'allers-retours possible, pas la premiere page au plus vite.
const FRANCE_TRAVAIL_BATCH_PAGE_SIZE: i64 = 150;
/// Plafond dur de l'API : la plage `range` ne depasse pas `0-3149`, quelle que soit la requete.
/// C'est ce plafond qui justifie le partitionnement par grand domaine ROME.
const FRANCE_TRAVAIL_MAX_RESULTS: i64 = 3150;
/// Marge de renouvellement du jeton : un jeton qui expire pendant l'appel produirait un 401.
const TOKEN_RENEWAL_MARGIN_SECS: i64 = 60;

/// Connecteur pour l'API Offres d'emploi v2 de France Travail (OAuth2 client_credentials).
pub struct FranceTravailConnector {
    client: Client,
    config: FranceTravailConfig,
    token: Mutex<Option<CachedToken>>,
}

struct CachedToken {
    value: String,
    expires_at: chrono::DateTime<Utc>,
}

impl FranceTravailConnector {
    pub fn new(client: Client, config: FranceTravailConfig) -> Self {
        Self {
            client,
            config,
            token: Mutex::new(None),
        }
    }

    /// Jeton d'acces, mis en cache jusqu'a sa peremption.
    ///
    /// Sans cache, chaque recherche paierait un aller-retour OAuth2 supplementaire, et l'API
    /// limite le nombre de demandes de jeton.
    async fn access_token(&self) -> Option<String> {
        if let Ok(guard) = self.token.lock() {
            if let Some(cached) = guard.as_ref() {
                let margin = Utc::now() + ChronoDuration::seconds(TOKEN_RENEWAL_MARGIN_SECS);
                if cached.expires_at > margin {
                    return Some(cached.value.clone());
                }
            }
        }

        let (client_id, client_secret) = (
            self.config.client_id.as_deref()?,
            self.config.client_secret.as_deref()?,
        );

        let form = [
            ("grant_type", "client_credentials"),
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("scope", FRANCE_TRAVAIL_SCOPE),
        ];

        let response = self
            .client
            .post(FRANCE_TRAVAIL_TOKEN_URL)
            .form(&form)
            .send()
            .await
            .ok()?;

        if !response.status().is_success() {
            tracing::warn!(
                status = %response.status(),
                "France Travail : obtention du jeton refusee"
            );
            return None;
        }

        let token = response.json::<FranceTravailToken>().await.ok()?;
        let value = token.access_token?;
        let expires_at = Utc::now() + ChronoDuration::seconds(token.expires_in.unwrap_or(1_200));

        if let Ok(mut guard) = self.token.lock() {
            *guard = Some(CachedToken {
                value: value.clone(),
                expires_at,
            });
        }
        Some(value)
    }

    /// Recupere toutes les offres d'un grand domaine ROME, pour l'ingestion de masse.
    ///
    /// Deroule les plages jusqu'au plafond de l'API ou jusqu'a une page incomplete. La partition
    /// par domaine est ce qui rend ce plafond acceptable : voir
    /// [`crate::services::ingestion_partitions`].
    pub async fn fetch_by_grand_domaine(
        &self,
        grand_domaine: &str,
        departement: Option<&str>,
    ) -> Vec<JobOffer> {
        if !self.config.is_configured() {
            return Vec::new();
        }

        let partition = FranceTravailPartition {
            grand_domaine,
            departement,
        };
        let mut collected: Vec<JobOffer> = Vec::new();
        let mut start = 0i64;

        while start < FRANCE_TRAVAIL_MAX_RESULTS {
            // La derniere plage est tronquee au plafond : demander `3100-3249` renvoie une erreur
            // plutot qu'une page partielle.
            let size =
                (start + FRANCE_TRAVAIL_BATCH_PAGE_SIZE).min(FRANCE_TRAVAIL_MAX_RESULTS) - start;
            let page = self
                .fetch_range_filtered(None, None, Some(partition), start, size)
                .await;
            let fetched = page.len() as i64;
            collected.extend(page);

            // Une page vide ou plus courte que demandee est la derniere : continuer paierait un
            // appel pour rien, sur une API a quota.
            if fetched < size {
                break;
            }
            start += size;
        }

        collected
    }

    async fn fetch_range(
        &self,
        keywords: Option<&str>,
        location: Option<&str>,
        start: i64,
        size: i64,
    ) -> Vec<JobOffer> {
        self.fetch_range_filtered(keywords, location, None, start, size)
            .await
    }

    async fn fetch_range_filtered(
        &self,
        keywords: Option<&str>,
        location: Option<&str>,
        partition: Option<FranceTravailPartition<'_>>,
        start: i64,
        size: i64,
    ) -> Vec<JobOffer> {
        let Some(token) = self.access_token().await else {
            return Vec::new();
        };

        let mut url = format!("{FRANCE_TRAVAIL_API}?range={start}-{}", start + size - 1);
        if let Some(keywords) = keywords.map(str::trim).filter(|v| !v.is_empty()) {
            url.push_str(&format!("&motsCles={}", urlencoding::encode(keywords)));
        }
        // `codePostal` n'accepte qu'un code postal : « Paris » y produit un 400, donc aucune offre.
        // Une localisation non numerique est donc ignoree — des resultats nationaux, que le
        // reclassement geographique de `JobSearchService` remettra dans l'ordre, valent mieux
        // qu'une requete en erreur. Meme arbitrage que le connecteur Java.
        match location.map(str::trim).filter(|v| !v.is_empty()) {
            Some(location) if is_postal_code(location) => {
                url.push_str(&format!("&codePostal={}", urlencoding::encode(location)));
            }
            Some(location) => tracing::debug!(
                location,
                "France Travail : localisation ignoree, un code postal est attendu"
            ),
            None => {}
        }
        if let Some(partition) = partition {
            url.push_str(&format!(
                "&grandDomaine={}",
                urlencoding::encode(partition.grand_domaine)
            ));
            if let Some(departement) = partition.departement.map(str::trim).filter(|v| !v.is_empty())
            {
                url.push_str(&format!("&departement={}", urlencoding::encode(departement)));
            }
        }

        let bearer = format!("Bearer {token}");
        let headers = [("Authorization", bearer.as_str())];

        let Some(search) =
            get_json::<FranceTravailSearch>(&self.client, "FRANCE_TRAVAIL", &url, &headers).await
        else {
            return Vec::new();
        };

        search
            .resultats
            .unwrap_or_default()
            .into_iter()
            .map(map_france_travail)
            .collect()
    }
}

/// Filtres de partitionnement de l'ingestion de masse : grand domaine ROME et departement INSEE.
#[derive(Debug, Clone, Copy)]
struct FranceTravailPartition<'a> {
    grand_domaine: &'a str,
    departement: Option<&'a str>,
}

/// Vrai pour un code postal francais, seule forme que `codePostal` accepte.
///
/// Le test porte sur la forme et non sur l'existence du code : rejeter un code valide mais inconnu
/// d'une liste locale couterait plus que de laisser l'API repondre « aucun resultat ».
fn is_postal_code(value: &str) -> bool {
    let trimmed = value.trim();
    trimmed.len() == 5 && trimmed.chars().all(|c| c.is_ascii_digit())
}

#[derive(Debug, Deserialize)]
struct FranceTravailToken {
    access_token: Option<String>,
    expires_in: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct FranceTravailSearch {
    #[serde(default)]
    resultats: Option<Vec<FranceTravailOffer>>,
}

#[derive(Debug, Deserialize)]
struct FranceTravailOffer {
    id: Option<String>,
    intitule: Option<String>,
    description: Option<String>,
    #[serde(rename = "typeContrat")]
    type_contrat: Option<String>,
    #[serde(rename = "dateCreation")]
    date_creation: Option<String>,
    #[serde(rename = "romeCode")]
    rome_code: Option<String>,
    entreprise: Option<FranceTravailEntreprise>,
    #[serde(rename = "lieuTravail")]
    lieu_travail: Option<FranceTravailLieu>,
    #[serde(rename = "origineOffre")]
    origine_offre: Option<FranceTravailOrigine>,
}

#[derive(Debug, Deserialize)]
struct FranceTravailEntreprise {
    nom: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FranceTravailLieu {
    libelle: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FranceTravailOrigine {
    #[serde(rename = "urlOrigine")]
    url_origine: Option<String>,
}

fn map_france_travail(offer: FranceTravailOffer) -> JobOffer {
    JobOffer {
        title: offer.intitule.unwrap_or_default(),
        company: offer.entreprise.and_then(|e| trim_to_none(e.nom)),
        location: offer.lieu_travail.and_then(|l| trim_to_none(l.libelle)),
        country: Some("FR".to_string()),
        description: trim_to_none(offer.description),
        contract_type: trim_to_none(offer.type_contrat),
        apply_url: offer.origine_offre.and_then(|o| trim_to_none(o.url_origine)),
        source_id: trim_to_none(offer.id),
        source: Some(JobSource::FranceTravail.as_str().to_string()),
        // L'API ne publie pas de drapeau teletravail, et la description est deja en texte : on ne
        // devine pas, comme la version Java.
        remote: Some(false),
        published_at: offer.date_creation.as_deref().and_then(AtsDates::parse),
        // Code ROME : vocabulaire metier propre a France Travail, traduit a l'ingestion.
        source_category: trim_to_none(offer.rome_code),
        ..Default::default()
    }
}

#[async_trait]
impl AtsConnector for FranceTravailConnector {
    fn supports(&self, url: &str) -> bool {
        url.contains("francetravail.") || url.contains("pole-emploi.")
    }

    fn source(&self) -> JobSource {
        JobSource::FranceTravail
    }

    async fn fetch_jobs(
        &self,
        keywords: Option<&str>,
        location: Option<&str>,
    ) -> Result<Vec<JobOffer>, AppError> {
        if !self.config.is_configured() {
            tracing::warn!(
                "France Travail inactif : FRANCE_TRAVAIL_CLIENT_ID et FRANCE_TRAVAIL_CLIENT_SECRET sont requis"
            );
            return Ok(Vec::new());
        }
        Ok(self
            .fetch_range(keywords, location, 0, FRANCE_TRAVAIL_PAGE_SIZE)
            .await)
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

/// Salaire arrondi a l'entier. Les agregateurs publient des flottants ; la colonne est un entier.
fn rounded_or_none(value: Option<f64>) -> Option<i32> {
    value
        .filter(|amount| *amount > 0.0 && amount.is_finite())
        .map(|amount| amount.round() as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn adzuna_config(configured: bool) -> AdzunaConfig {
        AdzunaConfig {
            app_id: configured.then(|| "id".to_string()),
            app_key: configured.then(|| "key".to_string()),
            country: "fr".into(),
            where_filter: None,
            results_per_page: 50,
            max_pages: 5,
            queries: vec![],
        }
    }

    fn careerjet_config(configured: bool) -> CareerjetConfig {
        CareerjetConfig {
            api_key: configured.then(|| "key".to_string()),
            locale_code: "fr_FR".into(),
            user_ip: configured.then(|| "10.0.0.1".to_string()),
            user_agent: "ua".into(),
            referer: configured.then(|| "https://example.com".to_string()),
            page_size: 50,
            max_pages: 5,
        }
    }

    // --- Adzuna ---------------------------------------------------------------------------

    #[test]
    fn adzuna_reads_the_country_from_the_area_hierarchy() {
        let job = AdzunaJob {
            id: Some("1".into()),
            title: Some("Dev".into()),
            description: Some("<p>Mission</p>".into()),
            company: Some(AdzunaCompany { display_name: Some("ACME".into()) }),
            location: Some(AdzunaLocation {
                display_name: Some("Paris".into()),
                // `area` va du plus large au plus fin : le pays est en tete.
                area: Some(vec!["France".into(), "Ile-de-France".into(), "Paris".into()]),
            }),
            redirect_url: Some("https://adzuna.com/l/1".into()),
            created: Some("2026-08-01T00:00:00Z".into()),
            salary_min: Some(45000.4),
            salary_max: Some(55000.6),
            contract_type: None,
            contract_time: Some("full_time".into()),
            category: Some(AdzunaCategory { tag: Some("it-jobs".into()) }),
        };
        let offer = map_adzuna(job, "fr");

        assert_eq!(offer.country.as_deref(), Some("France"));
        assert_eq!(offer.description.as_deref(), Some("Mission"));
        // Les montants sont arrondis, pas tronques.
        assert_eq!(offer.salary_min, Some(45000));
        assert_eq!(offer.salary_max, Some(55001));
        // `contract_type` absent : on retombe sur `contract_time`.
        assert_eq!(offer.contract_type.as_deref(), Some("full_time"));
        assert_eq!(offer.source_category.as_deref(), Some("it-jobs"));
    }

    #[test]
    fn adzuna_falls_back_to_the_configured_country() {
        let job = AdzunaJob {
            id: Some("1".into()),
            title: Some("Dev".into()),
            description: None,
            company: None,
            // Pas de zone : le marche configure fait foi.
            location: Some(AdzunaLocation { display_name: Some("Paris".into()), area: None }),
            redirect_url: None,
            created: None,
            salary_min: None,
            salary_max: None,
            contract_type: None,
            contract_time: None,
            category: None,
        };
        assert_eq!(map_adzuna(job, "gb").country.as_deref(), Some("GB"));
    }

    #[tokio::test]
    async fn adzuna_stays_silent_when_unconfigured() {
        // Sans identifiants, chaque appel renverrait 401 : mieux vaut une source inactive.
        let connector = AdzunaConnector::new(Client::new(), adzuna_config(false));
        assert!(connector.fetch_jobs(Some("rust"), None).await.unwrap().is_empty());
        assert!(connector.fetch_configured_seeds().await.is_empty());
    }

    #[test]
    fn un_code_postal_est_reconnu_a_sa_forme() {
        assert!(is_postal_code("75001"));
        assert!(is_postal_code(" 69003 "));
    }

    #[test]
    fn un_nom_de_ville_nest_pas_un_code_postal() {
        // Le cas qui produisait un 400 de l'API, donc zero offre : « Paris » passe en `codePostal`.
        assert!(!is_postal_code("Paris"));
        assert!(!is_postal_code("75"));
        assert!(!is_postal_code("750011"));
        assert!(!is_postal_code(""));
        // Chiffres arabo-indiens : cinq caracteres numeriques, mais pas un code postal. C'est ce
        // que `is_ascii_digit` ecarte et que `is_numeric` aurait accepte.
        assert!(!is_postal_code("\u{0661}\u{0662}\u{0663}\u{0664}\u{0665}"));
    }

    #[tokio::test]
    async fn adzuna_ingestion_needs_seed_queries() {
        let mut config = adzuna_config(true);
        config.queries = vec![];
        let connector = AdzunaConnector::new(Client::new(), config);
        // Configure mais sans requete : rien a ingerer, et c'est journalise.
        assert!(connector.fetch_configured_seeds().await.is_empty());
    }

    // --- Careerjet ------------------------------------------------------------------------

    #[test]
    fn careerjet_derives_a_stable_id_from_the_posting_invariants() {
        let make = || CareerjetJob {
            title: Some("Dev".into()),
            company: Some("ACME".into()),
            locations: Some("Paris".into()),
            description: Some("<p>Mission</p>".into()),
            // L'URL est une redirection signee qui change a chaque appel : elle ne doit pas
            // entrer dans l'identite, sinon chaque ingestion recreerait toutes les offres.
            url: Some("https://careerjet.fr/redirect?sig=aleatoire".into()),
            date: Some("2026-08-01".into()),
            salary_min: None,
            salary_max: None,
            salary_currency_code: None,
        };

        let first = map_careerjet(make(), "fr_FR");
        let second = map_careerjet(make(), "fr_FR");
        assert_eq!(first.source_id, second.source_id);
        assert_eq!(first.source_id.as_ref().map(|id| id.len()), Some(32));
        assert_eq!(first.country.as_deref(), Some("FR"));
    }

    #[test]
    fn careerjet_locale_selects_the_market() {
        let connector = CareerjetConnector::new(Client::new(), careerjet_config(true));
        assert_eq!(connector.locale_for(Some("au")), "en_AU");
        assert_eq!(connector.locale_for(Some("UK")), "en_GB");
        // Un marche inconnu retombe sur la locale configuree plutot que d'echouer.
        assert_eq!(connector.locale_for(Some("zz")), "fr_FR");
        assert_eq!(connector.locale_for(None), "fr_FR");
    }

    #[test]
    fn country_of_locale_extracts_the_region() {
        assert_eq!(country_of_locale("fr_FR").as_deref(), Some("FR"));
        assert_eq!(country_of_locale("en_AU").as_deref(), Some("AU"));
        assert_eq!(country_of_locale("malforme"), None);
    }

    #[tokio::test]
    async fn careerjet_stays_silent_without_key_ip_and_referer() {
        let connector = CareerjetConnector::new(Client::new(), careerjet_config(false));
        assert!(connector.fetch_jobs(Some("rust"), None).await.unwrap().is_empty());
    }

    // --- France Travail ------------------------------------------------------------------

    #[test]
    fn france_travail_maps_its_french_field_names() {
        let offer = FranceTravailOffer {
            id: Some("187XYZB".into()),
            intitule: Some("Developpeur".into()),
            description: Some("Mission longue".into()),
            type_contrat: Some("CDI".into()),
            date_creation: Some("2026-08-01T09:00:00Z".into()),
            rome_code: Some("M1805".into()),
            entreprise: Some(FranceTravailEntreprise { nom: Some("ACME".into()) }),
            lieu_travail: Some(FranceTravailLieu { libelle: Some("75 - PARIS".into()) }),
            origine_offre: Some(FranceTravailOrigine {
                url_origine: Some("https://candidat.francetravail.fr/offres/187XYZB".into()),
            }),
        };
        let mapped = map_france_travail(offer);

        assert_eq!(mapped.title, "Developpeur");
        assert_eq!(mapped.company.as_deref(), Some("ACME"));
        assert_eq!(mapped.location.as_deref(), Some("75 - PARIS"));
        assert_eq!(mapped.country.as_deref(), Some("FR"));
        assert_eq!(mapped.source_category.as_deref(), Some("M1805"));
        // L'API n'expose pas de drapeau teletravail : on ne devine pas.
        assert_eq!(mapped.remote, Some(false));
    }

    #[tokio::test]
    async fn france_travail_stays_silent_when_unconfigured() {
        let connector = FranceTravailConnector::new(
            Client::new(),
            FranceTravailConfig {
                client_id: None,
                client_secret: None,
                departements: vec![],
            },
        );
        assert!(connector.fetch_jobs(Some("rust"), None).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn france_travail_serves_a_cached_token() {
        let connector = FranceTravailConnector::new(
            Client::new(),
            FranceTravailConfig {
                client_id: Some("id".into()),
                client_secret: Some("secret".into()),
                departements: vec![],
            },
        );
        {
            let mut guard = connector.token.lock().unwrap();
            *guard = Some(CachedToken {
                value: "jeton-en-cache".into(),
                expires_at: Utc::now() + ChronoDuration::hours(1),
            });
        }
        // Le jeton en cache doit etre rendu sans aller-retour OAuth2.
        assert_eq!(connector.access_token().await.as_deref(), Some("jeton-en-cache"));
    }

    #[tokio::test]
    async fn france_travail_renews_a_token_inside_the_expiry_margin() {
        let connector = FranceTravailConnector::new(
            Client::new(),
            FranceTravailConfig {
                client_id: None,
                client_secret: None,
                departements: vec![],
            },
        );
        {
            let mut guard = connector.token.lock().unwrap();
            *guard = Some(CachedToken {
                value: "presque-expire".into(),
                // Dans la marge : un jeton qui expire pendant l'appel produirait un 401.
                expires_at: Utc::now() + ChronoDuration::seconds(TOKEN_RENEWAL_MARGIN_SECS / 2),
            });
        }
        // Le jeton en cache est ecarte ; sans identifiants le renouvellement echoue, donc `None`.
        assert_eq!(connector.access_token().await, None);
    }

    // --- Divers ---------------------------------------------------------------------------

    #[test]
    fn rounded_or_none_rejects_zero_and_non_finite_amounts() {
        assert_eq!(rounded_or_none(Some(45000.6)), Some(45001));
        assert_eq!(rounded_or_none(Some(0.0)), None);
        assert_eq!(rounded_or_none(Some(-10.0)), None);
        assert_eq!(rounded_or_none(Some(f64::NAN)), None);
        assert_eq!(rounded_or_none(Some(f64::INFINITY)), None);
        assert_eq!(rounded_or_none(None), None);
    }

    #[test]
    fn aggregators_recognise_their_hosts() {
        assert!(AdzunaConnector::new(Client::new(), adzuna_config(true))
            .supports("https://api.adzuna.com/v1"));
        assert!(CareerjetConnector::new(Client::new(), careerjet_config(true))
            .supports("https://www.careerjet.fr/offre"));
        assert!(FranceTravailConnector::new(
            Client::new(),
            FranceTravailConfig { client_id: None, client_secret: None, departements: vec![] }
        )
        .supports("https://candidat.francetravail.fr/offres/1"));
    }
}
