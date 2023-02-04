-- Add migration script here
CREATE TABLE buttons(
    id UUID NOT NULL,
    PRIMARY KEY (id),
    ip INET NOT NULL UNIQUE,
    name TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL,
    username TEXT NOT NULL
);

CREATE TABLE button_plugs(
    button_id UUID REFERENCES buttons(id) NOT NULL,
    plug_id UUID REFERENCES plugs(id) NOT NULL,
    UNIQUE (button_id, plug_id)
);
