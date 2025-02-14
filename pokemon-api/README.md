# Data-Centric Stateless Pokémon Battling API

## Overview

This project attempts to expose the Pokémon battling system through a GraphQL API, fulling
implemented in Rust. The API is built on a database of all required information made available
by [Veekun](https://github.com/veekun) at [GitHub](https://github.com/veekun/pokedex/tree/master)
. Additionally, any battle state can be queried and modified using plain GraphQL. This allows 
for any application to be built on top of the API to make e.g. a in-browser battle simulator or 
a battle A.I.

My personal goal for this project is to learn GraphQL and build my own API. Therefore, I do 
not want to spend too much time re-implementing Pokémon's core mechanics only for the sake of 
completeness, as this does not align with my personal goals. I do, however, want to learn 
about data modelling for GraphQL and therefore use a good share of the tables made available by 
the aforementioned git repository. Additionally, I want to learn about both queries and 
modifications in GraphQL. To that end, I will implement a functional turn simulator that allows 
2 opponents to choose their moves until one entire team is fainted.

## Features
The API currently supports the following queries and modifications.
### Queries
- Pokémon Species lookup to provide base stats, natures, abilities, and move pools.
- Move lookup.
- Battle lookup.
### Modifications
- Random Pokémon generation: creates a Pokémon with random species, nature, and 4 moves.
- Start a battle with teams of selected Pokémon.
- Play a turn in a created battle and get returned an abstract of the events occurred during 
  that turn.

## Limitations (but may be patched)

- Team sizes are currently unbounded, whereas the game requires at least 1 allows for a maximum
  of 6.
- Only singles is implemented, double battles are not yet possible.
- Pokémon to be used in battles are created at randomly and cannot be modified.
- Switching is not supported.
- Natures have no effect.
- Abilities have no effect.
- Status conditions are not implemented.
- Moves with specific effects do not work (entirely), most notably
  - Recovery is not implemented.
  - Multi-turn moves (like solar beam, wish, and perish song) do not work.
- PP tracking is not implemented
  - Consequently, struggle is not implemented either.
- Dynamic speed is not implemented (i.e. speed rules are like the earlier games)
  - Speed stat boosts are an exception to this, they do work.
- Weather, terrain, and other global effects (e.g. Trick Room and Tailwind) are not implemented.
- Held items are not implemented

## Usage
### Setup

```bash
python load_data.py  # Load primary database
cargo install sqlx-cli  # Install `sqlx`
sqlx migrate run  # Add system tables
```

### Example
<details>
<summary>Generating a random Pokémon</summary>

Input
```graphql
mutation {
  randomPokemon{
    id
    species{
      identifier
    }
    nature {
      name
    }
    move1{
      name
    }
    move2{
      name
    }
    move3{
      name
    }
    move4{
      name
    }
  }
}
```
Output
```json
{
  "data": {
    "randomPokemon": {
      "id": 9,
      "species": {
        "identifier": "bastiodon"
      },
      "nature": {
        "name": "jolly"
      },
      "move1": {
        "name": "blizzard"
      },
      "move2": {
        "name": "round"
      },
      "move3": {
        "name": "mud-slap"
      },
      "move4": {
        "name": "block"
      }
    }
  }
}
```
</details>

#### Starting a Battle
Input
```graphql
mutation {
  startBattle(teamA:[5,6], teamB:[7,9]){
    id
  }
}
```
Output
```json
{
  "data": {
    "startBattle": {
      "id": 7
    }
  }
}
```