//! Decoupage de l'ingestion en partitions independantes.
//!
//! Portage de `GrandDomainePartitioner` et `RomeGrandDomaine` du batch Java, sans le moteur qui les
//! portait : ce qui compte fonctionnellement est le decoupage lui-meme, pas l'abstraction
//! `Partitioner`.
//!
//! Le probleme resolu est une limite dure de l'API France Travail : une recherche ne peut jamais
//! renvoyer plus de 3150 resultats (plage `0-3149`). Une requete nationale non filtree ne verra
//! donc jamais le corpus entier, quel que soit le nombre de pages demandees. Filtrer par grand
//! domaine ROME donne quatorze requetes bornees separement, soit un plafond effectif de 44 100
//! offres — et le croisement par departement le multiplie encore quand un domaine national sature
//! a lui seul.

/// Un grand domaine ROME : le code d'une lettre passe en parametre `grandDomaine` a l'API, et son
/// libelle, qui ne sert qu'aux journaux.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RomeGrandDomaine {
    pub code: &'static str,
    pub label: &'static str,
}

/// Les quatorze grands domaines ROME.
///
/// La liste est figee dans le code plutot que lue en base : c'est une nomenclature publique et
/// stable, et une table de reference vide au demarrage rendrait l'ingestion silencieusement
/// inoperante. Meme contenu que `RomeGrandDomaine.ALL` cote Java, pour que les deux backends
/// couvrent exactement les memes categories.
pub const ROME_GRANDS_DOMAINES: [RomeGrandDomaine; 14] = [
    RomeGrandDomaine { code: "A", label: "Agriculture et Peche, Espaces naturels et verts, Soins aux animaux" },
    RomeGrandDomaine { code: "B", label: "Arts et Faconnage d'ouvrages d'art" },
    RomeGrandDomaine { code: "C", label: "Banque, Assurance, Immobilier" },
    RomeGrandDomaine { code: "D", label: "Commerce, Vente et Grande distribution" },
    RomeGrandDomaine { code: "E", label: "Communication, Media et Multimedia" },
    RomeGrandDomaine { code: "F", label: "Construction, Batiment et Travaux publics" },
    RomeGrandDomaine { code: "G", label: "Hotellerie-Restauration, Tourisme, Loisirs et Animation" },
    RomeGrandDomaine { code: "H", label: "Industrie" },
    RomeGrandDomaine { code: "I", label: "Installation et Maintenance" },
    RomeGrandDomaine { code: "J", label: "Sante" },
    RomeGrandDomaine { code: "K", label: "Services a la personne et a la collectivite" },
    RomeGrandDomaine { code: "L", label: "Spectacle" },
    RomeGrandDomaine { code: "M", label: "Support a l'entreprise" },
    RomeGrandDomaine { code: "N", label: "Transport et Logistique" },
];

/// Une unite de travail de l'ingestion : ce qu'une tache interroge et journalise seule.
///
/// `key` est la cle stockee dans `ingestion_run.partition_key`. Elle est **stable d'un passage a
/// l'autre** — c'est ce qui permet a la reprise de reconnaitre une partition deja aboutie. La
/// construire par formatage plutot que de la laisser choisir a l'appelant evite qu'un renommage
/// cosmetique fasse rejouer les quatorze partitions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Partition {
    /// Source interrogee, dans le vocabulaire de `job_offer.source`.
    pub source: String,
    /// Cle lisible et stable, unique au sein de la source.
    pub key: String,
    /// Grand domaine ROME, pour les partitions France Travail uniquement.
    pub grand_domaine: Option<&'static str>,
    /// Departement INSEE croise avec le domaine, quand la configuration en fournit.
    pub departement: Option<String>,
}

impl Partition {
    /// Partition unique d'une source qui ne se decoupe pas.
    ///
    /// La cle `all` n'est pas une valeur par defaut faute de mieux : elle dit que la source a ete
    /// traitee en entier, ce qui se lit dans le journal aussi bien qu'un `domaine:M`.
    pub fn whole(source: &str) -> Self {
        Self {
            source: source.to_string(),
            key: "all".to_string(),
            grand_domaine: None,
            departement: None,
        }
    }
}

/// Partitions France Travail : un grand domaine, croise avec les departements si la configuration
/// en donne.
///
/// Sans departement, les partitions sont nationales — quatorze requetes. En fournir multiplie le
/// nombre de partitions ; c'est le levier a actionner quand un domaine national depasse a lui seul
/// le plafond de 3150, et non un reglage a activer par defaut : chaque partition est un appel
/// reseau supplementaire sur une API a quota.
pub fn france_travail_partitions(source: &str, departements: &[String]) -> Vec<Partition> {
    let retained: Vec<&String> = departements
        .iter()
        .filter(|d| !d.trim().is_empty())
        .collect();

    let mut partitions = Vec::with_capacity(ROME_GRANDS_DOMAINES.len() * retained.len().max(1));
    for domaine in ROME_GRANDS_DOMAINES {
        if retained.is_empty() {
            partitions.push(Partition {
                source: source.to_string(),
                key: format!("domaine:{}", domaine.code),
                grand_domaine: Some(domaine.code),
                departement: None,
            });
            continue;
        }
        for departement in &retained {
            let departement = departement.trim();
            partitions.push(Partition {
                source: source.to_string(),
                key: format!("domaine:{}:dep:{departement}", domaine.code),
                grand_domaine: Some(domaine.code),
                departement: Some(departement.to_string()),
            });
        }
    }
    partitions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn les_quatorze_domaines_rome_ont_des_codes_distincts() {
        let mut codes: Vec<&str> = ROME_GRANDS_DOMAINES.iter().map(|d| d.code).collect();
        codes.sort_unstable();
        codes.dedup();
        // Un doublon ferait interroger deux fois le meme domaine tout en en laissant un autre
        // jamais ingere — un trou de couverture qu'aucune erreur ne signalerait.
        assert_eq!(codes.len(), ROME_GRANDS_DOMAINES.len());
        assert_eq!(codes, vec!["A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N"]);
    }

    #[test]
    fn sans_departement_les_partitions_sont_nationales() {
        let partitions = france_travail_partitions("FRANCE_TRAVAIL", &[]);

        assert_eq!(partitions.len(), 14);
        assert_eq!(partitions[0].key, "domaine:A");
        assert_eq!(partitions[0].grand_domaine, Some("A"));
        assert!(partitions.iter().all(|p| p.departement.is_none()));
    }

    #[test]
    fn les_departements_multiplient_les_partitions() {
        let departements = vec!["75".to_string(), "69".to_string()];
        let partitions = france_travail_partitions("FRANCE_TRAVAIL", &departements);

        assert_eq!(partitions.len(), 28);
        assert_eq!(partitions[0].key, "domaine:A:dep:75");
        assert_eq!(partitions[1].key, "domaine:A:dep:69");
        assert_eq!(partitions[1].departement.as_deref(), Some("69"));
    }

    #[test]
    fn les_departements_vides_sont_ignores_et_ne_creent_pas_de_partition_fantome() {
        // Un `FRANCE_TRAVAIL_DEPARTEMENTS=" , "` mal renseigne ne doit pas produire quatorze
        // partitions dont la cle se termine par `dep:` et dont la requete n'aurait aucun filtre.
        let partitions = france_travail_partitions("FRANCE_TRAVAIL", &["".to_string(), "  ".to_string()]);

        assert_eq!(partitions.len(), 14);
        assert_eq!(partitions[0].key, "domaine:A");
    }

    #[test]
    fn les_cles_de_partition_sont_uniques() {
        let departements = vec!["75".to_string(), "69".to_string()];
        let partitions = france_travail_partitions("FRANCE_TRAVAIL", &departements);

        let mut keys: Vec<&str> = partitions.iter().map(|p| p.key.as_str()).collect();
        keys.sort_unstable();
        keys.dedup();
        // Deux partitions de meme cle se recouvriraient dans `ingestion_run` : la reprise en
        // considererait une comme aboutie alors que l'autre a echoue.
        assert_eq!(keys.len(), partitions.len());
    }

    #[test]
    fn une_source_non_partitionnee_a_une_seule_unite_de_travail() {
        let partition = Partition::whole("EMPLOI_NC");

        assert_eq!(partition.key, "all");
        assert_eq!(partition.source, "EMPLOI_NC");
        assert!(partition.grand_domaine.is_none());
    }
}
