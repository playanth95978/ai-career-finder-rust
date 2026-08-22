-- Recherche vectorielle sur les offres. Jusqu'ici seul `candidate_profile` portait un vecteur :
-- `search-smart` et `jobs/match` devaient donc se rabattre sur un classement lexical, incapable
-- de rapprocher « devops » et « ingenieur infrastructure ».
--
-- 768 dimensions : identique a `candidate_profile.embedding`, pour que les deux cotes restent
-- comparables (meme modele nomic-embed-text que l'application Spring).
CREATE EXTENSION IF NOT EXISTS vector;

ALTER TABLE job_offer ADD COLUMN IF NOT EXISTS embedding vector(768);

-- HNSW avec la distance cosinus, comme l'index du profil candidat.
CREATE INDEX IF NOT EXISTS job_offer_embedding_idx
    ON job_offer USING HNSW (embedding vector_cosine_ops);

-- Le poller d'embedding balaye les offres en attente : sans cet index il scanne toute la table
-- a chaque reveil.
CREATE INDEX IF NOT EXISTS job_offer_embedding_status_idx
    ON job_offer (embedding_status);
