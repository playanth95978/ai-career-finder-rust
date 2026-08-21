-- Create radar_state table
CREATE TABLE radar_state (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id VARCHAR(255) NOT NULL UNIQUE,
    last_offer_at TIMESTAMP,
    created_by VARCHAR(50),
    created_date TIMESTAMP,
    last_modified_by VARCHAR(50),
    last_modified_date TIMESTAMP);

-- Create indexes
CREATE INDEX idx_radar_state_id ON radar_state(id);
CREATE INDEX idx_radar_state_user_id ON radar_state(user_id);
