-- Add migration script here
CREATE TABLE singles_battle (
    id INTEGER NOT NULL PRIMARY KEY,
    active_a INTEGER NOT NULL DEFAULT 0,
    active_b INTEGER NOT NULL DEFAULT 0,
    turn_nr INTEGER NOT NULL DEFAULT 1
)