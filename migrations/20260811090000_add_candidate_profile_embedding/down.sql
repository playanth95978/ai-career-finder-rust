DROP INDEX IF EXISTS candidate_profile_embedding_idx;
ALTER TABLE candidate_profile DROP COLUMN IF EXISTS embedding;
