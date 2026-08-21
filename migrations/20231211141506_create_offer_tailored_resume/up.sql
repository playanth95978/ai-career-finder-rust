-- Create offer_tailored_resume table
CREATE TABLE offer_tailored_resume (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id VARCHAR(255) NOT NULL,
    data VARCHAR(255) NOT NULL,
    title VARCHAR(255),
    created_at TIMESTAMP,
    jobOffer_id UUID REFERENCES job_offer(id),
    created_by VARCHAR(50),
    created_date TIMESTAMP,
    last_modified_by VARCHAR(50),
    last_modified_date TIMESTAMP);

-- Create indexes
CREATE INDEX idx_offer_tailored_resume_id ON offer_tailored_resume(id);
CREATE INDEX idx_offer_tailored_resume_jobOffer_id ON offer_tailored_resume(jobOffer_id);
