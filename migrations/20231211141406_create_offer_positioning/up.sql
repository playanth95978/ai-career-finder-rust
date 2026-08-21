-- Create offer_positioning table
CREATE TABLE offer_positioning (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id VARCHAR(255) NOT NULL,
    result VARCHAR(255) NOT NULL,
    created_at TIMESTAMP,
    jobOffer_id UUID REFERENCES job_offer(id),
    created_by VARCHAR(50),
    created_date TIMESTAMP,
    last_modified_by VARCHAR(50),
    last_modified_date TIMESTAMP);

-- Create indexes
CREATE INDEX idx_offer_positioning_id ON offer_positioning(id);
CREATE INDEX idx_offer_positioning_jobOffer_id ON offer_positioning(jobOffer_id);
