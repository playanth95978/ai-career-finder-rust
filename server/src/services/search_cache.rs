//! Caches en memoire du chemin de recherche.
//!
//! Deux caches, parce que les deux valeurs n'ont pas du tout la meme duree de validite :
//!
//! * **l'embedding d'une requete** ne perime jamais. Le texte « developpeur rust » produit toujours
//!   le meme vecteur tant que le modele ne change pas. Duree de vie longue, donc, et le seul risque
//!   est de changer `OLLAMA_EMBEDDING_MODEL` sans vider le cache — d'ou le nom du modele dans la
//!   cle ;
//! * **les resultats d'une recherche** perime vite. L'ingestion tourne chaque nuit et les offres
//!   expirent en continu ; garder une page de resultats plusieurs jours afficherait des annonces
//!   mortes et masquerait les nouvelles. Le cache ne sert ici qu'a absorber les pics — plusieurs
//!   personnes cherchant « developpeur » dans la meme minute — pas a conserver.
//!
//! # Pourquoi en memoire et pas Redis
//!
//! Redis n'apporte quelque chose qu'a partir de deux instances, pour qu'elles partagent leurs
//! entrees. En dessous, il ajoute un aller-retour reseau et un service a exploiter pour rendre le
//! meme service qu'une `HashMap`. Les deux caches sont donc locaux au processus, derriere une
//! interface qui pourra etre rebranchee sur Redis sans toucher aux appelants.
//!
//! Les deux sont **bornes en nombre d'entrees**, pas seulement en duree : un cache dont la taille
//! ne depend que du TTL grandit avec le trafic, et c'est exactement sous charge qu'on ne veut pas
//! d'une consommation memoire imprevisible.

use std::sync::OnceLock;
use std::time::Duration;

use moka::sync::Cache;

use crate::models::JobOffer;

/// Duree de vie d'un embedding de requete.
///
/// Longue a dessein : la valeur est deterministe. Une semaine borne surtout la memoire — les
/// requetes rares finissent par sortir — et donne une limite haute a la persistance d'un vecteur
/// calcule par un modele qu'on aurait remplace sans vider le cache.
const EMBEDDING_TTL: Duration = Duration::from_secs(7 * 24 * 3600);

/// Nombre d'embeddings de requetes conserves.
///
/// Chaque entree pese 768 flottants, soit environ 3 Ko : dix mille entrees tiennent dans une
/// trentaine de megaoctets, ce qui reste negligeable devant le modele de reclassement.
const EMBEDDING_CAPACITY: u64 = 10_000;

/// Duree de vie d'une page de resultats.
///
/// Cinq minutes : assez pour absorber un pic sur une requete populaire, trop court pour qu'un
/// utilisateur voie un corpus perime. L'ingestion nocturne doit de toute facon appeler
/// [`invalidate_results`] en fin de passage.
const RESULTS_TTL: Duration = Duration::from_secs(300);

/// Nombre de pages de resultats conservees.
///
/// Une page est une dizaine d'offres avec leur description, soit quelques dizaines de kilooctets.
/// Mille pages plafonnent donc l'empreinte a quelques dizaines de megaoctets.
const RESULTS_CAPACITY: u64 = 1_000;

static EMBEDDINGS: OnceLock<Cache<String, Vec<f32>>> = OnceLock::new();
static RESULTS: OnceLock<Cache<String, Vec<JobOffer>>> = OnceLock::new();

fn embeddings() -> &'static Cache<String, Vec<f32>> {
    EMBEDDINGS.get_or_init(|| {
        Cache::builder()
            .max_capacity(EMBEDDING_CAPACITY)
            .time_to_live(EMBEDDING_TTL)
            .build()
    })
}

fn results() -> &'static Cache<String, Vec<JobOffer>> {
    RESULTS.get_or_init(|| {
        Cache::builder()
            .max_capacity(RESULTS_CAPACITY)
            .time_to_live(RESULTS_TTL)
            .build()
    })
}

/// Embedding deja calcule pour cette requete, s'il est connu.
pub fn cached_embedding(model: &str, query: &str) -> Option<Vec<f32>> {
    embeddings().get(&embedding_key(model, query))
}

pub fn store_embedding(model: &str, query: &str, vector: &[f32]) {
    embeddings().insert(embedding_key(model, query), vector.to_vec());
}

/// Page de resultats deja calculee pour ces criteres, si elle est encore valide.
pub fn cached_results(query: &str, source: Option<&str>, limit: i64) -> Option<Vec<JobOffer>> {
    results().get(&results_key(query, source, limit))
}

pub fn store_results(query: &str, source: Option<&str>, limit: i64, offers: &[JobOffer]) {
    results().insert(results_key(query, source, limit), offers.to_vec());
}

/// Vide le cache de resultats. A appeler apres une ingestion : le corpus a change.
///
/// Les embeddings ne sont pas touches — de nouvelles offres ne changent rien au vecteur d'une
/// requete.
pub fn invalidate_results() {
    results().invalidate_all();
    tracing::info!("Cache de resultats de recherche vide");
}

/// Le modele fait partie de la cle : changer `OLLAMA_EMBEDDING_MODEL` doit invalider les vecteurs
/// deja calcules, sans quoi la recherche comparerait des vecteurs de deux espaces differents.
fn embedding_key(model: &str, query: &str) -> String {
    format!("{model}\u{1f}{}", query.trim())
}

/// Tous les parametres qui changent le resultat entrent dans la cle.
///
/// `limit` en fait partie : la diversification et le reclassement operent sur la page rendue, donc
/// une recherche a dix resultats n'est pas le prefixe d'une recherche a cinquante. Le separateur
/// est un caractere de controle, absent des requetes utilisateur, pour qu'une requete contenant le
/// separateur ne puisse pas se faire passer pour une autre cle.
fn results_key(query: &str, source: Option<&str>, limit: i64) -> String {
    format!(
        "{}\u{1f}{}\u{1f}{limit}",
        query.trim(),
        source.map(str::trim).unwrap_or("")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_cle_d_embedding_distingue_les_modeles() {
        // Deux modeles produisent des vecteurs incomparables : les confondre reviendrait a chercher
        // dans un espace avec les coordonnees d'un autre.
        assert_ne!(
            embedding_key("nomic-embed-text", "developpeur"),
            embedding_key("mxbai-embed-large", "developpeur")
        );
    }

    #[test]
    fn la_cle_d_embedding_ignore_les_espaces_de_bord() {
        assert_eq!(
            embedding_key("m", " developpeur "),
            embedding_key("m", "developpeur")
        );
    }

    #[test]
    fn la_cle_de_resultats_distingue_source_et_limite() {
        let base = results_key("developpeur", None, 10);
        assert_ne!(base, results_key("developpeur", Some("ADZUNA"), 10));
        assert_ne!(base, results_key("developpeur", None, 50));
    }

    #[test]
    fn la_cle_de_resultats_ne_confond_pas_des_criteres_concatenes() {
        // Sans separateur distinct, ("dev x", None) et ("dev", Some("x")) donneraient la meme cle,
        // et une recherche filtree par source servirait le resultat d'une recherche libre.
        assert_ne!(
            results_key("dev x", None, 10),
            results_key("dev", Some("x"), 10)
        );
    }

    #[test]
    fn un_embedding_stocke_est_relu() {
        store_embedding("modele-test", "requete-test", &[0.1, 0.2, 0.3]);
        assert_eq!(
            cached_embedding("modele-test", "requete-test"),
            Some(vec![0.1, 0.2, 0.3])
        );
        assert_eq!(cached_embedding("modele-test", "jamais-vue"), None);
    }
}
