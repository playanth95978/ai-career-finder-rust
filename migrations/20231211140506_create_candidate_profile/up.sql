-- Create candidate_profile table
CREATE TABLE candidate_profile (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id VARCHAR(255) NOT NULL,
    full_name VARCHAR(255),
    email VARCHAR(255),
    location VARCHAR(255),
    years_of_experience INTEGER,
    skills VARCHAR(255),
    experiences VARCHAR(255),
    preferred_roles VARCHAR(255),
    languages VARCHAR(255),
    education VARCHAR(255),
    certifications VARCHAR(255),
    raw_markdown VARCHAR(255),
    cv_filename VARCHAR(255),
    embedding_model VARCHAR(255),
    embedded_at TIMESTAMP,
    created_at TIMESTAMP,
    updated_at TIMESTAMP,
    created_by VARCHAR(50),
    created_date TIMESTAMP,
    last_modified_by VARCHAR(50),
    last_modified_date TIMESTAMP);

-- Create indexes
CREATE INDEX idx_candidate_profile_id ON candidate_profile(id);
