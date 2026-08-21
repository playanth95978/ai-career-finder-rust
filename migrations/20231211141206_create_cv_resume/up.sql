-- Create cv_resume table
CREATE TABLE cv_resume (
    id SERIAL PRIMARY KEY,
    user_id VARCHAR(255) NOT NULL UNIQUE,
    title VARCHAR(255),
    template VARCHAR(255),
    data VARCHAR(255) NOT NULL,
    version_number INTEGER NOT NULL,
    created_at TIMESTAMP,
    updated_at TIMESTAMP,
    created_by VARCHAR(50),
    created_date TIMESTAMP,
    last_modified_by VARCHAR(50),
    last_modified_date TIMESTAMP);

-- Create indexes
CREATE INDEX idx_cv_resume_id ON cv_resume(id);
CREATE INDEX idx_cv_resume_user_id ON cv_resume(user_id);
