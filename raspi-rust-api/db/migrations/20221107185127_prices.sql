-- Add migration script here
CREATE TABLE prices (
    starts_at TIMESTAMP,
    PRIMARY KEY (starts_at),
    amount DECIMAL NOT NULL,
    currency TEXT NOT NULL,
    ext_price_level TEXT NOT NULL,
    price_level TEXT
)
