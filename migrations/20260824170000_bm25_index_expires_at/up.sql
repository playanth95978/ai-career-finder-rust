-- Ajoute `expires_at` a l'index BM25 pour que le filtre d'expiration soit evalue dans l'index.
--
-- Mesure sur un corpus de 500 000 offres (jeu de donnees de charge, voir
-- `scripts/seed_bench_dataset.py`) :
--
--   SELECT id FROM job_offer WHERE search_text @@@ '...' ORDER BY paradedb.score(id) DESC LIMIT 60
--     sans le filtre d'expiration ................    8,5 ms    (400 blocs lus)
--     avec le filtre d'expiration ...............  245,0 ms  (100 804 blocs lus)
--
-- Soit un facteur 29, et environ 800 Mo relus a chaque recherche. La cause est visible dans le
-- plan : `expires_at` n'etant pas dans l'index, ParadeDB produit un `heap_filter` et va relire le
-- heap pour chaque candidat du haut de classement au lieu d'ecarter les offres expirees dans
-- Tantivy.
--
--   Tantivy Query: {"boolean":{"must":[ ... {"heap_filter":{"indexed_query":"all",
--                   "field_filters":[{"heap_filter":"(expires_at IS NULL)"}]}} ... ]}}
--
-- Le filtre lui-meme n'est pas negociable : il est ce qui empeche une annonce morte de remonter
-- dans les resultats, et les deux volets de la recherche hybride doivent partager le meme perimetre
-- pour que la fusion RRF compare des listes comparables. C'est donc l'index qu'il faut aligner sur
-- la requete.
--
-- Le volet vectoriel n'a pas ce probleme : HNSW repond en 1,9 ms sur le meme corpus, filtre inclus.
-- BM25 etait devenu a lui seul la latence de la recherche hybride.
--
-- ATTENTION : ParadeDB n'autorise qu'un seul index `USING bm25` par table, donc la mise a jour
-- passe obligatoirement par une suppression puis une reconstruction. Pendant la reconstruction, le
-- volet BM25 est indisponible et `JobSearchService::search_semantic` se replie sur le classement
-- lexical maison — ce qui est prevu, mais coute environ une seconde par requete sur ce volume.
-- Prevoir une fenetre calme sur les gros corpus.

DROP INDEX IF EXISTS job_offer_bm25_idx;

CREATE INDEX job_offer_bm25_idx
    ON job_offer
    USING bm25 (id, title, company, search_text, expires_at)
    WITH (
        key_field = id,
        text_fields = '{
        "title":       {"tokenizer": {"type": "default", "ascii_folding": true}, "normalizer": "lowercase"},
        "company":     {"tokenizer": {"type": "default", "ascii_folding": true}, "normalizer": "lowercase"},
        "search_text": {"tokenizer": {"type": "default", "ascii_folding": true}, "normalizer": "lowercase"}
    }'
    );
