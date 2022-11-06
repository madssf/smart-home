-- Add migration script here
BEGIN;

DROP TABLE IF EXISTS schedules CASCADE;

CREATE TABLE schedules (
    id UUID NOT NULL,
    PRIMARY KEY (id),
    days TEXT[] NOT NULL
);

CREATE TABLE schedule_time_windows (
    schedule_id UUID REFERENCES schedules(id) NOT NULL,
    from_time TIME NOT NULL,
    to_time TIME NOT NULL,
    UNIQUE (from_time, schedule_id),
    UNIQUE (to_time, schedule_id)
);

CREATE TABLE schedule_temps (
    schedule_id UUID REFERENCES schedules(id) NOT NULL,
    price_level TEXT NOT NULL,
    temp DECIMAL NOT NULL,
    UNIQUE (price_level, schedule_id)
);

COMMIT;
