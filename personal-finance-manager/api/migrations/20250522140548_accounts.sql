-- Add migration script here
CREATE TABLE accounts (
    iban VARCHAR PRIMARY KEY NOT NULL,
    holder VARCHAR NOT NULL,
    name VARCHAR NOT NULL
)