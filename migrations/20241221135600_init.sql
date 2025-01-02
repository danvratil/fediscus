CREATE TABLE accounts (
    id BIGINT PRIMARY KEY NOT NULL,
    created_at DATETIME DEFAULT (DATETIME('now')) NOT NULL,
    updated_at DATETIME DEFAULT (DATETIME('now')) NOT NULL,
    uri VARCHAR(255) NOT NULL UNIQUE,
    username VARCHAR(255) NOT NULL,
    host VARCHAR(255) NOT NULL,
    inbox VARCHAR(255) NOT NULL,
    outbox VARCHAR(255) NOT NULL,
    shared_inbox VARCHAR(255) NOT NULL,
    public_key TEXT NOT NULL,
    private_key TEXT DEFAULT NULL,
    local BOOLEAN NOT NULL,

    UNIQUE (username, host)
);
CREATE INDEX accounts_username ON accounts(username);

CREATE TABLE follows (
    id BIGINT PRIMARY KEY NOT NULL,
    created_at DATETIME DEFAULT (DATETIME('now')) NOT NULL,
    account_id BIGINT NOT NULL, -- who is following
    target_account_id BIGINT NOT NULL, -- who is being followed
    uri VARCHAR(255) NOT NULL,

    FOREIGN KEY (account_id) REFERENCES accounts(id),
    FOREIGN KEY (target_account_id) REFERENCES accounts(id),
    UNIQUE (account_id, target_account_id)
);

CREATE TABLE follow_requests (
    id BIGINT PRIMARY KEY NOT NULL,
    created_at DATETIME DEFAULT (DATETIME('now')) NOT NULL,
    account_id BIGINT NOT NULL, -- who is following
    target_account_id BIGINT NOT NULL, -- who is being followed
    uri VARCHAR(255) NOT NULL,

    FOREIGN KEY (account_id) REFERENCES accounts(id),
    FOREIGN KEY (target_account_id) REFERENCES accounts(id),
    UNIQUE (account_id, target_account_id)
);