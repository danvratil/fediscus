CREATE TABLE fediverse_account (
    id BIGINT PRIMARY KEY,
    uid VARCHAR(255) NOT NULL,
    username VARCHAR(255) NOT NULL,
    instance VARCHAR(255) NOT NULL,
    inbox VARCHAR(255) NOT NULL,
    outbox VARCHAR(255) NOT NULL,
    public_key TEXT NOT NULL,
    private_key TEXT DEFAULT NULL,
    local BOOLEAN NOT NULL,
    blocked BOOLEAN NOT NULL DEFAULT FALSE,
    followed BOOLEAN NOT NULL DEFAULT FALSE -- whether the user is followed by us
);
CREATE INDEX fediverse_account_username ON fediverse_account(username);
CREATE INDEX fediverse_account_domain ON fediverse_account(instance);
CREATE UNIQUE INDEX fediverse_account_uid ON fediverse_account(uid);
CREATE UNIQUE INDEX fediverse_account_identifier ON fediverse_account(username, instance);

CREATE TABLE blog_post (
    url VARCHAR(255) PRIMARY KEY,
    fediverse_account BIGINT NOT NULL,
    fediverse_post_id BIGINT NOT NULL,
    replies_count INT NOT NULL DEFAULT 0,
    FOREIGN KEY (fediverse_account) REFERENCES fediverse_account(id)
);

CREATE INDEX blog_post_fediverse_post ON blog_post(fediverse_post_id);
