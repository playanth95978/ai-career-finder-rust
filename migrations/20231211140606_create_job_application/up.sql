-- Create job_application table
CREATE TABLE job_application (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id VARCHAR(255) NOT NULL,
    status VARCHAR(255),
    cover_letter VARCHAR(255),
    notes VARCHAR(255),
    match_score DOUBLE PRECISION,
    created_at TIMESTAMP,
    updated_at TIMESTAMP,
    applied_at TIMESTAMP,
    jobOffer_id UUID REFERENCES job_offer(id),
    candidateProfile_id UUID REFERENCES candidate_profile(id),
    created_by VARCHAR(50),
    created_date TIMESTAMP,
    last_modified_by VARCHAR(50),
    last_modified_date TIMESTAMP);

-- Create indexes
CREATE INDEX idx_job_application_id ON job_application(id);
CREATE INDEX idx_job_application_jobOffer_id ON job_application(jobOffer_id);
CREATE INDEX idx_job_application_candidateProfile_id ON job_application(candidateProfile_id);
