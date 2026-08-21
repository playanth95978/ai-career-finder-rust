-- Retour a VARCHAR(255) : tronque les valeurs plus longues, d'ou le USING explicite.
ALTER TABLE candidate_profile ALTER COLUMN raw_markdown TYPE VARCHAR(255) USING left(raw_markdown, 255);
ALTER TABLE job_offer ALTER COLUMN description TYPE VARCHAR(255) USING left(description, 255),
                      ALTER COLUMN search_text TYPE VARCHAR(255) USING left(search_text, 255);
