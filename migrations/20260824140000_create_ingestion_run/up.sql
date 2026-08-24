-- Journal des partitions d'ingestion, equivalent minimal du `JobRepository` de Spring Batch.
--
-- Le batch Java tire trois choses de son metadata repository : savoir ce qui a deja tourne
-- (reprise), savoir ce qui a echoue (diagnostic), et compter ce qui a ete lu/ecrit
-- (observabilite). Cette table porte les trois, a la granularite qui compte ici : la **partition**,
-- c'est-a-dire un couple (source, cle de partition) traite independamment des autres.
--
-- Pas de table `job_execution` separee : il n'y a qu'un seul job. `run_id` suffit a regrouper les
-- partitions d'un meme passage, et se lit comme un identifiant d'execution.
CREATE TABLE IF NOT EXISTS ingestion_run (
    id UUID PRIMARY KEY,

    -- Regroupe les partitions d'un meme declenchement. Genere par l'ordonnanceur, pas par la base :
    -- il doit etre connu avant la premiere insertion pour que les partitions le partagent.
    run_id UUID NOT NULL,

    -- Source interrogee (FRANCE_TRAVAIL, EMPLOI_NC, ...). Meme vocabulaire que `job_offer.source`.
    source VARCHAR(64) NOT NULL,

    -- Cle de partition, lisible : « domaine:M », « domaine:M:dep:75 », ou « all » pour les sources
    -- non partitionnees. Stockee en texte et non eclatee en colonnes : chaque source partitionne
    -- selon ses propres criteres, et une colonne par critere serait vide pour toutes les autres.
    partition_key VARCHAR(128) NOT NULL,

    -- PENDING -> RUNNING -> COMPLETED | FAILED. Meme convention que `job_offer.embedding_status`,
    -- pour qu'un seul vocabulaire d'etat serve dans tout le projet.
    status VARCHAR(16) NOT NULL,

    -- Offres rapportees par le connecteur, puis reellement inserees. Les deux comptes sont gardes
    -- separement : leur ecart est le taux de doublons, la seule mesure qui dise si une partition
    -- apporte encore quelque chose ou si elle ne fait que relire ce qu'on a deja.
    read_count INTEGER NOT NULL DEFAULT 0,
    written_count INTEGER NOT NULL DEFAULT 0,

    -- Offres ecartees par une erreur non fatale (mapping, ligne invalide) : l'equivalent du
    -- `skipLimit` de Spring Batch, mais compte plutot que plafonne — une partition qui echoue est
    -- reprise au passage suivant, donc rien ne justifie d'interrompre les treize autres.
    skipped_count INTEGER NOT NULL DEFAULT 0,

    -- Message d'erreur de la partition, quand `status = 'FAILED'`.
    error VARCHAR(1024),

    started_at TIMESTAMP NOT NULL,
    finished_at TIMESTAMP
);

-- Reprise : « quelles partitions de cette source n'ont pas abouti depuis telle date ». C'est la
-- seule requete du chemin chaud, elle dicte donc la forme de l'index.
CREATE INDEX IF NOT EXISTS ingestion_run_source_status_started_idx
    ON ingestion_run (source, status, started_at DESC);

-- Lecture d'un passage complet, pour le diagnostic.
CREATE INDEX IF NOT EXISTS ingestion_run_run_id_idx
    ON ingestion_run (run_id);
