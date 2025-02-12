-- Add migration script here
CREATE TABLE pokemon_in_battle (
    battle_id INTEGER NOT NULL,
    realised_pokemon_id INTEGER NOT NULL,
    team INTEGER NOT NULL,
    position INTEGER NOT NULL,
    remaining_hp INTEGER NOT NULL
)