-- Retour a la definition sans `expires_at`, telle que posee par
-- `20260823120000_add_job_offer_bm25_index`.
DROP INDEX IF EXISTS job_offer_bm25_idx;

CREATE INDEX job_offer_bm25_idx
    ON job_offer
    USING bm25 (id, title, company, search_text)
    WITH (
        key_field = id,
        text_fields = '{
        "title":       {"tokenizer": {"type": "default", "ascii_folding": true}, "normalizer": "lowercase"},
        "company":     {"tokenizer": {"type": "default", "ascii_folding": true}, "normalizer": "lowercase"},
        "search_text": {"tokenizer": {"type": "default", "ascii_folding": true}, "normalizer": "lowercase"}
    }'
    );
