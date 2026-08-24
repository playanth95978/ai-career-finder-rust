//! Reranking cross-encoder ONNX, portage du `OnnxRerankerService` de l'application Spring.
//!
//! La recherche vectorielle repond « quels documents ressemblent a la requete » ; un cross-encoder
//! repond « lequel repond a la requete ». Il lit la paire (requete, document) ensemble au lieu de
//! comparer deux vecteurs calcules separement, ce qui corrige les inversions de classement que le
//! cosinus laisse passer — precisement le point faible mesure sur ce corpus, ou la fenetre entre
//! pertinent et hors-sujet ne fait que quelques centiemes.
//!
//! # Choix d'implementation : `ort` brut et non `fastembed`
//!
//! Ce module utilisait `fastembed::TextRerank`, qui empaquetait tokenizer, session ONNX et
//! decoupage en lots. Il a fallu en sortir pour une raison mesuree, pas esthetique : **fastembed ne
//! laisse pas regler son parallelisme**, et son reglage par defaut rend le service incapable de
//! servir deux utilisateurs.
//!
//! `fastembed-4.9.1/src/reranking/impl.rs` fixe `with_intra_threads(available_parallelism())` — 16
//! threads ONNX sur une machine a 16 coeurs — et decoupe en plus les lots avec `par_chunks`, donc
//! rayon par-dessus. Aucun des deux n'est expose : `RerankInitOptionsUserDefined` ne porte que
//! `execution_providers` et `max_length`. Mesure de bout en bout sur 500 000 offres :
//!
//! | utilisateurs | p50    | debit      | CPU (16 coeurs) |
//! |--------------|--------|------------|-----------------|
//! | 1            | 804 ms | 1,2 req/s  | 78 %            |
//! | 2            | 1533 ms| 1,3 req/s  | 93 %            |
//! | 8            | 4647 ms| 1,6 req/s  | 89 %            |
//!
//! Une seule requete saturait la machine, donc le debit restait plat quelle que soit la
//! concurrence : la latence ne faisait que refleter une file d'attente. Le meme test sans
//! reranking montait a 16 req/s.
//!
//! Le tokenizer et la session sont donc pilotes directement, ce qui donne les deux leviers
//! manquants : le nombre de threads d'inference, et le choix du fournisseur d'execution. La
//! semantique de tokenisation reproduit celle de `fastembed::load_tokenizer` (troncature a
//! `MAX_LENGTH`, remplissage au plus long du lot, `pad_token` et `pad_token_id` lus dans les
//! fichiers du modele), pour que les scores restent comparables a ceux d'avant.
//!
//! # Degradation
//!
//! Le reranking est un **raffinement**, pas une condition de la recherche : modele absent, fichier
//! corrompu ou inference en echec renvoient l'ordre d'entree inchange, apres journalisation. Une
//! recherche qui repond moins bien vaut mieux qu'une recherche qui ne repond plus.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use ndarray::{Array2, Axis};
use ort::session::{builder::GraphOptimizationLevel, Session};
use ort::value::Value;
use tokenizers::{PaddingParams, PaddingStrategy, Tokenizer, TruncationParams};

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
const MAX_LENGTH: usize = 128;

/// Taille de lot d'inference.
///
/// Un lot est materialise en tenseurs `MAX_LENGTH * taille_du_lot`, et on ne reclasse jamais plus
/// de quelques dizaines d'offres. Les lots sont traites **en serie** : c'est volontaire, le
/// parallelisme est confie a ONNX Runtime via `RERANKER_THREADS`, la ou il est borne. Empiler rayon
/// par-dessus les threads d'inference — ce que faisait fastembed — multipliait les deux.
const BATCH_SIZE: usize = 32;

/// Threads d'inference ONNX par appel.
///
/// **Le reglage decisif pour la capacite du service.** Il ne s'agit pas d'aller plus vite mais de
/// laisser de la place : une inference qui prend les seize coeurs rend le serveur mono-utilisateur,
/// puisque la deuxieme requete n'a plus rien a prendre.
///
/// Valeur choisie par balayage, sur une machine a 16 coeurs et un corpus de 500 000 offres :
///
/// | threads | 1 utilisateur | 4 utilisateurs | debit a 16 | CPU a 4 util. |
/// |---------|---------------|----------------|------------|---------------|
/// | 1       | —             | —              | 3,16 req/s | —             |
/// | 2       | 861 ms        | —              | 3,65 req/s | —             |
/// | **4**   | **471 ms**    | **1053 ms**    | **3,72**   | **44 %**      |
/// | 8       | 393 ms        | 1004 ms        | —          | 69 %          |
/// | 16      | 491 ms        | 1309 ms        | 2,94 req/s | 93 %          |
///
/// Deux enseignements. D'abord, seize threads sont **plus lents** que quatre, meme pour un seul
/// utilisateur : la contention entre threads sur un graphe de cette taille coute plus que le
/// parallelisme ne rapporte — c'etait le reglage code en dur par fastembed. Ensuite, huit threads
/// gagnent 78 ms sur une requete isolee mais consomment 69 % du CPU a quatre utilisateurs contre
/// 44 % : quatre laisse de la marge pour le reste du serveur.
///
/// Surchargeable par `RERANKER_THREADS` : une instance dediee a un seul utilisateur a interet a
/// monter a huit.
const DEFAULT_THREADS: usize = 4;

fn inference_threads() -> usize {
    std::env::var("RERANKER_THREADS")
        .ok()
        .and_then(|value| value.trim().parse::<usize>().ok())
        .filter(|threads| *threads > 0)
        .unwrap_or(DEFAULT_THREADS)
}

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

/// Modele charge : session ONNX, tokenizer configure, et presence de `token_type_ids` a l'entree.
///
/// Le nom des entrees varie d'un cross-encoder a l'autre : `bge-reranker-base` en attend trois,
/// d'autres deux. On l'interroge une fois au chargement plutot que de deviner, un nom d'entree
/// inattendu faisant echouer chaque inference.
struct Reranker {
    session: Session,
    tokenizer: Tokenizer,
    needs_token_type_ids: bool,
}

impl Reranker {
    /// Score de pertinence de chaque paire (requete, document), dans l'ordre d'entree.
    ///
    /// Le score est le premier logit de la sortie `logits`, comme le fait fastembed : le modele est
    /// entraine avec une seule tete de classement, et c'est ce logit brut — negatif hors-sujet,
    /// positif pertinent — que `relevance_floor` seuille.
    fn scores(&self, query: &str, documents: &[String]) -> Result<Vec<f32>, String> {
        let mut all = Vec::with_capacity(documents.len());

        for batch in documents.chunks(BATCH_SIZE) {
            let pairs: Vec<(&str, &str)> = batch.iter().map(|d| (query, d.as_str())).collect();
            let encodings = self
                .tokenizer
                .encode_batch(pairs, true)
                .map_err(|e| format!("tokenisation : {e}"))?;

            let Some(width) = encodings.first().map(|e| e.len()) else {
                continue;
            };
            let rows = encodings.len();

            let mut ids = Vec::with_capacity(rows * width);
            let mut mask = Vec::with_capacity(rows * width);
            let mut type_ids = Vec::with_capacity(rows * width);
            for encoding in &encodings {
                // Le remplissage est `BatchLongest`, donc toutes les lignes ont la meme largeur.
                // La verifier ici plutot que de laisser `from_shape_vec` echouer donne un message
                // exploitable au lieu d'une erreur de forme.
                if encoding.len() != width {
                    return Err(format!(
                        "lignes de largeurs differentes ({} vs {width})",
                        encoding.len()
                    ));
                }
                ids.extend(encoding.get_ids().iter().map(|v| *v as i64));
                mask.extend(encoding.get_attention_mask().iter().map(|v| *v as i64));
                type_ids.extend(encoding.get_type_ids().iter().map(|v| *v as i64));
            }

            let ids = Array2::from_shape_vec((rows, width), ids)
                .map_err(|e| format!("forme input_ids : {e}"))?;
            let mask = Array2::from_shape_vec((rows, width), mask)
                .map_err(|e| format!("forme attention_mask : {e}"))?;

            // Les tenseurs sont construits avant la macro : `ort::inputs!` applique son propre `?`
            // sur `ort::Error`, donc un `map_err` vers `String` a l'interieur ne compile pas.
            let ids = Value::from_array(ids).map_err(|e| e.to_string())?;
            let mask = Value::from_array(mask).map_err(|e| e.to_string())?;
            let mut inputs = ort::inputs![
                "input_ids" => ids,
                "attention_mask" => mask,
            ]
            .map_err(|e| e.to_string())?;

            if self.needs_token_type_ids {
                let type_ids = Array2::from_shape_vec((rows, width), type_ids)
                    .map_err(|e| format!("forme token_type_ids : {e}"))?;
                let type_ids = Value::from_array(type_ids).map_err(|e| e.to_string())?;
                inputs.push(("token_type_ids".into(), type_ids.into()));
            }

            let outputs = self.session.run(inputs).map_err(|e| e.to_string())?;
            let logits = outputs["logits"]
                .try_extract_tensor::<f32>()
                .map_err(|e| format!("extraction des logits : {e}"))?;

            // `index_axis(Axis(1), 0)` et non un `slice` : la sortie est `(lot, 1)` pour ce modele
            // mais `(lot, n)` pour un cross-encoder multi-tete, et seule la premiere colonne porte
            // le score de pertinence dans les deux cas.
            all.extend(logits.index_axis(Axis(1), 0).iter().copied());
        }

        if all.len() != documents.len() {
            return Err(format!("{} scores pour {} documents", all.len(), documents.len()));
        }
        Ok(all)
    }
}

/// Modele charge une seule fois pour la duree du processus.
///
/// Le fichier pese 279 Mo : le relire a chaque requete rendrait le reranking plus couteux que le
/// gain de pertinence qu'il apporte. `OnceLock` plutot que `lazy_static` : l'echec de chargement
/// doit etre memorise tel quel (`None`), pour ne pas retenter — et rejournaliser — a chaque appel.
static MODEL: OnceLock<Option<Reranker>> = OnceLock::new();

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

        // `spawn_blocking` : l'inference est synchrone et tient son thread pendant toute sa duree.
        // Sur le thread du runtime, elle bloquerait aussi toutes les autres requetes qu'il sert.
        tokio::task::spawn_blocking(move || {
            let reranker = Self::model()?;
            match reranker.scores(&query, &documents) {
                Ok(scores) => {
                    let mut scored: Vec<(usize, f32)> =
                        scores.into_iter().enumerate().collect();
                    // Tri decroissant. `total_cmp` et non `partial_cmp` : un NaN sorti du modele
                    // ferait paniquer un comparateur qui deballe une option.
                    scored.sort_by(|a, b| b.1.total_cmp(&a.1));
                    Some(scored)
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Inference de reranking echouee");
                    None
                }
            }
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
    fn model() -> Option<&'static Reranker> {
        MODEL.get_or_init(Self::load).as_ref()
    }

    fn load() -> Option<Reranker> {
        let dir = Self::model_dir();
        let onnx = dir.join("model.onnx");

        if !onnx.is_file() {
            tracing::info!(
                path = %onnx.display(),
                "Reranking desactive : modele absent. Lancer scripts/download-rerankers.sh pour l'activer."
            );
            return None;
        }

        let tokenizer = Self::load_tokenizer(&dir)?;
        let threads = inference_threads();

        // `with_intra_threads` est la raison d'etre de ce module : c'est le reglage que fastembed
        // codait en dur au nombre de coeurs.
        let session = Session::builder()
            .and_then(|builder| builder.with_optimization_level(GraphOptimizationLevel::Level3))
            .and_then(|builder| builder.with_intra_threads(threads))
            // Un seul thread inter-operateurs : le graphe d'un cross-encoder est essentiellement
            // sequentiel, il n'y a pas de branches a executer en parallele.
            .and_then(|builder| builder.with_inter_threads(1))
            .and_then(|builder| builder.commit_from_file(&onnx));

        let session = match session {
            Ok(session) => session,
            Err(e) => {
                tracing::warn!(path = %onnx.display(), error = %e, "Chargement du reranker echoue");
                return None;
            }
        };

        let needs_token_type_ids = session
            .inputs
            .iter()
            .any(|input| input.name == "token_type_ids");

        tracing::info!(
            path = %onnx.display(),
            threads,
            token_type_ids = needs_token_type_ids,
            "Reranker cross-encoder charge"
        );
        Some(Reranker {
            session,
            tokenizer,
            needs_token_type_ids,
        })
    }

    /// Tokenizer configure comme le faisait `fastembed::load_tokenizer`.
    ///
    /// Les valeurs de remplissage viennent des fichiers du modele et non de constantes : un
    /// `pad_id` errone decale tout le lot et fait juger au modele des sequences qu'il n'a jamais
    /// vues, sans qu'aucune erreur ne soit levee.
    fn load_tokenizer(dir: &Path) -> Option<Tokenizer> {
        let mut tokenizer = match Tokenizer::from_file(dir.join("tokenizer.json")) {
            Ok(tokenizer) => tokenizer,
            Err(e) => {
                tracing::warn!(error = %e, "tokenizer.json illisible : reranking desactive");
                return None;
            }
        };

        let config: serde_json::Value = Self::read_json(dir, "config.json")?;
        let tokenizer_config: serde_json::Value = Self::read_json(dir, "tokenizer_config.json")?;

        // `model_max_length` vaut 1e30 pour ce modele, donc il ne borne rien en pratique — mais le
        // lire evite de tronquer au-dela de ce que le modele accepte sur un autre cross-encoder.
        let model_max_length = tokenizer_config["model_max_length"]
            .as_f64()
            .unwrap_or(f64::MAX);
        let max_length = MAX_LENGTH.min(model_max_length as usize);

        let pad_id = config["pad_token_id"].as_u64().unwrap_or(0) as u32;
        let pad_token = tokenizer_config["pad_token"]
            .as_str()
            .unwrap_or("[PAD]")
            .to_string();

        tokenizer.with_padding(Some(PaddingParams {
            strategy: PaddingStrategy::BatchLongest,
            pad_token,
            pad_id,
            ..Default::default()
        }));
        if let Err(e) = tokenizer.with_truncation(Some(TruncationParams {
            max_length,
            ..Default::default()
        })) {
            tracing::warn!(error = %e, "Troncature du tokenizer refusee : reranking desactive");
            return None;
        }

        Some(tokenizer)
    }

    fn read_json(dir: &Path, name: &str) -> Option<serde_json::Value> {
        let path = dir.join(name);
        match std::fs::read(&path).map(|bytes| serde_json::from_slice(&bytes)) {
            Ok(Ok(value)) => Some(value),
            Ok(Err(e)) => {
                tracing::warn!(path = %path.display(), error = %e, "Fichier de tokenizer illisible");
                None
            }
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
        let expected = documents.len();
        // Rien a reclasser en dessous de deux documents, et une requete vide ne porte aucun signal.
        if query.trim().is_empty() || expected < 2 {
            return (0..expected).collect();
        }

        match Self::rank_scored(query, documents).await {
            Some(scored) => {
                let indices: Vec<usize> = scored.into_iter().map(|(index, _)| index).collect();
                if Self::is_valid_permutation(&indices, expected) {
                    indices
                } else {
                    // Un classement partiel reordonnerait en perdant des offres : mieux vaut
                    // l'ordre d'entree, complet, qu'un sous-ensemble presente comme un classement.
                    tracing::warn!(
                        got = indices.len(),
                        expected,
                        "Reranking incoherent, ordre initial conserve"
                    );
                    (0..expected).collect()
                }
            }
            None => (0..expected).collect(),
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

