-- Add migration script here
CREATE TABLE pokemon_modifiers
(
    battle_id           INTEGER NOT NULL,
    realised_pokemon_id INTEGER NOT NULL,
    stat                INTEGER NOT NULL,
    stages              INTEGER NOT NULL
)