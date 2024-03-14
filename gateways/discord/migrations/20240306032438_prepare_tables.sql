-- Add migration script here
CREATE TABLE IF NOT EXISTS discord_guilds
(
    guild_id   INT8        NOT NULL,
    purpose    TEXT        NOT NULL,
    app_id     UUID        NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (app_id, purpose),
    UNIQUE (guild_id, purpose),
    INDEX guild_id_index (guild_id)
);

CREATE TABLE IF NOT EXISTS discord_app_channels
(
    guild_id   INT8        NOT NULL,
    id         INT8        NOT NULL PRIMARY KEY,
    purpose    TEXT        NOT NULL,
    app_id     UUID        NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    INDEX discord_app_channels_app_id_idx (app_id),
    INDEX discord_app_channels_guild_id_idx (guild_id),
    UNIQUE (app_id, purpose)
);