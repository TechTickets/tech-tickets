-- Add migration script here
CREATE TABLE IF NOT EXISTS tt_user
(
    id         INT8 PRIMARY KEY,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS app
(
    id         UUID PRIMARY KEY,
    name       TEXT        NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    owner_id   INT8,
    UNIQUE (name),
    FOREIGN KEY (owner_id) REFERENCES tt_user (id)
);

CREATE TABLE IF NOT EXISTS user_app
(
    user_id INT8 NOT NULL,
    app_id  UUID NOT NULL,
    role    TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES tt_user (id),
    FOREIGN KEY (app_id) REFERENCES app (id),
    PRIMARY KEY (user_id, app_id)
);

CREATE TABLE IF NOT EXISTS gateway
(
    name       TEXT        NOT NULL,
    enabled    BOOLEAN     NOT NULL DEFAULT TRUE,
    app_id     UUID        NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY (app_id) REFERENCES app (id),
    PRIMARY KEY (app_id, name)
);

CREATE INDEX IF NOT EXISTS gateway_app_id_index ON gateway (app_id);

CREATE TABLE IF NOT EXISTS ticket
(
    id         UUID PRIMARY KEY,
    app_id     UUID        NOT NULL,
    message    TEXT        NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    gateway    TEXT        NOT NULL,
    FOREIGN KEY (app_id) REFERENCES app (id)
);

CREATE INDEX IF NOT EXISTS ticket_app_id_index ON ticket (app_id);
