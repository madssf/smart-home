-- Add migration script here
ALTER TABLE plugs
ADD COLUMN scheduled BOOLEAN;
UPDATE plugs SET scheduled = TRUE;
ALTER TABLE plugs ALTER COLUMN scheduled SET NOT NULL;
