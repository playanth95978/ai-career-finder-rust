//! Connecteurs de boards ATS : Greenhouse, Lever, SmartRecruiters, Ashby, Workable, Recruitee.
//!
//! Regroupes parce qu'ils partagent la meme forme : aucun endpoint de recherche transverse, un
//! board par entreprise, et donc `fetch_jobs` inoperant. C'est `fetch_by_slug` qui les rend
//! utiles — sonder un slug sans savoir quel ATS l'entreprise utilise, un resultat vide signifiant
//! « pas celui-la ». Un fichier par connecteur repeterait six fois la meme structure.
//!
//! Aucun ne soumet de candidature : la soumission automatique sort du cadre des CGU des ATS, et
//! envoyer au nom du candidat exige son geste explicite. Le candidat postule via `apply_url`.

use async_trait::async_trait;
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::Client;
use serde::Deserialize;

use super::ats_connector::{AtsConnector, AtsDates, JobSource, PostingRef};
use super::support::{
    detect_remote, first_non_blank, get_json, join_location, positive_or_none, slug_after,
    strip_html,
};
use crate::errors::AppError;
use crate::models::JobOffer;

lazy_static! {
    /// URL d'annonce Greenhouse : (job-)boards.greenhouse.io/{board}/jobs/{id}.
    static ref GREENHOUSE_REF: Regex = Regex::new(r"greenhouse\.io/([^/]+)/jobs/(\d+)").unwrap();
    static ref GREENHOUSE_JID: Regex = Regex::new(r"[?&]gh_jid=(\d+)").unwrap();
    /// URL d'annonce Lever : jobs.lever.co/{site}/{postingId}, l'identifiant etant un UUID.
    static ref LEVER_REF: Regex = Regex::new(r"lever\.co/([^/]+)/([0-9a-fA-F-]{36})").unwrap();
    /// URL d'annonce SmartRecruiters : jobs.smartrecruiters.com/{companyId}/{postingId}.
    static ref SMARTRECRUITERS_REF: Regex = Regex::new(r"smartrecruiters\.com/([^/]+)/(\d+)").unwrap();
    /// URL d'annonce Ashby : jobs.ashbyhq.com/{org}/{postingId}, l'identifiant etant un UUID.
    static ref ASHBY_REF: Regex = Regex::new(r"ashbyhq\.com/([^/?#]+)/([0-9a-fA-F-]{36})").unwrap();
    /// URL d'annonce Workable, avec ou sans compte dans le chemin.
    static ref WORKABLE_ACCOUNT_POSTING: Regex =
        Regex::new(r"apply\.workable\.com/([^/]+)/j/([A-Za-z0-9]+)").unwrap();
    static ref WORKABLE_DIRECT_POSTING: Regex =
        Regex::new(r"apply\.workable\.com/j/([A-Za-z0-9]+)").unwrap();
    static ref WORKABLE_SUBDOMAIN: Regex =
        Regex::new(r"https?://([^./]+)\.workable\.com").unwrap();
    /// Site carriere Recruitee : {company}.recruitee.com — le sous-domaine est le slug.
    static ref RECRUITEE_SUBDOMAIN: Regex =
        Regex::new(r"https?://([^./]+)\.recruitee\.com").unwrap();
    static ref RECRUITEE_OFFER_SLUG: Regex = Regex::new(r"recruitee\.com/o/([^/?#]+)").unwrap();
}

// ===========================================================================
// Greenhouse
// ===========================================================================

const GREENHOUSE_API: &str = "https://boards-api.greenhouse.io/v1/boards";

/// Connecteur pour la Job Board API de Greenhouse, publique, un board par entreprise.
pub struct GreenhouseConnector {
    client: Client,
}

impl GreenhouseConnector {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn fetch_from_board(&self, board: &str) -> Vec<JobOffer> {
        let url = format!("{GREENHOUSE_API}/{board}/jobs?content=true");
        let Some(payload) =
            get_json::<GreenhouseBoard>(&self.client, "GREENHOUSE", &url, &[]).await
        else {
            return Vec::new();
        };

        payload
            .jobs
            .unwrap_or_default()
            .into_iter()
            .map(|job| map_greenhouse(job, board))
            .collect()
    }
}

#[derive(Debug, Deserialize)]
struct GreenhouseBoard {
    #[serde(default)]
    jobs: Option<Vec<GreenhouseJob>>,
}

#[derive(Debug, Deserialize)]
struct GreenhouseJob {
    id: Option<i64>,
    title: Option<String>,
    content: Option<String>,
    location: Option<GreenhouseLocation>,
    absolute_url: Option<String>,
    updated_at: Option<String>,
    first_published: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GreenhouseLocation {
    name: Option<String>,
}

fn map_greenhouse(job: GreenhouseJob, board: &str) -> JobOffer {
    let location = job.location.and_then(|l| trim_to_none(l.name));
    let title = job.title.unwrap_or_default();
    JobOffer {
        title: title.clone(),
        company: Some(board.to_string()),
        location: location.clone(),
        // `content` est du balisage echappe : le parseur le desechappe et l'aplatit d'un coup.
        description: non_empty(strip_html(job.content.as_deref())),
        apply_url: trim_to_none(job.absolute_url),
        source_id: job.id.map(|id| id.to_string()),
        source: Some(JobSource::Greenhouse.as_str().to_string()),
        published_at: parse_first(&[job.first_published.as_deref(), job.updated_at.as_deref()]),
        remote: Some(detect_remote(&[location.as_deref(), Some(&title)])),
        ..Default::default()
    }
}

#[async_trait]
impl AtsConnector for GreenhouseConnector {
    fn supports(&self, url: &str) -> bool {
        url.contains("greenhouse.io")
    }

    fn source(&self) -> JobSource {
        JobSource::Greenhouse
    }

    /// Board sans recherche transverse : la recherche par mots-cles est sans objet.
    async fn fetch_jobs(&self, _: Option<&str>, _: Option<&str>) -> Result<Vec<JobOffer>, AppError> {
        Ok(Vec::new())
    }

    async fn fetch_by_slug(&self, slug: &str) -> Result<Vec<JobOffer>, AppError> {
        Ok(if slug.trim().is_empty() {
            Vec::new()
        } else {
            self.fetch_from_board(slug.trim()).await
        })
    }

    async fn fetch_from_url(&self, url: &str) -> Result<Vec<JobOffer>, AppError> {
        let board = slug_after(Some(url), "greenhouse.io/");
        self.fetch_by_slug(&board).await
    }

    fn parse_ref(&self, url: &str) -> Option<PostingRef> {
        if let Some(captures) = GREENHOUSE_REF.captures(url) {
            return Some(PostingRef {
                site: captures[1].to_string(),
                posting_id: captures[2].to_string(),
            });
        }
        GREENHOUSE_JID.captures(url).map(|captures| PostingRef {
            site: String::new(),
            posting_id: captures[1].to_string(),
        })
    }
}

// ===========================================================================
// Lever
// ===========================================================================

const LEVER_API: &str = "https://api.lever.co/v0/postings";

/// Connecteur pour l'API de postings Lever, publique, un flux par entreprise.
pub struct LeverConnector {
    client: Client,
}

impl LeverConnector {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn fetch_from_company(&self, company: &str) -> Vec<JobOffer> {
        let url = format!("{LEVER_API}/{company}?mode=json");
        let Some(postings) = get_json::<Vec<LeverPosting>>(&self.client, "LEVER", &url, &[]).await
        else {
            return Vec::new();
        };

        postings
            .into_iter()
            .map(|posting| map_lever(posting, company))
            .collect()
    }
}

#[derive(Debug, Deserialize)]
struct LeverPosting {
    id: Option<String>,
    text: Option<String>,
    #[serde(rename = "descriptionPlain")]
    description_plain: Option<String>,
    description: Option<String>,
    #[serde(rename = "applyUrl")]
    apply_url: Option<String>,
    #[serde(rename = "hostedUrl")]
    hosted_url: Option<String>,
    #[serde(rename = "createdAt")]
    created_at: Option<i64>,
    categories: Option<LeverCategories>,
}

#[derive(Debug, Deserialize)]
struct LeverCategories {
    location: Option<String>,
    commitment: Option<String>,
}

fn map_lever(posting: LeverPosting, company: &str) -> JobOffer {
    let (location, contract_type) = posting
        .categories
        .map(|c| (trim_to_none(c.location), trim_to_none(c.commitment)))
        .unwrap_or((None, None));
    let title = posting.text.unwrap_or_default();
    let stripped = strip_html(posting.description.as_deref());

    JobOffer {
        title: title.clone(),
        company: Some(company.to_string()),
        location: location.clone(),
        contract_type,
        description: first_non_blank(&[posting.description_plain.as_deref(), Some(&stripped)]),
        apply_url: first_non_blank(&[posting.apply_url.as_deref(), posting.hosted_url.as_deref()]),
        source_id: trim_to_none(posting.id),
        source: Some(JobSource::Lever.as_str().to_string()),
        // `createdAt` est en millisecondes depuis l'epoch.
        published_at: posting.created_at.and_then(AtsDates::from_epoch),
        remote: Some(detect_remote(&[location.as_deref(), Some(&title)])),
        ..Default::default()
    }
}

#[async_trait]
impl AtsConnector for LeverConnector {
    fn supports(&self, url: &str) -> bool {
        url.contains("lever.co")
    }

    fn source(&self) -> JobSource {
        JobSource::Lever
    }

    async fn fetch_jobs(&self, _: Option<&str>, _: Option<&str>) -> Result<Vec<JobOffer>, AppError> {
        Ok(Vec::new())
    }

    async fn fetch_by_slug(&self, slug: &str) -> Result<Vec<JobOffer>, AppError> {
        Ok(if slug.trim().is_empty() {
            Vec::new()
        } else {
            self.fetch_from_company(slug.trim()).await
        })
    }

    async fn fetch_from_url(&self, url: &str) -> Result<Vec<JobOffer>, AppError> {
        let company = slug_after(Some(url), "lever.co/");
        self.fetch_by_slug(&company).await
    }

    fn parse_ref(&self, url: &str) -> Option<PostingRef> {
        LEVER_REF.captures(url).map(|captures| PostingRef {
            site: captures[1].to_string(),
            posting_id: captures[2].to_string(),
        })
    }
}

// ===========================================================================
// SmartRecruiters
// ===========================================================================

const SMARTRECRUITERS_API: &str = "https://api.smartrecruiters.com/v1/companies";
const SMARTRECRUITERS_POSTING_BASE: &str = "https://jobs.smartrecruiters.com/";

/// Connecteur SmartRecruiters, en lecture seule : il lit les offres publiees d'un board public.
pub struct SmartRecruitersConnector {
    client: Client,
}

impl SmartRecruitersConnector {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn fetch_from_company(&self, company_id: &str) -> Vec<JobOffer> {
        let url = format!("{SMARTRECRUITERS_API}/{company_id}/postings");
        let Some(payload) =
            get_json::<SmartRecruitersPostings>(&self.client, "SMARTRECRUITERS", &url, &[]).await
        else {
            return Vec::new();
        };

        payload
            .content
            .unwrap_or_default()
            .into_iter()
            .map(|posting| map_smart_recruiters(posting, company_id))
            .collect()
    }
}

#[derive(Debug, Deserialize)]
struct SmartRecruitersPostings {
    #[serde(default)]
    content: Option<Vec<SmartRecruitersPosting>>,
}

#[derive(Debug, Deserialize)]
struct SmartRecruitersPosting {
    id: Option<String>,
    name: Option<String>,
    #[serde(rename = "releasedDate")]
    released_date: Option<String>,
    location: Option<SmartRecruitersLocation>,
    #[serde(rename = "typeOfEmployment")]
    type_of_employment: Option<SmartRecruitersEmployment>,
}

#[derive(Debug, Deserialize)]
struct SmartRecruitersLocation {
    city: Option<String>,
    country: Option<String>,
    remote: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct SmartRecruitersEmployment {
    label: Option<String>,
}

fn map_smart_recruiters(posting: SmartRecruitersPosting, company_id: &str) -> JobOffer {
    let id = posting.id.clone().unwrap_or_default();
    let (location, country, remote) = posting
        .location
        .map(|l| (trim_to_none(l.city), trim_to_none(l.country), l.remote))
        .unwrap_or((None, None, None));

    JobOffer {
        title: posting.name.unwrap_or_default(),
        company: Some(company_id.to_string()),
        location,
        country,
        // La source expose un drapeau : on l'utilise plutot que l'heuristique textuelle.
        remote: Some(remote.unwrap_or(false)),
        apply_url: Some(format!("{SMARTRECRUITERS_POSTING_BASE}{company_id}/{id}")),
        source_id: trim_to_none(posting.id),
        source: Some(JobSource::SmartRecruiters.as_str().to_string()),
        contract_type: posting.type_of_employment.and_then(|t| trim_to_none(t.label)),
        published_at: posting.released_date.as_deref().and_then(AtsDates::parse),
        ..Default::default()
    }
}

#[async_trait]
impl AtsConnector for SmartRecruitersConnector {
    fn supports(&self, url: &str) -> bool {
        url.contains("smartrecruiters.com")
    }

    fn source(&self) -> JobSource {
        JobSource::SmartRecruiters
    }

    async fn fetch_jobs(&self, _: Option<&str>, _: Option<&str>) -> Result<Vec<JobOffer>, AppError> {
        Ok(Vec::new())
    }

    async fn fetch_by_slug(&self, slug: &str) -> Result<Vec<JobOffer>, AppError> {
        Ok(if slug.trim().is_empty() {
            Vec::new()
        } else {
            self.fetch_from_company(slug.trim()).await
        })
    }

    async fn fetch_from_url(&self, url: &str) -> Result<Vec<JobOffer>, AppError> {
        let company = self
            .parse_ref(url)
            .map(|r| r.site)
            .filter(|site| !site.is_empty())
            .unwrap_or_else(|| slug_after(Some(url), "smartrecruiters.com/"));
        self.fetch_by_slug(&company).await
    }

    fn parse_ref(&self, url: &str) -> Option<PostingRef> {
        SMARTRECRUITERS_REF
            .captures(url)
            .map(|captures| PostingRef {
                site: captures[1].to_string(),
                posting_id: captures[2].to_string(),
            })
    }
}

// ===========================================================================
// Ashby
// ===========================================================================

const ASHBY_API: &str = "https://api.ashbyhq.com/posting-api/job-board/";
/// `workplaceType` porte aussi « Hybrid », qui n'est pas du teletravail.
const ASHBY_REMOTE_WORKPLACE: &str = "Remote";

/// Connecteur pour l'API publique de Job Posting d'Ashby, sans authentification.
pub struct AshbyConnector {
    client: Client,
    /// Boards parcourus par l'ingestion ; vide = seulement les URL collees par l'utilisateur.
    boards: Vec<String>,
}

impl AshbyConnector {
    pub fn new(client: Client, boards: Vec<String>) -> Self {
        Self { client, boards }
    }

    /// Toutes les annonces listees des boards configures (point d'entree de l'ingestion).
    pub async fn fetch_configured_boards(&self) -> Vec<JobOffer> {
        let mut all = Vec::new();
        for board in &self.boards {
            all.extend(self.fetch_from_board(board).await);
        }
        all
    }

    pub async fn fetch_from_board(&self, board: &str) -> Vec<JobOffer> {
        let url = format!("{ASHBY_API}{board}");
        let Some(payload) = get_json::<AshbyBoard>(&self.client, "ASHBY", &url, &[]).await else {
            return Vec::new();
        };

        payload
            .jobs
            .unwrap_or_default()
            .into_iter()
            // Une annonce non listee est une annonce que l'entreprise a choisi de ne pas publier.
            .filter(|job| job.is_listed.unwrap_or(true))
            .map(|job| map_ashby(job, board))
            .collect()
    }

    /// Nom du board depuis une URL hebergee ou depuis le chemin d'API.
    fn board_of(url: &str) -> String {
        if url.contains("/job-board/") {
            slug_after(Some(url), "/job-board/")
        } else {
            slug_after(Some(url), "ashbyhq.com/")
        }
    }
}

#[derive(Debug, Deserialize)]
struct AshbyBoard {
    #[serde(default)]
    jobs: Option<Vec<AshbyJob>>,
}

#[derive(Debug, Deserialize)]
struct AshbyJob {
    id: Option<String>,
    title: Option<String>,
    location: Option<String>,
    #[serde(rename = "isListed")]
    is_listed: Option<bool>,
    #[serde(rename = "isRemote")]
    is_remote: Option<bool>,
    #[serde(rename = "workplaceType")]
    workplace_type: Option<String>,
    #[serde(rename = "employmentType")]
    employment_type: Option<String>,
    #[serde(rename = "descriptionHtml")]
    description_html: Option<String>,
    #[serde(rename = "descriptionPlain")]
    description_plain: Option<String>,
    #[serde(rename = "publishedAt")]
    published_at: Option<String>,
    address: Option<AshbyAddress>,
    #[serde(rename = "jobUrl")]
    job_url: Option<String>,
    #[serde(rename = "applyUrl")]
    apply_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AshbyAddress {
    #[serde(rename = "postalAddress")]
    postal_address: Option<AshbyPostalAddress>,
}

#[derive(Debug, Deserialize)]
struct AshbyPostalAddress {
    #[serde(rename = "addressCountry")]
    address_country: Option<String>,
}

fn map_ashby(job: AshbyJob, board: &str) -> JobOffer {
    let apply_url = first_non_blank(&[job.apply_url.as_deref(), job.job_url.as_deref()]);
    let stripped = strip_html(job.description_html.as_deref());

    JobOffer {
        title: job.title.unwrap_or_default(),
        company: Some(board.to_string()),
        location: trim_to_none(job.location),
        country: job
            .address
            .and_then(|a| a.postal_address)
            .and_then(|p| trim_to_none(p.address_country)),
        // `descriptionPlain` est fourni a cote de `descriptionHtml` : pas d'aplatissement inutile.
        description: first_non_blank(&[job.description_plain.as_deref(), Some(&stripped)]),
        source_id: first_non_blank(&[job.id.as_deref(), apply_url.as_deref()]),
        apply_url,
        source: Some(JobSource::Ashby.as_str().to_string()),
        contract_type: trim_to_none(job.employment_type),
        // `isRemote` fait foi ; `workplaceType` ajoute « Hybrid », qui n'est pas du teletravail.
        remote: Some(
            job.is_remote.unwrap_or(false)
                || job.workplace_type.as_deref() == Some(ASHBY_REMOTE_WORKPLACE),
        ),
        published_at: job.published_at.as_deref().and_then(AtsDates::parse),
        ..Default::default()
    }
}

#[async_trait]
impl AtsConnector for AshbyConnector {
    fn supports(&self, url: &str) -> bool {
        url.contains("ashbyhq.com")
    }

    fn source(&self) -> JobSource {
        JobSource::Ashby
    }

    async fn fetch_jobs(&self, _: Option<&str>, _: Option<&str>) -> Result<Vec<JobOffer>, AppError> {
        Ok(Vec::new())
    }

    async fn fetch_by_slug(&self, slug: &str) -> Result<Vec<JobOffer>, AppError> {
        Ok(if slug.trim().is_empty() {
            Vec::new()
        } else {
            self.fetch_from_board(slug.trim()).await
        })
    }

    async fn fetch_from_url(&self, url: &str) -> Result<Vec<JobOffer>, AppError> {
        let board = Self::board_of(url);
        self.fetch_by_slug(&board).await
    }

    fn parse_ref(&self, url: &str) -> Option<PostingRef> {
        ASHBY_REF.captures(url).map(|captures| PostingRef {
            site: captures[1].to_string(),
            posting_id: captures[2].to_string(),
        })
    }
}

// ===========================================================================
// Workable
// ===========================================================================

const WORKABLE_API: &str = "https://apply.workable.com/api/v1/widget/accounts/";

/// Connecteur pour le widget public des comptes Workable.
pub struct WorkableConnector {
    client: Client,
    accounts: Vec<String>,
}

impl WorkableConnector {
    pub fn new(client: Client, accounts: Vec<String>) -> Self {
        Self { client, accounts }
    }

    pub async fn fetch_configured_accounts(&self) -> Vec<JobOffer> {
        let mut all = Vec::new();
        for account in &self.accounts {
            all.extend(self.fetch_from_account(account).await);
        }
        all
    }

    pub async fn fetch_from_account(&self, account: &str) -> Vec<JobOffer> {
        let url = format!("{WORKABLE_API}{account}?details=true");
        let Some(payload) = get_json::<WorkableAccount>(&self.client, "WORKABLE", &url, &[]).await
        else {
            return Vec::new();
        };

        let company = payload
            .name
            .filter(|name| !name.trim().is_empty())
            .unwrap_or_else(|| account.to_string());

        payload
            .jobs
            .unwrap_or_default()
            .into_iter()
            .map(|job| map_workable(job, &company))
            .collect()
    }

    /// Slug de compte depuis une URL d'annonce, de widget ou de sous-domaine carriere.
    fn account_of(url: &str) -> String {
        if let Some(captures) = WORKABLE_ACCOUNT_POSTING.captures(url) {
            // `/j/` est le chemin des annonces directes, pas un nom de compte.
            if &captures[1] != "j" {
                return captures[1].to_string();
            }
        }
        if url.contains("/accounts/") {
            return slug_after(Some(url), "/accounts/");
        }
        if url.contains("apply.workable.com/") {
            let slug = slug_after(Some(url), "apply.workable.com/");
            return if slug == "j" { String::new() } else { slug };
        }
        WORKABLE_SUBDOMAIN
            .captures(url)
            .map(|captures| captures[1].to_string())
            .filter(|sub| sub != "apply" && sub != "www")
            .unwrap_or_default()
    }
}

#[derive(Debug, Deserialize)]
struct WorkableAccount {
    name: Option<String>,
    #[serde(default)]
    jobs: Option<Vec<WorkableJob>>,
}

#[derive(Debug, Deserialize)]
struct WorkableJob {
    title: Option<String>,
    shortcode: Option<String>,
    employment_type: Option<String>,
    telecommuting: Option<bool>,
    url: Option<String>,
    shortlink: Option<String>,
    application_url: Option<String>,
    published_on: Option<String>,
    created_at: Option<String>,
    country: Option<String>,
    city: Option<String>,
    experience: Option<String>,
    #[serde(default)]
    locations: Option<Vec<WorkableLocation>>,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WorkableLocation {
    country: Option<String>,
    #[serde(rename = "countryCode")]
    country_code: Option<String>,
    city: Option<String>,
}

fn map_workable(job: WorkableJob, company: &str) -> JobOffer {
    let first_location = job.locations.as_ref().and_then(|l| l.first());
    // Les champs plats city/country refletent `locations[0]` ; on les prefere, puis on retombe
    // sur la liste quand ils sont vides.
    let location = join_location(&[job.city.as_deref(), job.country.as_deref()]).or_else(|| {
        first_location
            .and_then(|l| join_location(&[l.city.as_deref(), l.country.as_deref()]))
    });

    JobOffer {
        title: job.title.unwrap_or_default(),
        company: Some(company.to_string()),
        location,
        country: first_location.and_then(|l| l.country_code.clone()),
        description: non_empty(strip_html(job.description.as_deref())),
        apply_url: first_non_blank(&[
            job.application_url.as_deref(),
            job.url.as_deref(),
            job.shortlink.as_deref(),
        ]),
        source_id: trim_to_none(job.shortcode),
        source: Some(JobSource::Workable.as_str().to_string()),
        contract_type: trim_to_none(job.employment_type),
        experience_level: trim_to_none(job.experience),
        remote: Some(job.telecommuting.unwrap_or(false)),
        published_at: parse_first(&[job.published_on.as_deref(), job.created_at.as_deref()]),
        ..Default::default()
    }
}

#[async_trait]
impl AtsConnector for WorkableConnector {
    fn supports(&self, url: &str) -> bool {
        url.contains("workable.com")
    }

    fn source(&self) -> JobSource {
        JobSource::Workable
    }

    async fn fetch_jobs(&self, _: Option<&str>, _: Option<&str>) -> Result<Vec<JobOffer>, AppError> {
        Ok(Vec::new())
    }

    async fn fetch_by_slug(&self, slug: &str) -> Result<Vec<JobOffer>, AppError> {
        Ok(if slug.trim().is_empty() {
            Vec::new()
        } else {
            self.fetch_from_account(slug.trim()).await
        })
    }

    async fn fetch_from_url(&self, url: &str) -> Result<Vec<JobOffer>, AppError> {
        let account = Self::account_of(url);
        self.fetch_by_slug(&account).await
    }

    fn parse_ref(&self, url: &str) -> Option<PostingRef> {
        if let Some(captures) = WORKABLE_ACCOUNT_POSTING.captures(url) {
            if &captures[1] != "j" {
                return Some(PostingRef {
                    site: captures[1].to_string(),
                    posting_id: captures[2].to_string(),
                });
            }
        }
        WORKABLE_DIRECT_POSTING
            .captures(url)
            .map(|captures| PostingRef {
                site: String::new(),
                posting_id: captures[1].to_string(),
            })
    }
}

// ===========================================================================
// Recruitee
// ===========================================================================

/// Seules les offres a ce statut sont publiees.
const RECRUITEE_PUBLISHED: &str = "published";

/// Connecteur pour l'API publique des sites carriere Recruitee.
pub struct RecruiteeConnector {
    client: Client,
    companies: Vec<String>,
}

impl RecruiteeConnector {
    pub fn new(client: Client, companies: Vec<String>) -> Self {
        Self { client, companies }
    }

    pub async fn fetch_configured_companies(&self) -> Vec<JobOffer> {
        let mut all = Vec::new();
        for company in &self.companies {
            all.extend(self.fetch_from_company(company).await);
        }
        all
    }

    pub async fn fetch_from_company(&self, company: &str) -> Vec<JobOffer> {
        let url = format!("https://{company}.recruitee.com/api/offers/");
        let Some(payload) = get_json::<RecruiteeOffers>(&self.client, "RECRUITEE", &url, &[]).await
        else {
            return Vec::new();
        };

        payload
            .offers
            .unwrap_or_default()
            .into_iter()
            .filter(|offer| {
                offer
                    .status
                    .as_deref()
                    .is_some_and(|status| status.eq_ignore_ascii_case(RECRUITEE_PUBLISHED))
            })
            .map(|offer| map_recruitee(offer, company))
            .collect()
    }

    fn company_of(url: &str) -> String {
        RECRUITEE_SUBDOMAIN
            .captures(url)
            .map(|captures| captures[1].to_string())
            .filter(|sub| sub != "www")
            .unwrap_or_default()
    }
}

#[derive(Debug, Deserialize)]
struct RecruiteeOffers {
    #[serde(default)]
    offers: Option<Vec<RecruiteeOffer>>,
}

#[derive(Debug, Deserialize)]
struct RecruiteeOffer {
    id: Option<i64>,
    title: Option<String>,
    status: Option<String>,
    description: Option<String>,
    requirements: Option<String>,
    remote: Option<bool>,
    employment_type_code: Option<String>,
    experience_code: Option<String>,
    #[serde(default)]
    locations: Option<Vec<RecruiteeLocation>>,
    salary: Option<RecruiteeSalary>,
    careers_url: Option<String>,
    careers_apply_url: Option<String>,
    published_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RecruiteeLocation {
    city: Option<String>,
    country: Option<String>,
    country_code: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RecruiteeSalary {
    min: Option<i32>,
    max: Option<i32>,
    currency: Option<String>,
}

fn map_recruitee(offer: RecruiteeOffer, company: &str) -> JobOffer {
    let first_location = offer.locations.as_ref().and_then(|l| l.first());
    let (salary_min, salary_max, salary_currency) = offer
        .salary
        .map(|s| {
            (
                positive_or_none(s.min),
                positive_or_none(s.max),
                trim_to_none(s.currency),
            )
        })
        .unwrap_or((None, None, None));

    JobOffer {
        title: offer.title.unwrap_or_default(),
        company: Some(company.to_string()),
        location: first_location
            .and_then(|l| join_location(&[l.city.as_deref(), l.country.as_deref()])),
        country: first_location.and_then(|l| l.country_code.clone()),
        description: recruitee_description(offer.description.as_deref(), offer.requirements.as_deref()),
        apply_url: first_non_blank(&[
            offer.careers_apply_url.as_deref(),
            offer.careers_url.as_deref(),
        ]),
        source_id: offer.id.map(|id| id.to_string()),
        source: Some(JobSource::Recruitee.as_str().to_string()),
        contract_type: trim_to_none(offer.employment_type_code),
        experience_level: trim_to_none(offer.experience_code),
        remote: Some(offer.remote.unwrap_or(false)),
        published_at: offer.published_at.as_deref().and_then(AtsDates::parse),
        // La devise n'est conservee que si un montant l'accompagne : « EUR » seul n'apprend rien.
        salary_currency: (salary_min.is_some() || salary_max.is_some())
            .then_some(salary_currency)
            .flatten(),
        salary_min,
        salary_max,
        ..Default::default()
    }
}

/// Recruitee separe la description des prerequis : les concatener evite de perdre la moitie de
/// l'annonce dans le texte vectorise.
fn recruitee_description(description: Option<&str>, requirements: Option<&str>) -> Option<String> {
    let description = strip_html(description);
    let requirements = strip_html(requirements);

    match (description.is_empty(), requirements.is_empty()) {
        (true, true) => None,
        (true, false) => Some(requirements),
        (false, true) => Some(description),
        (false, false) => Some(format!("{description}\n\n{requirements}")),
    }
}

#[async_trait]
impl AtsConnector for RecruiteeConnector {
    fn supports(&self, url: &str) -> bool {
        url.contains("recruitee.com")
    }

    fn source(&self) -> JobSource {
        JobSource::Recruitee
    }

    async fn fetch_jobs(&self, _: Option<&str>, _: Option<&str>) -> Result<Vec<JobOffer>, AppError> {
        Ok(Vec::new())
    }

    async fn fetch_by_slug(&self, slug: &str) -> Result<Vec<JobOffer>, AppError> {
        Ok(if slug.trim().is_empty() {
            Vec::new()
        } else {
            self.fetch_from_company(slug.trim()).await
        })
    }

    async fn fetch_from_url(&self, url: &str) -> Result<Vec<JobOffer>, AppError> {
        let company = Self::company_of(url);
        self.fetch_by_slug(&company).await
    }

    fn parse_ref(&self, url: &str) -> Option<PostingRef> {
        RECRUITEE_OFFER_SLUG
            .captures(url)
            .map(|captures| PostingRef {
                site: Self::company_of(url),
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

/// Premiere date analysable de la liste. Une date de publication est une metadonnee : son absence
/// n'est jamais une raison d'ecarter une offre.
fn parse_first(candidates: &[Option<&str>]) -> Option<chrono::NaiveDateTime> {
    candidates
        .iter()
        .flatten()
        .find_map(|value| AtsDates::parse(value))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn client() -> Client {
        Client::new()
    }

    // --- Greenhouse -----------------------------------------------------------------------

    #[test]
    fn greenhouse_parses_both_posting_url_shapes() {
        let connector = GreenhouseConnector::new(client());
        let path = connector
            .parse_ref("https://boards.greenhouse.io/acme/jobs/4012345")
            .expect("URL de board");
        assert_eq!(path.site, "acme");
        assert_eq!(path.posting_id, "4012345");

        // La forme `?gh_jid=` n'expose pas le board : seul l'identifiant est connu.
        let query = connector
            .parse_ref("https://acme.com/careers?gh_jid=987")
            .expect("URL avec gh_jid");
        assert!(query.site.is_empty());
        assert_eq!(query.posting_id, "987");

        assert!(connector.parse_ref("https://example.com/emploi").is_none());
    }

    #[test]
    fn greenhouse_mapping_unescapes_the_html_content() {
        let job = GreenhouseJob {
            id: Some(42),
            title: Some("Dev".into()),
            content: Some("<p>R&amp;D</p>".into()),
            location: Some(GreenhouseLocation { name: Some(" Remote ".into()) }),
            absolute_url: Some("https://boards.greenhouse.io/acme/jobs/42".into()),
            updated_at: None,
            first_published: Some("2026-08-01T10:00:00Z".into()),
        };
        let offer = map_greenhouse(job, "acme");

        assert_eq!(offer.description.as_deref(), Some("R&D"));
        assert_eq!(offer.company.as_deref(), Some("acme"));
        assert_eq!(offer.location.as_deref(), Some("Remote"));
        // Le lieu « Remote » doit declencher la detection, la source n'ayant pas de drapeau.
        assert_eq!(offer.remote, Some(true));
        assert_eq!(offer.source_id.as_deref(), Some("42"));
        assert!(offer.published_at.is_some());
    }

    // --- Lever ----------------------------------------------------------------------------

    #[test]
    fn lever_prefers_the_plain_description_over_the_html_one() {
        let posting = LeverPosting {
            id: Some("abc".into()),
            text: Some("Dev".into()),
            description_plain: Some("Texte plat".into()),
            description: Some("<p>Balise</p>".into()),
            apply_url: None,
            hosted_url: Some("https://jobs.lever.co/acme/abc".into()),
            created_at: Some(1_760_000_000_000),
            categories: Some(LeverCategories {
                location: Some("Paris".into()),
                commitment: Some("Full-time".into()),
            }),
        };
        let offer = map_lever(posting, "acme");

        assert_eq!(offer.description.as_deref(), Some("Texte plat"));
        assert_eq!(offer.contract_type.as_deref(), Some("Full-time"));
        // `hostedUrl` sert de repli quand `applyUrl` est absent.
        assert_eq!(
            offer.apply_url.as_deref(),
            Some("https://jobs.lever.co/acme/abc")
        );
        // createdAt est en millisecondes : une lecture en secondes daterait l'offre en 57 000.
        let published = offer.published_at.expect("date");
        assert_eq!(published.date().to_string(), "2025-10-09");
    }

    #[test]
    fn lever_ref_requires_a_uuid_posting_id() {
        let connector = LeverConnector::new(client());
        assert!(connector
            .parse_ref("https://jobs.lever.co/acme/0e6b1a2c-3d4e-5f60-7181-92a3b4c5d6e7")
            .is_some());
        // Un segment qui n'est pas un UUID n'est pas une annonce : c'est la page du board.
        assert!(connector.parse_ref("https://jobs.lever.co/acme/engineering").is_none());
    }

    // --- SmartRecruiters ------------------------------------------------------------------

    #[test]
    fn smart_recruiters_uses_its_explicit_remote_flag() {
        let posting = SmartRecruitersPosting {
            id: Some("77".into()),
            name: Some("Ingenieur".into()),
            released_date: Some("2026-08-01T00:00:00Z".into()),
            location: Some(SmartRecruitersLocation {
                city: Some("Noumea".into()),
                country: Some("NC".into()),
                // Le titre ne contient aucun marqueur : seul le drapeau peut trancher.
                remote: Some(true),
            }),
            type_of_employment: Some(SmartRecruitersEmployment { label: Some("CDI".into()) }),
        };
        let offer = map_smart_recruiters(posting, "acme");

        assert_eq!(offer.remote, Some(true));
        assert_eq!(offer.country.as_deref(), Some("NC"));
        assert_eq!(
            offer.apply_url.as_deref(),
            Some("https://jobs.smartrecruiters.com/acme/77")
        );
    }

    // --- Ashby ----------------------------------------------------------------------------

    #[test]
    fn ashby_treats_hybrid_as_not_remote() {
        let hybrid = AshbyJob {
            id: Some("1".into()),
            title: Some("Dev".into()),
            location: Some("Paris".into()),
            is_listed: Some(true),
            is_remote: Some(false),
            workplace_type: Some("Hybrid".into()),
            employment_type: None,
            description_html: None,
            description_plain: Some("Texte".into()),
            published_at: None,
            address: None,
            job_url: Some("https://jobs.ashbyhq.com/acme/1".into()),
            apply_url: None,
        };
        assert_eq!(map_ashby(hybrid, "acme").remote, Some(false));

        let remote = AshbyJob {
            id: Some("2".into()),
            title: Some("Dev".into()),
            location: None,
            is_listed: Some(true),
            is_remote: Some(false),
            workplace_type: Some("Remote".into()),
            employment_type: None,
            description_html: None,
            description_plain: None,
            published_at: None,
            address: None,
            job_url: None,
            apply_url: None,
        };
        assert_eq!(map_ashby(remote, "acme").remote, Some(true));
    }

    #[test]
    fn ashby_board_is_read_from_both_url_shapes() {
        assert_eq!(AshbyConnector::board_of("https://jobs.ashbyhq.com/acme"), "acme");
        assert_eq!(
            AshbyConnector::board_of("https://api.ashbyhq.com/posting-api/job-board/acme"),
            "acme"
        );
    }

    // --- Workable -------------------------------------------------------------------------

    #[test]
    fn workable_account_is_not_confused_with_the_posting_path() {
        // `apply.workable.com/j/CODE` n'a pas de compte dans l'URL : « j » ne doit pas etre pris
        // pour un nom de compte, sinon on interrogerait un board inexistant.
        assert_eq!(WorkableConnector::account_of("https://apply.workable.com/j/ABC123"), "");
        assert_eq!(
            WorkableConnector::account_of("https://apply.workable.com/acme/j/ABC123"),
            "acme"
        );
        assert_eq!(WorkableConnector::account_of("https://acme.workable.com/"), "acme");
        // Les sous-domaines techniques ne sont pas des comptes.
        assert_eq!(WorkableConnector::account_of("https://www.workable.com/"), "");
    }

    #[test]
    fn workable_parse_ref_skips_the_j_segment() {
        let connector = WorkableConnector::new(client(), vec![]);
        let direct = connector
            .parse_ref("https://apply.workable.com/j/ABC123")
            .expect("annonce directe");
        assert!(direct.site.is_empty());
        assert_eq!(direct.posting_id, "ABC123");

        let with_account = connector
            .parse_ref("https://apply.workable.com/acme/j/ABC123")
            .expect("annonce avec compte");
        assert_eq!(with_account.site, "acme");
    }

    #[test]
    fn workable_falls_back_to_the_locations_list_for_the_place() {
        let job = WorkableJob {
            title: Some("Dev".into()),
            shortcode: Some("ABC".into()),
            employment_type: None,
            telecommuting: Some(false),
            url: None,
            shortlink: None,
            application_url: None,
            published_on: None,
            created_at: None,
            // Champs plats vides : la liste doit prendre le relais.
            country: None,
            city: None,
            experience: None,
            locations: Some(vec![WorkableLocation {
                country: Some("France".into()),
                country_code: Some("FR".into()),
                city: Some("Lyon".into()),
            }]),
            description: None,
        };
        let offer = map_workable(job, "ACME");
        assert_eq!(offer.location.as_deref(), Some("Lyon, France"));
        assert_eq!(offer.country.as_deref(), Some("FR"));
    }

    // --- Recruitee ------------------------------------------------------------------------

    #[test]
    fn recruitee_concatenates_description_and_requirements() {
        let combined = recruitee_description(Some("<p>Mission</p>"), Some("<p>Prerequis</p>"));
        assert_eq!(combined.as_deref(), Some("Mission\n\nPrerequis"));
        // Un seul des deux champs suffit ; aucun donne `None`, pas une chaine vide.
        assert_eq!(recruitee_description(Some("Mission"), None).as_deref(), Some("Mission"));
        assert_eq!(recruitee_description(None, Some("Prerequis")).as_deref(), Some("Prerequis"));
        assert_eq!(recruitee_description(None, None), None);
    }

    #[test]
    fn recruitee_drops_a_currency_with_no_amount() {
        let offer = RecruiteeOffer {
            id: Some(5),
            title: Some("Dev".into()),
            status: Some("published".into()),
            description: None,
            requirements: None,
            remote: Some(true),
            employment_type_code: None,
            experience_code: None,
            locations: None,
            // `0` est le « non renseigne » de Recruitee : la devise seule serait trompeuse.
            salary: Some(RecruiteeSalary {
                min: Some(0),
                max: Some(0),
                currency: Some("EUR".into()),
            }),
            careers_url: None,
            careers_apply_url: None,
            published_at: None,
        };
        let mapped = map_recruitee(offer, "acme");
        assert_eq!(mapped.salary_min, None);
        assert_eq!(mapped.salary_max, None);
        assert_eq!(mapped.salary_currency, None);
    }

    #[test]
    fn recruitee_company_comes_from_the_subdomain() {
        assert_eq!(
            RecruiteeConnector::company_of("https://acme.recruitee.com/o/dev-rust"),
            "acme"
        );
        assert_eq!(RecruiteeConnector::company_of("https://www.recruitee.com/"), "");
    }

    // --- Contrat commun -------------------------------------------------------------------

    #[tokio::test]
    async fn board_connectors_return_nothing_for_a_keyword_search() {
        // Aucun de ces ATS n'a d'endpoint de recherche transverse : promettre le contraire
        // ferait croire a une source vide alors qu'elle n'a jamais ete interrogee.
        let greenhouse = GreenhouseConnector::new(client());
        let lever = LeverConnector::new(client());
        let ashby = AshbyConnector::new(client(), vec![]);

        assert!(greenhouse.fetch_jobs(Some("rust"), None).await.unwrap().is_empty());
        assert!(lever.fetch_jobs(Some("rust"), None).await.unwrap().is_empty());
        assert!(ashby.fetch_jobs(Some("rust"), None).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn blank_slugs_are_rejected_without_a_network_call() {
        // Un slug vide construirait une URL comme `.../boards//jobs`, qui renvoie une erreur.
        let connector = GreenhouseConnector::new(client());
        assert!(connector.fetch_by_slug("   ").await.unwrap().is_empty());
    }

    #[test]
    fn supports_recognises_each_host() {
        assert!(GreenhouseConnector::new(client()).supports("https://boards.greenhouse.io/acme"));
        assert!(LeverConnector::new(client()).supports("https://jobs.lever.co/acme"));
        assert!(SmartRecruitersConnector::new(client())
            .supports("https://jobs.smartrecruiters.com/acme/1"));
        assert!(AshbyConnector::new(client(), vec![]).supports("https://jobs.ashbyhq.com/acme"));
        assert!(WorkableConnector::new(client(), vec![]).supports("https://apply.workable.com/acme"));
        assert!(RecruiteeConnector::new(client(), vec![]).supports("https://acme.recruitee.com"));

        assert!(!GreenhouseConnector::new(client()).supports("https://jobs.lever.co/acme"));
    }
}
