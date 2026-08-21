-- Create cv_resume_version table
CREATE TABLE cv_resume_version (
    id SERIAL PRIMARY KEY,
    version_number INTEGER NOT NULL,
    title VARCHAR(255),
    template VARCHAR(255),
    data VARCHAR(255) NOT NULL,
    created_at TIMESTAMP,
    resume_id INTEGER REFERENCES cv_resume(id),
    created_by VARCHAR(50),
    created_date TIMESTAMP,
    last_modified_by VARCHAR(50),
    last_modified_date TIMESTAMP);

-- Create indexes
CREATE INDEX idx_cv_resume_version_id ON cv_resume_version(id);
CREATE INDEX idx_cv_resume_version_resume_id ON cv_resume_version(resume_id);
