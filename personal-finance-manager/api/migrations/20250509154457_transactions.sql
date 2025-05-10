-- Add migration script here
CREATE TABLE transactions
(
    IBAN          VARCHAR             NOT NULL,
    currency      VARCHAR             NOT NULL,
    BIC           VARCHAR             NOT NULL,
    MTCN          INTEGER PRIMARY KEY NOT NULL, -- Money Transfer Control Number
    date          DATE                NOT NULL,
    interest_date DATE                NOT NULL,
    value         FLOAT               NOT NULL,
    balance_after FLOAT               NOT NULL,
    IBAN_other    VARCHAR,
    name_other    VARCHAR             NOT NULL,
    BIC_other     VARCHAR,
    code          VARCHAR,
    reference     VARCHAR,
    description   VARCHAR,
    value_orig    FLOAT,
    currency_orig VARCHAR,
    exchange_rate FLOAT
);

CREATE INDEX idx_transaction_date ON transactions (date);