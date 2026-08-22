DROP INDEX IF EXISTS job_offer_embedding_status_idx;
DROP INDEX IF EXISTS job_offer_embedding_idx;
ALTER TABLE job_offer DROP COLUMN IF EXISTS embedding;
