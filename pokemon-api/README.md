# Data-Centric Stateless Pokémon Battling API

## Overview

This project attempts to expose the Pokémon battling system through a GraphQL API, fulling
implemented in Rust. The API is built on a database of all required information made available
by [Veekun](https://github.com/veekun) at [GitHub](https://github.com/veekun/pokedex/tree/master)
. Additionally, any battle state can be queried and modified using plain GraphQL. This allows 
for any application to be built on top of the API to make e.g. a in-browser battle simulator.

## Features
The API currently supports the following queries and modifications.
### Queries
- Pokémon Species lookup to provide base stats, natures, abilities, and move pools.
- Move lookup.
- Battle lookup.
### Modifications
- Random Pokémon generation: creates a Pokémon with random species, nature, and 4 moves.

## Setup

```bash
python load_data.py  # Load primary database
cargo install sqlx-cli  # Install `sqlx`
sqlx migrate run  # Add system tables
```

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
  - Stat changes are not currently implemented (as primary or secondary effect)
  - Recovery is not implemented.
  - Multi-turn moves (like wish and perish song) do not work.
- PP tracking is not implemented
  - Consequently, struggle is not implemented either.
- Dynamic speed is not implemented (i.e. speed rules are like the earlier games)
- Weather, terrain, and other global effects (e.g. Trick Room and Tailwind) are not implemented.
- Held items are not implemented