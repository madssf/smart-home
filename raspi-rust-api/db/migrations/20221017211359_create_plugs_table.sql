-- Add migration script here
CREATE TABLE plugs(
                              id uuid NOT NULL,
                              PRIMARY KEY (id),
                              ip INET NOT NULL UNIQUE,
                              name TEXT NOT NULL UNIQUE,
                              password TEXT NOT NULL,
                              username TEXT NOT NULL
);
