-- Create the sequence if it doesn't exist
CREATE SEQUENCE IF NOT EXISTS pull_request_surrogate;

CREATE TABLE pull_requests (
    _id INT DEFAULT nextval('pull_request_surrogate') NOT NULL,
    repo VARCHAR(255) NOT NULL,
    head VARCHAR(255) NOT NULL,
    base VARCHAR(255) NOT NULL,
    commit_after_merge VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    PRIMARY KEY(repo, head, base),
    UNIQUE(_id)
);
