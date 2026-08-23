    //! Fusion de listes classees par Reciprocal Rank Fusion (RRF).
    //!
    //! Portage de l'etape de fusion du pipeline hybride Java. Le probleme qu'elle resout : la
    //! recherche vectorielle et la recherche lexicale produisent des scores **non comparables** — une
    //! similarite cosinus de 0,59 et un score de correspondance lexicale de 12 ne se moyennent pas.
    //! RRF ignore les scores et ne regarde que les **rangs**, ce qui rend les listes fusionnables sans
    //! normalisation ni calibrage par source.
    //!
    //! Pour chaque document, le score est la somme sur toutes les listes de `1 / (k + rang)`, le rang
    //! commencant a 1. Un document bien classe dans deux listes passe donc devant un document premier
    //! d'une seule — c'est exactement l'effet recherche : le vectoriel rattrape ce que le lexical
    //! manque faute de mot commun, le lexical rattrape ce que le vectoriel noie.

    use std::collections::HashMap;
    use std::hash::Hash;

    /// Constante d'amortissement de l'article d'origine (Cormack, Clarke & Buettcher, 2009).
    ///
    /// Elle borne l'avantage des toutes premieres places : sans elle, `1/rang` donnerait au premier
    /// resultat un poids ecrasant (1,0 contre 0,5 pour le second), et une seule liste dicterait la
    /// fusion. Avec k = 60, l'ecart entre le premier et le second tombe sous les 2 %, ce qui laisse
    /// l'accord entre listes decider.
    pub const RRF_K: f64 = 60.0;

    pub struct RrfFusionService;

    impl RrfFusionService {
        /// Fusionne des listes classees et renvoie les elements par score RRF decroissant.
        ///
        /// `identity` extrait la cle de deduplication : un meme document apparait dans plusieurs
        /// listes, et c'est cet accord qui doit le faire remonter. La valeur conservee est celle de la
        /// premiere occurrence — les listes portent le meme document, seul son rang differe.
        ///
        /// Une liste vide est ignoree sans cas particulier : une source indisponible ne doit pas
        /// modifier le classement des autres.
        pub fn fuse<T, K, F>(lists: Vec<Vec<T>>, identity: F) -> Vec<T>
        where
            K: Eq + Hash + Clone,
            F: Fn(&T) -> K,
        {
            Self::fuse_with_k(lists, identity, RRF_K)
        }

        /// Variante a constante explicite, pour les tests et l'experimentation.
        pub fn fuse_with_k<T, K, F>(lists: Vec<Vec<T>>, identity: F, k: f64) -> Vec<T>
        where
            K: Eq + Hash + Clone,
            F: Fn(&T) -> K,
        {
            // Un seul passage : chaque document rencontre est accumule dans `entries`, dont l'ordre
            // d'insertion sert de bris d'egalite stable. Sans cela, deux documents de meme score
            // sortiraient dans un ordre dependant du hachage.
            let mut entries: Vec<(T, f64)> = Vec::new();
            let mut slot_of: HashMap<K, usize> = HashMap::new();

            for list in lists {
                for (position, item) in list.into_iter().enumerate() {
                    // Rang a partir de 1 : a partir de 0, le premier resultat vaudrait `1/k`, comme
                    // s'il n'etait pas classe premier.
                    let contribution = 1.0 / (k + position as f64 + 1.0);
                    let key = identity(&item);

                    match slot_of.get(&key) {
                        Some(slot) => entries[*slot].1 += contribution,
                        None => {
                            slot_of.insert(key, entries.len());
                            entries.push((item, contribution));
                        }
                    }
                }
            }

            // `sort_by` est stable : a score egal, l'ordre de premiere apparition est preserve.
            entries.sort_by(|a, b| b.1.total_cmp(&a.1));
            entries.into_iter().map(|(item, _)| item).collect()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        /// Document de test : un identifiant suffit a exercer la fusion.
        fn ids(items: &[&str]) -> Vec<String> {
            items.iter().map(|s| s.to_string()).collect()
        }

        fn fuse(lists: Vec<Vec<String>>) -> Vec<String> {
            RrfFusionService::fuse(lists, |item: &String| item.clone())
        }

        #[test]
        fn agreement_between_lists_outranks_a_single_first_place() {
            // « b » est second partout, « a » premier d'une seule liste. C'est tout l'interet de RRF :
            // l'accord entre sources l'emporte sur une premiere place isolee.
            let fused = fuse(vec![ids(&["a", "b"]), ids(&["c", "b"])]);
            assert_eq!(fused.first().map(String::as_str), Some("b"));
        }

        #[test]
        fn a_document_found_by_one_source_only_is_still_kept() {
            // Le vectoriel doit pouvoir rattraper ce que le lexical manque : rien ne doit disparaitre.
            let fused = fuse(vec![ids(&["vecteur-seul"]), ids(&["lexical-seul"])]);
            assert_eq!(fused.len(), 2);
            assert!(fused.contains(&"vecteur-seul".to_string()));
            assert!(fused.contains(&"lexical-seul".to_string()));
        }

        #[test]
        fn duplicates_are_merged_not_repeated() {
            // Un doublon afficherait deux fois la meme offre au candidat.
            let fused = fuse(vec![ids(&["a", "b"]), ids(&["a", "b"]), ids(&["a"])]);
            assert_eq!(fused, ids(&["a", "b"]));
        }

        #[test]
        fn empty_lists_do_not_disturb_the_ranking() {
            // Une source indisponible rend une liste vide : le classement des autres est inchange.
            let with_empty = fuse(vec![vec![], ids(&["a", "b", "c"]), vec![]]);
            assert_eq!(with_empty, ids(&["a", "b", "c"]));
            assert!(fuse(vec![]).is_empty());
            assert!(fuse(vec![vec![], vec![]]).is_empty());
        }

        #[test]
        fn a_single_list_is_returned_in_its_original_order() {
            // Avec une seule source, RRF est l'identite : les rangs sont deja strictement decroissants.
            assert_eq!(fuse(vec![ids(&["a", "b", "c", "d"])]), ids(&["a", "b", "c", "d"]));
        }

        #[test]
        fn ties_keep_the_first_seen_order() {
            // Deux documents premiers de listes distinctes ont le meme score : l'ordre doit etre
            // deterministe, pas dependant du hachage.
            let first = fuse(vec![ids(&["a"]), ids(&["b"])]);
            for _ in 0..20 {
                assert_eq!(fuse(vec![ids(&["a"]), ids(&["b"])]), first);
            }
            assert_eq!(first, ids(&["a", "b"]));
        }

        #[test]
        fn k_damps_the_advantage_of_the_top_rank() {
            // Justification de RRF_K : avec k = 0, la premiere place ecrase tout et une seule liste
            // dicte la fusion ; avec k = 60, l'accord entre listes reprend la main.
            let lists = || vec![ids(&["a", "b"]), ids(&["c", "b"])];

            let damped = RrfFusionService::fuse_with_k(lists(), |i: &String| i.clone(), RRF_K);
            assert_eq!(damped.first().map(String::as_str), Some("b"));

            let undamped = RrfFusionService::fuse_with_k(lists(), |i: &String| i.clone(), 0.0);
            assert_eq!(undamped.first().map(String::as_str), Some("a"));
        }

        #[test]
        fn scores_follow_the_reciprocal_rank_formula() {
            // Verification arithmetique directe : « b » cumule deux secondes places
            // (2 / (60 + 2) = 0.03226), « a » une seule premiere (1 / (60 + 1) = 0.01639).
            let expected_b = 2.0 / (RRF_K + 2.0);
            let expected_a = 1.0 / (RRF_K + 1.0);
            assert!(expected_b > expected_a, "l'accord doit primer : {expected_b} > {expected_a}");
        }
    }
