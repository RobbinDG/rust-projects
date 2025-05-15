-- Add migration script here
CREATE TABLE party_categories (
    party_name VARCHAR PRIMARY KEY NOT NULL,
    category VARCHAR NOT NULL
)