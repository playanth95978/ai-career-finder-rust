//! Scoring deterministe profil <-> offres, port du `JobMatchingService` de l'application Spring.
//!
//! Ecart assume : la version Java resout les lieux via un agent LLM (`GeoLocationService`) qui
//! connait les agglomerations et les regions. Ici la comparaison est purement textuelle
//! (egalite, inclusion, meme pays). Les tags produits et la ponderation finale sont identiques,
//! donc le front n'a rien a adapter ; seule la finesse du score de localisation regresse.

use std::collections::HashSet;

use crate::dto::job_copilot_dto::{match_tag, JobOfferSummaryDto, MatchResultDto, MatchTagDto};
use crate::dto::parse_string_list;
use crate::models::{CandidateProfile, JobOffer, UserPreference};

/// Ponderation finale, alignee sur la version Java : la pertinence semantique domine parce que
/// c'est le signal le plus fiable quand les metadonnees structurees (skills, salaire) sont
/// absentes — cas ou toutes les heuristiques retombent a 0.5 et n'ordonnent plus rien.
const WEIGHT_RELEVANCE: f64 = 0.45;
const WEIGHT_SKILLS: f64 = 0.35;
const WEIGHT_LOCATION: f64 = 0.10;
const WEIGHT_REMOTE: f64 = 0.05;
const WEIGHT_SALARY: f64 = 0.05;

/// Seuil au-dela duquel l'offre porte le tag « correspondance forte ».
const STRONG_MATCH_RELEVANCE: f64 = 0.8;

pub struct JobMatchingService;

impl JobMatchingService {
    /// Score chaque offre et renvoie la liste triee par score decroissant.
    ///
    /// `offers` **doit** etre ordonnee par pertinence de recherche : la position alimente la
    /// composante semantique du score, qui est ce qui differencie les offres.
    pub fn match_jobs(
        profile: &CandidateProfile,
        offers: &[JobOffer],
        preferences: Option<&UserPreference>,
        search_location: Option<&str>,
    ) -> Vec<MatchResultDto> {
        let candidate_skills = lowercase_set(&parse_string_list(profile.skills.as_deref()));
        let references = Self::reference_locations(profile, preferences, search_location);

        let total = offers.len();
        let mut scored: Vec<MatchResultDto> = offers
            .iter()
            .enumerate()
            .map(|(rank, offer)| {
                let relevance = Self::rank_relevance(rank, total);
                Self::score_offer(&candidate_skills, offer, preferences, relevance, &references)
            })
            .collect();

        // `total_cmp` plutot que `partial_cmp().unwrap()` : un NaN venant d'un salaire aberrant
        // ferait paniquer le handler au lieu de simplement mal classer une offre.
        scored.sort_by(|a, b| b.score.total_cmp(&a.score));
        scored
    }

    /// Lieux de reference d'une session de match. Un `search_location` explicite gagne seul :
    /// l'utilisateur regarde ailleurs que la ou son CV le situe.
    fn reference_locations(
        profile: &CandidateProfile,
        preferences: Option<&UserPreference>,
        search_location: Option<&str>,
    ) -> Vec<String> {
        if let Some(searched) = search_location.map(str::trim).filter(|s| !s.is_empty()) {
            return vec![searched.to_lowercase()];
        }

        let mut refs = Vec::new();
        if let Some(location) = profile.location.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            refs.push(location.to_lowercase());
        }
        if let Some(prefs) = preferences {
            for location in parse_string_list(prefs.preferred_locations.as_deref()) {
                let trimmed = location.trim();
                if !trimmed.is_empty() {
                    refs.push(trimmed.to_lowercase());
                }
            }
        }
        refs
    }

    /// Projette un rang 0-base sur une pertinence continue dans `[0.4, 1.0]` : le premier resultat
    /// vaut 1.0, le dernier 0.4. Le plancher a 0.4 evite de punir une queue de liste encore
    /// pertinente, tandis que l'etalement casse l'egalite « tout le monde a 50 % ».
    fn rank_relevance(rank: usize, total: usize) -> f64 {
        if total <= 1 {
            return 1.0;
        }
        1.0 - 0.6 * (rank as f64 / (total - 1) as f64)
    }

    fn score_offer(
        candidate_skills: &HashSet<String>,
        offer: &JobOffer,
        preferences: Option<&UserPreference>,
        relevance: f64,
        references: &[String],
    ) -> MatchResultDto {
        let mut strengths = Vec::new();
        let mut weaknesses = Vec::new();

        let skill_score = Self::skill_score(candidate_skills, offer, &mut strengths, &mut weaknesses);
        let location_score = Self::location_score(references, offer, &mut strengths, &mut weaknesses);
        let remote_score = Self::remote_score(offer, preferences, &mut strengths, &mut weaknesses);
        let salary_score = Self::salary_score(offer, preferences, &mut strengths, &mut weaknesses);

        if relevance >= STRONG_MATCH_RELEVANCE {
            strengths.push(MatchTagDto::of(match_tag::STRONG_MATCH));
        }

        let raw = relevance * WEIGHT_RELEVANCE
            + skill_score * WEIGHT_SKILLS
            + location_score * WEIGHT_LOCATION
            + remote_score * WEIGHT_REMOTE
            + salary_score * WEIGHT_SALARY;

        MatchResultDto {
            job_offer: JobOfferSummaryDto::from(offer),
            score: (raw * 100.0).round() / 100.0,
            strengths,
            weaknesses,
        }
    }

    /// Part des competences demandees que le candidat possede. Une offre sans competences listees
    /// vaut 0.5 : on ne sait pas, ce n'est ni un point fort ni un point faible.
    fn skill_score(
        candidate: &HashSet<String>,
        offer: &JobOffer,
        strengths: &mut Vec<MatchTagDto>,
        weaknesses: &mut Vec<MatchTagDto>,
    ) -> f64 {
        let job_skills = parse_string_list(offer.skills.as_deref());
        let job_skills = lowercase_set(&job_skills);
        if job_skills.is_empty() {
            return 0.5;
        }

        let mut matched = 0usize;
        for skill in &job_skills {
            if candidate.contains(skill) {
                matched += 1;
                strengths.push(MatchTagDto::with_skill(match_tag::MATCHED_SKILL, skill));
            } else {
                weaknesses.push(MatchTagDto::with_skill(match_tag::MISSING_SKILL, skill));
            }
        }

        matched as f64 / job_skills.len() as f64
    }

    /// Score de localisation textuel : ville identique 1.0, l'une contenue dans l'autre 0.8,
    /// remote 0.85, meme pays 0.5, lieu inconnu 0.4, pays different 0.15.
    fn location_score(
        references: &[String],
        offer: &JobOffer,
        strengths: &mut Vec<MatchTagDto>,
        weaknesses: &mut Vec<MatchTagDto>,
    ) -> f64 {
        if references.is_empty() {
            return 0.5;
        }

        let offer_location = offer
            .location
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_lowercase);

        let Some(offer_location) = offer_location else {
            // Pas de lieu sur l'offre : le remote reste un signal exploitable.
            if offer.remote.unwrap_or(false) {
                strengths.push(MatchTagDto::of(match_tag::REMOTE));
                return 0.85;
            }
            return 0.4;
        };

        let mut best = 0.0f64;
        for reference in references {
            let score = if reference == &offer_location {
                1.0
            } else if offer_location.contains(reference.as_str()) || reference.contains(offer_location.as_str()) {
                0.8
            } else {
                0.15
            };
            if score > best {
                best = score;
            }
        }

        // Le remote rattrape un lieu qui ne correspond pas : l'offre reste tenable.
        if best < 0.85 && offer.remote.unwrap_or(false) {
            strengths.push(MatchTagDto::of(match_tag::REMOTE));
            return 0.85;
        }

        if best >= 1.0 {
            strengths.push(MatchTagDto::of(match_tag::LOCATION_MATCH));
        } else if best >= 0.8 {
            strengths.push(MatchTagDto::of(match_tag::PREFERRED_LOCATION));
        } else {
            weaknesses.push(MatchTagDto::of(match_tag::LOCATION_MISMATCH));
        }
        best
    }

    /// Neutre (0.5) tant que l'utilisateur n'a pas declare vouloir du remote exclusivement :
    /// sans preference exprimee, une offre sur site n'est pas un defaut.
    fn remote_score(
        offer: &JobOffer,
        preferences: Option<&UserPreference>,
        strengths: &mut Vec<MatchTagDto>,
        weaknesses: &mut Vec<MatchTagDto>,
    ) -> f64 {
        let remote_only = preferences.and_then(|p| p.remote_only).unwrap_or(false);
        if !remote_only {
            return 0.5;
        }
        if offer.remote.unwrap_or(false) {
            strengths.push(MatchTagDto::of(match_tag::REMOTE));
            return 1.0;
        }
        weaknesses.push(MatchTagDto::of(match_tag::NOT_REMOTE));
        0.0
    }

    /// Neutre tant qu'il manque un des deux cotes de la comparaison (plancher souhaite ou haut de
    /// fourchette annonce) : la plupart des offres ne publient pas de salaire.
    fn salary_score(
        offer: &JobOffer,
        preferences: Option<&UserPreference>,
        strengths: &mut Vec<MatchTagDto>,
        weaknesses: &mut Vec<MatchTagDto>,
    ) -> f64 {
        let Some(wanted) = preferences.and_then(|p| p.salary_min) else {
            return 0.5;
        };
        let Some(offered) = offer.salary_max else {
            return 0.5;
        };

        if offered >= wanted {
            strengths.push(MatchTagDto::of(match_tag::SALARY_COMPATIBLE));
            1.0
        } else {
            weaknesses.push(MatchTagDto::of(match_tag::SALARY_BELOW));
            0.2
        }
    }
}

fn lowercase_set(values: &[String]) -> HashSet<String> {
    values
        .iter()
        .map(|v| v.trim().to_lowercase())
        .filter(|v| !v.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn offer(title: &str) -> JobOffer {
        JobOffer {
            title: title.to_string(),
            ..Default::default()
        }
    }

    fn profile() -> CandidateProfile {
        CandidateProfile {
            id: uuid::Uuid::nil(),
            user_id: "alice".to_string(),
            full_name: None,
            email: None,
            location: None,
            years_of_experience: None,
            skills: None,
            experiences: None,
            preferred_roles: None,
            languages: None,
            education: None,
            certifications: None,
            raw_markdown: None,
            cv_filename: None,
            embedding_model: None,
            embedded_at: None,
            created_at: None,
            updated_at: None,
            created_by: None,
            created_date: None,
            last_modified_by: None,
            last_modified_date: None,
        }
    }

    fn preferences() -> UserPreference {
        UserPreference {
            id: uuid::Uuid::nil(),
            user_id: "alice".to_string(),
            remote_only: None,
            contract_type: None,
            salary_min: None,
            salary_max: None,
            preferred_roles: None,
            excluded_technologies: None,
            preferred_locations: None,
            created_by: None,
            created_date: None,
            last_modified_by: None,
            last_modified_date: None,
        }
    }

    #[test]
    fn rank_relevance_spans_one_down_to_the_floor() {
        // Un seul resultat : pas de rang a etaler, il vaut le maximum.
        assert_eq!(JobMatchingService::rank_relevance(0, 1), 1.0);
        assert_eq!(JobMatchingService::rank_relevance(0, 5), 1.0);
        assert!((JobMatchingService::rank_relevance(4, 5) - 0.4).abs() < 1e-9);
    }

    #[test]
    fn results_are_sorted_by_score_descending() {
        let mut wanted = offer("Dev Rust");
        wanted.skills = Some(r#"["rust"]"#.to_string());
        let mut unwanted = offer("Dev COBOL");
        unwanted.skills = Some(r#"["cobol"]"#.to_string());

        let mut p = profile();
        p.skills = Some(r#"["rust"]"#.to_string());

        // L'offre pertinente est volontairement placee en second dans l'entree : c'est le score,
        // pas l'ordre d'arrivee, qui doit decider du classement final.
        let results = JobMatchingService::match_jobs(&p, &[unwanted, wanted], None, None);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].job_offer.title, "Dev Rust");
        assert!(results[0].score > results[1].score);
    }

    #[test]
    fn matched_and_missing_skills_are_tagged() {
        let mut o = offer("Dev");
        o.skills = Some(r#"["rust","kubernetes"]"#.to_string());
        let mut p = profile();
        p.skills = Some(r#"["Rust"]"#.to_string());

        let results = JobMatchingService::match_jobs(&p, &[o], None, None);
        let strengths = &results[0].strengths;
        let weaknesses = &results[0].weaknesses;

        // La comparaison est insensible a la casse : « Rust » du CV couvre « rust » de l'offre.
        assert!(strengths.iter().any(|t| t.key == match_tag::MATCHED_SKILL && t.skill.as_deref() == Some("rust")));
        assert!(weaknesses.iter().any(|t| t.key == match_tag::MISSING_SKILL && t.skill.as_deref() == Some("kubernetes")));
    }

    #[test]
    fn remote_only_preference_penalises_onsite_offers() {
        let mut onsite = offer("Sur site");
        onsite.remote = Some(false);
        let mut prefs = preferences();
        prefs.remote_only = Some(true);

        let results = JobMatchingService::match_jobs(&profile(), &[onsite], Some(&prefs), None);
        assert!(results[0].weaknesses.iter().any(|t| t.key == match_tag::NOT_REMOTE));
    }

    #[test]
    fn salary_below_expectation_is_flagged() {
        let mut low = offer("Mal paye");
        low.salary_max = Some(1_000);
        let mut prefs = preferences();
        prefs.salary_min = Some(5_000);

        let results = JobMatchingService::match_jobs(&profile(), &[low], Some(&prefs), None);
        assert!(results[0].weaknesses.iter().any(|t| t.key == match_tag::SALARY_BELOW));
    }

    #[test]
    fn search_location_overrides_the_cv_location() {
        let mut o = offer("Dev");
        o.location = Some("Noumea".to_string());
        let mut p = profile();
        p.location = Some("Paris".to_string());

        // Sans lieu de recherche, l'offre est loin du CV.
        let default_run = JobMatchingService::match_jobs(&p, std::slice::from_ref(&o), None, None);
        assert!(default_run[0].weaknesses.iter().any(|t| t.key == match_tag::LOCATION_MISMATCH));

        // Le lieu tape dans la barre de recherche devient la seule reference.
        let searched = JobMatchingService::match_jobs(&p, &[o], None, Some("Noumea"));
        assert!(searched[0].strengths.iter().any(|t| t.key == match_tag::LOCATION_MATCH));
    }

    #[test]
    fn missing_metadata_stays_neutral_rather_than_penalising() {
        // Offre sans competences, sans lieu, sans salaire et sans preferences : le score doit
        // refleter la seule pertinence de rang, pas une accumulation de zeros.
        let results = JobMatchingService::match_jobs(&profile(), &[offer("Inconnue")], None, None);
        assert!(results[0].weaknesses.is_empty());
        assert!(results[0].score > 0.5);
    }
}
