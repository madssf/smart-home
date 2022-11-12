-- Add migration script here
CREATE TABLE notification_settings (
    id int GENERATED ALWAYS AS (1) STORED UNIQUE,
    max_consumption INT,
    max_consumption_timeout_minutes INT NOT NULL,
    ntfy_topic TEXT NOT NULL
)
