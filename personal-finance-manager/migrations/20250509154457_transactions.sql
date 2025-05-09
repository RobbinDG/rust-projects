-- Add migration script here
CREATE TABLE transactions (
    IBAN VARCHAR,
    currency VARCHAR,
    BIC VARCHAR,
    MTCN INTEGER PRIMARY KEY,  -- Money Transfer Control Number
    date DATE,
    interest_date DATE,
    value FLOAT,
    balance_after FLOAT,
    IBAN_other VARCHAR,
    name_other VARCHAR,
    BIC_other VARCHAR,
    code VARCHAR,
    reference VARCHAR,
    description VARCHAR,
    value_orig FLOAT,
    currency_orig VARCHAR,
    exchange_rate FLOAT
)