-- Le blueprint JDL a mappe tous les champs longs (TextBlob) en VARCHAR(255) : insuffisant pour
-- le markdown d'un CV, une description d'offre ou un payload JSON. On les passe en TEXT.
--
-- Pourquoi TEXT et non jsonb (comme cote Java) : en Diesel, `VarChar` est un alias de `Text`, donc
-- ce changement ne demande aucune modification de `schema.rs` ni des modeles (`Option<String>`).
-- Passer en jsonb imposerait `Option<serde_json::Value>` dans toute la chaine DTO/service ; a faire
-- le jour ou l'on aura besoin des operateurs JSON en SQL.
ALTER TABLE auto_apply_config     ALTER COLUMN sources              TYPE TEXT;
ALTER TABLE candidate_profile     ALTER COLUMN certifications       TYPE TEXT,
                                  ALTER COLUMN education            TYPE TEXT,
                                  ALTER COLUMN experiences           TYPE TEXT,
                                  ALTER COLUMN languages             TYPE TEXT,
                                  ALTER COLUMN preferred_roles       TYPE TEXT,
                                  ALTER COLUMN raw_markdown          TYPE TEXT,
                                  ALTER COLUMN skills                TYPE TEXT;
ALTER TABLE conversation          ALTER COLUMN metadata              TYPE TEXT,
                                  ALTER COLUMN summary               TYPE TEXT;
ALTER TABLE cv_resume             ALTER COLUMN data                  TYPE TEXT;
ALTER TABLE cv_resume_version     ALTER COLUMN data                  TYPE TEXT;
ALTER TABLE job_application       ALTER COLUMN cover_letter          TYPE TEXT,
                                  ALTER COLUMN notes                 TYPE TEXT;
ALTER TABLE job_offer             ALTER COLUMN description           TYPE TEXT,
                                  ALTER COLUMN indexing_error        TYPE TEXT,
                                  ALTER COLUMN metadata              TYPE TEXT,
                                  ALTER COLUMN raw_payload           TYPE TEXT,
                                  ALTER COLUMN search_text           TYPE TEXT,
                                  ALTER COLUMN skills                TYPE TEXT;
ALTER TABLE offer_positioning     ALTER COLUMN result                TYPE TEXT;
ALTER TABLE offer_tailored_resume ALTER COLUMN data                  TYPE TEXT;
ALTER TABLE radar_hit             ALTER COLUMN why_you               TYPE TEXT;
ALTER TABLE user_preference       ALTER COLUMN excluded_technologies TYPE TEXT,
                                  ALTER COLUMN preferred_locations   TYPE TEXT,
                                  ALTER COLUMN preferred_roles       TYPE TEXT;
