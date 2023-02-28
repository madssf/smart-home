-- Add migration script here
ALTER TABLE temp_actions
ADD COLUMN starts_at TIMESTAMP;
