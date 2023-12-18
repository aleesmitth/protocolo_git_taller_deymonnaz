-- Create the sequence if it doesn't exist
CREATE SEQUENCE IF NOT EXISTS pull_request_surrogate;

CREATE TABLE pull_requests (
    _id INT DEFAULT nextval('pull_request_surrogate'),
    name VARCHAR(255) NOT NULL,
    repo VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    head VARCHAR(255),  -- Add the "head" column
    base VARCHAR(255),  -- Add the "base" column
    PRIMARY KEY (_id),
    UNIQUE(name, repo)
);
