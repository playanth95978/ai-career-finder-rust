-- Embedding du profil candidat (pgvector). 768 dimensions = nomic-embed-text, le meme modele
-- que l'application Spring, afin que les vecteurs restent comparables entre les deux backends.
CREATE EXTENSION IF NOT EXISTS vector;

ALTER TABLE candidate_profile ADD COLUMN embedding vector(768);

-- Index HNSW sur la distance cosinus, comme cote Java : recherche de similarite approximative
-- efficace des que le corpus grossit.
CREATE INDEX IF NOT EXISTS candidate_profile_embedding_idx
    ON candidate_profile USING HNSW (embedding vector_cosine_ops);
