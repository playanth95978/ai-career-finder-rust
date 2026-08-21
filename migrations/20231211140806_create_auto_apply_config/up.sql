-- Create auto_apply_config table
CREATE TABLE auto_apply_config (
    id SERIAL PRIMARY KEY,
    user_id VARCHAR(255) NOT NULL UNIQUE,
    mode VARCHAR(255),
    min_score DOUBLE PRECISION,
    max_per_day INTEGER,
    sources VARCHAR(255),
    created_by VARCHAR(50),
    created_date TIMESTAMP,
    last_modified_by VARCHAR(50),
    last_modified_date TIMESTAMP);

-- Create indexes
CREATE INDEX idx_auto_apply_config_id ON auto_apply_config(id);
CREATE INDEX idx_auto_apply_config_user_id ON auto_apply_config(user_id);
