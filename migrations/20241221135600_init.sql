-- SPDX-FileCopyrightText: 2025 Daniel Vrátil <me@dvratil.cz>
--
-- SPDX-License-Identifier: MIT

CREATE TABLE accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    created_at DATETIME DEFAULT (DATETIME('now')) NOT NULL,
    updated_at DATETIME DEFAULT (DATETIME('now')) NOT NULL,
    uri VARCHAR(255) NOT NULL UNIQUE,
    username VARCHAR(255) NOT NULL,
    host VARCHAR(255) NOT NULL,
    inbox VARCHAR(255) NOT NULL,
    outbox VARCHAR(255) NOT NULL,
    shared_inbox VARCHAR(255) NULL,
    public_key TEXT NOT NULL,
    private_key TEXT DEFAULT NULL,
    local BOOLEAN NOT NULL,

    UNIQUE (username, host)
);
CREATE INDEX accounts_username ON accounts(username);

CREATE TABLE follows (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    created_at DATETIME DEFAULT (DATETIME('now')) NOT NULL,
    account_id INTEGER NOT NULL, -- who is following
    target_account_id INTEGER NOT NULL, -- who is being followed
    pending BOOLEAN NOT NULL, -- pending acceptance
    uri VARCHAR(255) NOT NULL,

    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE,
    FOREIGN KEY (target_account_id) REFERENCES accounts(id) ON DELETE CASCADE,
    UNIQUE (account_id, target_account_id)
);

CREATE TABLE blogs (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    created_at DATETIME DEFAULT (DATETIME('now')) NOT NULL,
    url VARCHAR(255) NOT NULL,

    UNIQUE (url)
);

CREATE TABLE notes (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    created_at DAETTIME DEFAULT (DATETIME('now')) NOT NULL,
    updated_at DATETIME DEFAULT (DATETIME('now')) NOT NULL,
    account_id INTEGER NOT NULL, -- author
    uri VARCHAR(255) NOT NULL,
    reply_to_id INTEGER NULL, -- reply to
    root_id INTEGER NULL, -- root post
    blog_id INTEGER NOT NULL, -- blog post

    FOREIGN KEY (account_id) REFERENCES accounts(id) ON DELETE CASCADE,
    FOREIGN KEY (reply_to_id) REFERENCES posts(id) ON DELETE CASCADE,
    FOREIGN KEY (root_id) REFERENCES posts(id) ON DELETE CASCADE,
    FOREIGN KEY (blog_id) REFERENCES blogs(id) ON DELETE CASCADE
);