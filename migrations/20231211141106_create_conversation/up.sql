-- Create conversation table
CREATE TABLE conversation (
    id SERIAL PRIMARY KEY,
    user_id VARCHAR(255) NOT NULL,
    title VARCHAR(255),
    summary VARCHAR(255),
    metadata VARCHAR(255),
    type_chat VARCHAR(255),
    created_at TIMESTAMP NOT NULL,
    last_message_at TIMESTAMP,
    created_by VARCHAR(50),
    created_date TIMESTAMP,
    last_modified_by VARCHAR(50),
    last_modified_date TIMESTAMP);

-- Create indexes
CREATE INDEX idx_conversation_id ON conversation(id);
