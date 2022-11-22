-- Add migration script here
ALTER TABLE temp_sensors
ADD COLUMN battery_level INT
