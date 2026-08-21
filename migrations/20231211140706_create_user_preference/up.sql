-- Create user_preference table
CREATE TABLE user_preference (
    id SERIAL PRIMARY KEY,
    user_id VARCHAR(255) NOT NULL UNIQUE,
    remote_only BOOLEAN,
    contract_type VARCHAR(255),
    salary_min INTEGER,
    salary_max INTEGER,
    preferred_roles VARCHAR(255),
    excluded_technologies VARCHAR(255),
    preferred_locations VARCHAR(255),
    created_by VARCHAR(50),
    created_date TIMESTAMP,
    last_modified_by VARCHAR(50),
    last_modified_date TIMESTAMP);

-- Create indexes
CREATE INDEX idx_user_preference_id ON user_preference(id);
CREATE INDEX idx_user_preference_user_id ON user_preference(user_id);
