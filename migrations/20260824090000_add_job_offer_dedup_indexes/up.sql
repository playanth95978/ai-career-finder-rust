-- Index sur les cles de deduplication de l'ingestion.
--
-- `JobSearchService::persist_all` fait une recherche par offre recuperee, d'abord sur
-- `(source, source_id)` puis en repli sur `apply_url`. Aucune des deux n'etait indexee : chaque
-- recherche live declenchait donc autant de balayages sequentiels que d'offres rapportees par les
-- connecteurs — une centaine par requete, sur une table qui grossit a chaque ingestion.
--
-- Constate en conditions reelles : une recherche live simultanee au poller d'embedding a fait
-- s'empiler les connexions du pool au point de depasser trois minutes. Le cout est invisible sur
-- mille lignes, prohibitif sur cinquante mille.
--
-- Pas d'unicite : deux sources peuvent legitimement publier la meme URL de candidature, et
-- `source_id` est absent chez les sources qui n'exposent pas d'identifiant.
CREATE INDEX IF NOT EXISTS job_offer_source_source_id_idx
    ON job_offer (source, source_id);

CREATE INDEX IF NOT EXISTS job_offer_apply_url_idx
    ON job_offer (apply_url);
