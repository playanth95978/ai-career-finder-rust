-- Create radar_hit table
CREATE TABLE radar_hit (
    id SERIAL PRIMARY KEY,
    user_id VARCHAR(255) NOT NULL,
    score DOUBLE PRECISION,
    why_you VARCHAR(255),
    seen BOOLEAN,
    dismissed BOOLEAN,
    created_at TIMESTAMP,
    jobOffer_id INTEGER REFERENCES job_offer(id),
    created_by VARCHAR(50),
    created_date TIMESTAMP,
    last_modified_by VARCHAR(50),
    last_modified_date TIMESTAMP);

-- Create indexes
CREATE INDEX idx_radar_hit_id ON radar_hit(id);
CREATE INDEX idx_radar_hit_jobOffer_id ON radar_hit(jobOffer_id);
