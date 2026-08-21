-- Create authorities table
CREATE TABLE authorities (
    name VARCHAR(50) PRIMARY KEY NOT NULL
);

-- Create users table
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    login VARCHAR(50) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    first_name VARCHAR(50),
    last_name VARCHAR(50),
    email VARCHAR(191) NOT NULL UNIQUE,
    activated BOOLEAN NOT NULL DEFAULT FALSE,
    lang_key VARCHAR(10),
    image_url VARCHAR(256),
    created_by VARCHAR(50),
    created_date TIMESTAMP,
    last_modified_by VARCHAR(50),
    last_modified_date TIMESTAMP
);

-- Create user_authorities join table
CREATE TABLE user_authorities (
    user_id INTEGER NOT NULL,
    authority_name VARCHAR(50) NOT NULL,
    PRIMARY KEY (user_id, authority_name),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (authority_name) REFERENCES authorities(name)
);

-- Create indexes
CREATE INDEX idx_users_login ON users(login);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_user_authorities_user_id ON user_authorities(user_id);

-- Insert default authorities
INSERT INTO authorities (name) VALUES ('ROLE_ADMIN');
INSERT INTO authorities (name) VALUES ('ROLE_USER');

-- Insert default admin user (password: admin)
-- Password hash for 'admin' using argon2
INSERT INTO users (login, password_hash, first_name, last_name, email, activated, lang_key, created_by, created_date)
VALUES (
    'admin',
    '$argon2id$v=19$m=19456,t=2,p=1$8BBmYkjL9OBiomwlcU11qA$hK8/3elLjKnt4mI7Tz+Q2AzU58pZqdpL1YnE0UEhjHE',
    'Administrator',
    'Administrator',
    'admin@localhost',
    TRUE,
    'en',
    'system',
    CURRENT_TIMESTAMP
);

-- Insert default user (password: user)
INSERT INTO users (login, password_hash, first_name, last_name, email, activated, lang_key, created_by, created_date)
VALUES (
    'user',
    '$argon2id$v=19$m=19456,t=2,p=1$f2qoLPEpsP79MNnV/DgLTw$sQ8ZzGZXDVbaHl9PxE9QzyHGaF9O+KsHp+2JbFRNf64',
    'User',
    'User',
    'user@localhost',
    TRUE,
    'en',
    'system',
    CURRENT_TIMESTAMP
);

-- Assign roles to users
INSERT INTO user_authorities (user_id, authority_name) VALUES (1, 'ROLE_ADMIN');
INSERT INTO user_authorities (user_id, authority_name) VALUES (1, 'ROLE_USER');
INSERT INTO user_authorities (user_id, authority_name) VALUES (2, 'ROLE_USER');
