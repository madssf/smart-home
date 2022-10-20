-- Add migration script here
CREATE TABLE rooms(
    id UUID NOT NULL,
    PRIMARY KEY (id),
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE plugs(
    id UUID NOT NULL,
    PRIMARY KEY (id),
    ip INET NOT NULL UNIQUE,
    name TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL,
    username TEXT NOT NULL,
    room_id UUID REFERENCES rooms(id) NOT NULL
);

CREATE TABLE schedules(
    id UUID NOT NULL,
    PRIMARY KEY (id),
    temp DECIMAL NOT NULL,
    price_level TEXT NOT NULL,
    days TEXT[] NOT NULL,
    time_windows TEXT[] NOT NULL
);

CREATE TABLE room_schedules(
    room_id UUID REFERENCES rooms(id) NOT NULL,
    schedule_id UUID REFERENCES schedules(id) NOT NULL,
    UNIQUE (room_id, schedule_id)
);

CREATE TABLE temp_actions(
    id UUID NOT NULL,
    PRIMARY KEY (id),
    room_ids UUID[] NOT NULL,
    action TEXT NOT NULL,
    expires_at TIMESTAMP NOT NULL
);

CREATE TABLE temperature_logs(
    room_id UUID REFERENCES rooms(id) NOT NULL,
    time TIMESTAMP NOT NULL,
    temp DECIMAL NOT NULL,
    UNIQUE (room_id, time)
);
