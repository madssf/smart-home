-- Add migration script here
BEGIN;

DROP TABLE IF EXISTS temp_actions CASCADE;

CREATE TABLE temp_actions
(
    id         UUID      NOT NULL,
    PRIMARY KEY (id),
    room_ids   UUID[]    NOT NULL,
    action     TEXT      NOT NULL,
    temp       DECIMAL,
    expires_at TIMESTAMP NOT NULL
);

COMMIT;
