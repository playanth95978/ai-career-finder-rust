//! Reranking cross-encoder ONNX, portage du `OnnxRerankerService` de l'application Spring.
//!
//! La recherche vectorielle repond « quels documents ressemblent a la requete » ; un cross-encoder
//! repond « lequel repond a la requete ». Il lit la paire (requete, document) ensemble au lieu de
//! comparer deux vecteurs calcules separement, ce qui corrige les inversions de classement que le
//! cosinus laisse passer — precisement le point faible mesure sur ce corpus, ou la fenetre entre
//! pertinent et hors-sujet ne fait que quelques centiemes.
//!
//! # Choix d'implementation
//!
//! `fastembed::TextRerank` plutot que `ort` brut : il empaquette le tokenizer HuggingFace, la
//! session ONNX et le decoupage en lots. Le modele est charge depuis le disque
//! (`try_new_from_user_defined`), ce qui permet de reutiliser les fichiers INT8 deja telecharges
//! par le script de l'application Java — meme modele, donc classements comparables entre les deux
//! backends.
//!
//! # Degradation
//!
//! Le reranking est un **raffinement**, pas une condition de la recherche : modele absent, fichier
//! corrompu ou inference en echec renvoient l'ordre d'entree inchange, apres journalisation. Une
//! recherche qui repond moins bien vaut mieux qu'une recherche qui ne repond plus.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use fastembed::{
    OnnxSource, RerankInitOptionsUserDefined, TextRerank, TokenizerFiles,
    UserDefinedRerankingModel,
};

/// Repertoire par defaut des fichiers du modele, identique a la version Java
/// (`reranker.model-path: scripts/rerankers/bge-base/model.onnx`).
const DEFAULT_MODEL_DIR: &str = "scripts/rerankers/bge-base";

/// Troncature du cross-encoder, en jetons.
///
/// 128 et non les 512 du modele : valeur mesuree cote Java sur ce meme `bge-reranker-base`, ou
/// passer de 256 a 128 rapporte environ 1,8x pour une correlation de rangs de 0,83 avec le
/// classement a 256, le top-1 restant le plus souvent identique. 128 jetons couvrent titre,
/// entreprise, contrat et le debut de la description — assez pour juger la pertinence d'une offre.
///
/// Ne pas descendre a 64 : le gain devient marginal et le classement s'effondre.
///
/// A noter : `fastembed` fixe cette longueur a l'initialisation du modele, pas par appel. Reclasser
/// des documents nettement plus longs demanderait une seconde instance.
const MAX_LENGTH: usize = 128;

/// Taille de lot d'inference.
///
/// Volontairement plus basse que les 256 par defaut de fastembed : un lot est materialise en
/// memoire sous forme de tenseurs `512 * taille_du_lot`, et on ne reclasse jamais plus de quelques
/// dizaines d'offres — un lot de 256 reserverait de la memoire pour rien.
const BATCH_SIZE: usize = 32;

/// Score minimal du cross-encoder pour qu'un document soit juge pertinent.
///
/// # Pourquoi un seuil, et pourquoi celui-la
///
/// Il remplace le plancher de similarite cosinus comme mecanisme de rejet. Ce dernier etait
/// calibre sur un corpus anglophone, ou tout le hors-sujet etait interlingue : la fenetre entre
/// pertinent et hors-sujet ne faisait que trois centiemes. L'arrivee d'offres francaises l'a
/// refermee pour de bon — « plombier chauffagiste » remontait « Secretaire comptable ».
///
/// Le score du cross-encoder est un logit. Sur des documents courts (titre + entreprise + lieu),
/// sa frontiere naturelle est zero :
///
/// | document pour « plombier chauffagiste »          | score   |
/// |--------------------------------------------------|---------|
/// | Technicien de maintenance - reseaux de chauffage  | **+1.02** |
/// | Plombier chauffagiste - entretien de chaudieres   | **+0.76** |
/// | Secretaire comptable                              | −10.16  |
/// | Syndic de copropriete                             | −10.18  |
///
/// Mais le document reellement soumis porte aussi un extrait de description, qui dilue le signal
/// et decale toute la distribution vers le bas. Le seuil a donc ete etabli par bissection sur deux
/// cas opposes du corpus reel — « developpeur » (5 offres a trouver) et « plombier chauffagiste »
/// (aucune) :
///
/// | seuil | « developpeur » | « plombier chauffagiste » |
/// |-------|-----------------|---------------------------|
/// | −1    | 0 — faux negatifs | 0 |
/// | −3    | 5 | 0 |
/// | **−4** | **5** | **0** |
/// | −5    | 5 | 0 |
/// | −7    | 5 | 5 — faux positifs |
///
/// −4 est le milieu de la fenetre utilisable, donc la marge maximale des deux cotes. Quatre logits
/// de latitude, contre trois centiemes pour le plancher cosinus : c'est ce qui en fait un reglage
/// tenable plutot qu'un equilibre a recalibrer a chaque evolution du corpus.
///
/// Raccourcir le document soumis (titre seul) ramenerait la frontiere vers zero, au prix des
/// requetes portant sur une technologie mentionnee dans le corps de l'annonce.
const DEFAULT_RELEVANCE_FLOOR: f32 = -4.0;

/// Seuil effectif, surchargeable par `RERANKER_RELEVANCE_FLOOR`.
pub fn relevance_floor() -> f32 {
    std::env::var("RERANKER_RELEVANCE_FLOOR")
        .ok()
        .and_then(|value| value.trim().parse::<f32>().ok())
        .filter(|value| value.is_finite())
        .unwrap_or(DEFAULT_RELEVANCE_FLOOR)
}

/// Modele charge une seule fois pour la duree du processus.
///
/// Le fichier pese 279 Mo : le relire a chaque requete rendrait le reranking plus couteux que le
/// gain de pertinence qu'il apporte. `OnceLock` plutot que `lazy_static` : l'echec de chargement
/// doit etre memorise tel quel (`None`), pour ne pas retenter — et rejournaliser — a chaque appel.
static MODEL: OnceLock<Option<TextRerank>> = OnceLock::new();

pub struct RerankerService;

impl RerankerService {
    /// Racine des fichiers du modele. Surchargeable par `RERANKER_MODEL_DIR`.
    fn model_dir() -> PathBuf {
        std::env::var("RERANKER_MODEL_DIR")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(DEFAULT_MODEL_DIR))
    }

    /// Reclasse et renvoie les paires `(indice, score)`, du plus pertinent au moins pertinent.
    ///
    /// Le score du cross-encoder est un logit brut : negatif quand le document ne repond pas a la
    /// requete, positif quand il y repond. C'est un juge de pertinence bien plus fiable que la
    /// similarite cosinus, qui ne mesure qu'une ressemblance de surface.
    pub async fn rank_scored(query: &str, documents: Vec<String>) -> Option<Vec<(usize, f32)>> {
        let query = query.trim().to_string();
        if query.is_empty() || documents.is_empty() {
            return None;
        }

        tokio::task::spawn_blocking(move || {
            let reranker = Self::model()?;
            reranker
                .rerank(&query, documents.iter().collect(), false, Some(BATCH_SIZE))
                .ok()
                .map(|results| {
                    results
                        .into_iter()
                        .map(|r| (r.index, r.score))
                        .collect::<Vec<(usize, f32)>>()
                })
        })
        .await
        .ok()
        .flatten()
    }

    /// Charge le modele hors du chemin de la requete.
    ///
    /// Sans cela, la premiere recherche paie les 279 Mo de lecture ONNX, et elle les paie *sur le
    /// thread du runtime* : `is_available()` etant appele depuis une tache async, le chargement
    /// bloque aussi toutes les autres requetes servies par ce thread. Apres ce prechargement,
    /// `is_available()` et `model()` ne sont plus qu'une lecture de `OnceLock`.
    pub fn spawn_warmup() {
        tokio::task::spawn_blocking(|| {
            if Self::model().is_some() {
                tracing::info!("Cross-encoder preche, reclassement actif");
            }
        });
    }

    /// Vrai si le reranking est utilisable, sans le declencher.
    ///
    /// Utile pour ne pas payer le chargement du modele dans un chemin qui n'en a pas besoin.
    pub fn is_available() -> bool {
        Self::model().is_some()
    }

    /// Modele charge, ou `None` si les fichiers sont absents ou illisibles.
    fn model() -> Option<&'static TextRerank> {
        MODEL.get_or_init(Self::load).as_ref()
    }

    fn load() -> Option<TextRerank> {
        let dir = Self::model_dir();
        let onnx = dir.join("model.onnx");

        if !onnx.is_file() {
            tracing::info!(
                path = %onnx.display(),
                "Reranking desactive : modele absent. Lancer scripts/download-rerankers.sh pour l'activer."
            );
            return None;
        }

        // Les quatre fichiers sont exiges par `load_tokenizer`, qui panique si
        // `tokenizer_config.json` ne porte pas `model_max_length` et `pad_token`. On les lit donc
        // ici, ou une absence se traduit par un simple `None`.
        let tokenizer_files = TokenizerFiles {
            tokenizer_file: Self::read(&dir, "tokenizer.json")?,
            config_file: Self::read(&dir, "config.json")?,
            special_tokens_map_file: Self::read(&dir, "special_tokens_map.json")?,
            tokenizer_config_file: Self::read(&dir, "tokenizer_config.json")?,
        };

        let model = UserDefinedRerankingModel::new(
            OnnxSource::File(onnx.clone()),
            tokenizer_files,
        );
        // `RerankInitOptionsUserDefined` est `#[non_exhaustive]` : on part du defaut et on ajuste
        // le champ, la construction litterale etant interdite hors du crate.
        let mut options = RerankInitOptionsUserDefined::default();
        options.max_length = MAX_LENGTH;

        match TextRerank::try_new_from_user_defined(model, options) {
            Ok(reranker) => {
                tracing::info!(path = %onnx.display(), "Reranker cross-encoder charge");
                Some(reranker)
            }
            Err(e) => {
                tracing::warn!(path = %onnx.display(), error = %e, "Chargement du reranker echoue");
                None
            }
        }
    }

    fn read(dir: &Path, name: &str) -> Option<Vec<u8>> {
        let path = dir.join(name);
        match std::fs::read(&path) {
            Ok(bytes) => Some(bytes),
            Err(e) => {
                tracing::warn!(
                    path = %path.display(),
                    error = %e,
                    "Fichier de tokenizer manquant : reranking desactive"
                );
                None
            }
        }
    }

    /// Reclasse `documents` par pertinence pour `query` et renvoie les indices, du plus pertinent
    /// au moins pertinent.
    ///
    /// Renvoie l'ordre d'entree (`0..n`) quand le reranking est indisponible : l'appelant applique
    /// la permutation sans avoir a distinguer les deux cas.
    pub async fn rank_indices(query: &str, documents: Vec<String>) -> Vec<usize> {
        let identity = || (0..documents.len()).collect::<Vec<usize>>();

        let query = query.trim().to_string();
        // Rien a reclasser en dessous de deux documents, et une requete vide ne porte aucun signal.
        if query.is_empty() || documents.len() < 2 {
            return identity();
        }

        let expected = documents.len();
        // `rerank` est synchrone et parallelisee par rayon : l'appeler directement bloquerait un
        // worker Tokio pendant toute l'inference.
        let ranked = tokio::task::spawn_blocking(move || {
            let reranker = Self::model()?;
            match reranker.rerank(&query, documents.iter().collect(), false, Some(BATCH_SIZE)) {
                Ok(results) => Some(results.into_iter().map(|r| r.index).collect::<Vec<usize>>()),
                Err(e) => {
                    tracing::warn!(error = %e, "Inference de reranking echouee, ordre initial conserve");
                    None
                }
            }
        })
        .await;

        match ranked {
            Ok(Some(indices)) if Self::is_valid_permutation(&indices, expected) => indices,
            Ok(Some(indices)) => {
                // Un classement partiel reordonnerait en perdant des offres : mieux vaut l'ordre
                // d'entree, complet, qu'un sous-ensemble presente comme un classement.
                tracing::warn!(
                    got = indices.len(),
                    expected,
                    "Reranking incoherent, ordre initial conserve"
                );
                (0..expected).collect()
            }
            Ok(None) => (0..expected).collect(),
            Err(e) => {
                tracing::warn!(error = %e, "Tache de reranking interrompue");
                (0..expected).collect()
            }
        }
    }

    /// Vrai si `indices` est bien une permutation de `0..len` : chaque position une fois et une
    /// seule. Sans cette verification, un index hors bornes ferait paniquer l'appelant.
    fn is_valid_permutation(indices: &[usize], len: usize) -> bool {
        if indices.len() != len {
            return false;
        }
        let mut seen = vec![false; len];
        for index in indices {
            match seen.get_mut(*index) {
                Some(slot) if !*slot => *slot = true,
                _ => return false,
            }
        }
        true
    }

    /// Applique la permutation a `items`, en gardant au plus `top_k` elements.
    ///
    /// Generique sur l'element : le service ne connait ni les offres ni les documents RAG, il ne
    /// manipule que du texte et des indices.
    pub fn apply<T>(items: Vec<T>, indices: &[usize], top_k: usize) -> Vec<T> {
        // Aucun indice retenu = rien de pertinent : on rend une liste vide. `top_k.max(1)`
        // ci-dessous ne compense qu'un `top_k` a zero venant d'un appelant qui n'a pas borne son
        // parametre ; il ne doit pas fabriquer un resultat quand le classement n'en a retenu aucun.
        if indices.is_empty() {
            return Vec::new();
        }

        let mut slots: Vec<Option<T>> = items.into_iter().map(Some).collect();
        indices
            .iter()
            .filter_map(|index| slots.get_mut(*index).and_then(Option::take))
            .take(top_k.max(1))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_valid_permutation_accepts_a_complete_reordering() {
        assert!(RerankerService::is_valid_permutation(&[2, 0, 1], 3));
        assert!(RerankerService::is_valid_permutation(&[0], 1));
        assert!(RerankerService::is_valid_permutation(&[], 0));
    }

    #[test]
    fn is_valid_permutation_rejects_duplicates_gaps_and_out_of_range() {
        // Un doublon ferait apparaitre deux fois la meme offre.
        assert!(!RerankerService::is_valid_permutation(&[0, 0, 1], 3));
        // Un classement partiel perdrait des offres.
        assert!(!RerankerService::is_valid_permutation(&[0, 1], 3));
        // Un index hors bornes ferait paniquer l'appelant.
        assert!(!RerankerService::is_valid_permutation(&[0, 1, 5], 3));
    }

    #[test]
    fn apply_reorders_and_truncates() {
        let items = vec!["a", "b", "c", "d"];
        assert_eq!(
            RerankerService::apply(items.clone(), &[2, 0, 3, 1], 4),
            vec!["c", "a", "d", "b"]
        );
        assert_eq!(RerankerService::apply(items.clone(), &[2, 0, 3, 1], 2), vec!["c", "a"]);
    }

    #[test]
    fn apply_never_yields_the_same_item_twice() {
        // Garde-fou : meme si un index revenait deux fois, l'element n'est pris qu'une fois.
        let items = vec!["a", "b"];
        assert_eq!(RerankerService::apply(items, &[0, 0, 1], 3), vec!["a", "b"]);
    }

    #[test]
    fn apply_returns_nothing_when_no_index_was_retained() {
        // Cas central du seuil de pertinence : une requete sans aucune offre pertinente doit
        // rendre une liste vide, pas « les moins mauvaises ».
        assert!(RerankerService::apply(vec!["a", "b"], &[], 5).is_empty());
    }

    #[test]
    fn relevance_floor_stays_inside_the_measured_window() {
        // Zero est la frontiere naturelle d'un logit : negatif = ne repond pas a la requete.
        if std::env::var("RERANKER_RELEVANCE_FLOOR").is_err() {
            assert_eq!(relevance_floor(), DEFAULT_RELEVANCE_FLOOR);
        }
        // La fenetre mesuree est [-3, -5] : un defaut hors de cet intervalle reintroduirait des
        // faux negatifs (au-dessus) ou des faux positifs (en dessous).
        assert!((-5.0..=-3.0).contains(&DEFAULT_RELEVANCE_FLOOR));
        assert!(relevance_floor().is_finite());
    }

    #[test]
    fn apply_treats_a_zero_top_k_as_one() {
        // `top_k = 0` viendrait d'un appelant qui n'a pas borne son parametre : rendre une liste
        // vide masquerait le bug, rendre un element le laisse visible sans casser l'affichage.
        assert_eq!(RerankerService::apply(vec!["a", "b"], &[1, 0], 0), vec!["b"]);
    }

    #[tokio::test]
    async fn rank_indices_short_circuits_when_there_is_nothing_to_rank() {
        // Aucun de ces cas ne doit charger le modele : ce sont les chemins chauds de la recherche.
        assert_eq!(RerankerService::rank_indices("", vec!["a".into()]).await, vec![0]);
        assert_eq!(RerankerService::rank_indices("  ", vec![]).await, Vec::<usize>::new());
        assert_eq!(
            RerankerService::rank_indices("requete", vec!["seul".into()]).await,
            vec![0]
        );
    }

    #[test]
    fn model_dir_defaults_to_the_java_layout() {
        // Meme chemin que `reranker.model-path` cote Spring, pour qu'un seul script de
        // telechargement serve les deux backends.
        if std::env::var("RERANKER_MODEL_DIR").is_err() {
            assert_eq!(RerankerService::model_dir(), PathBuf::from(DEFAULT_MODEL_DIR));
        }
    }
}
