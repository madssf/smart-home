-- Add migration script here
CREATE TABLE temp_sensors (
    id TEXT NOT NULL,
    PRIMARY KEY (id),
    room_id UUID REFERENCES rooms(id) NOT NULL
)
