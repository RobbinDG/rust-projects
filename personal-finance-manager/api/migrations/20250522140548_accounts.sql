-- Add migration script here
CREATE TABLE accounts (
    iban VARCHAR PRIMARY KEY,
    holder VARCHAR NOT NULL,
    name VARCHAR NOT NULL
)