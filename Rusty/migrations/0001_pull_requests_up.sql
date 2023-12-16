-- Create the sequence if it doesn't exist
CREATE SEQUENCE IF NOT EXISTS pull_request_surrogate;
CREATE TABLE pull_requests (
                       id INT DEFAULT nextval('pull_request_surrogate') PRIMARY KEY,
                       name VARCHAR(255) NOT NULL,
                       created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
