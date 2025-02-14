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

## Tech-stack

- GraphQL (async-graphql crate)
- SQLite (sqlx crate)
- Poem

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

# Build and run the server.
cargo build
cargo run 
```

When the application starts, the server is hosted at `localhost:8000`. This is not configurable
without code modification. `POST` requests can be used to make GraphQL requests, whereas `GET`
requests will return a GraphiQL web interface.

### Examples

<details>
<summary>Generating a random Pokémon</summary>

#### Input

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

#### Output

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

<details>
<summary>Starting a Battle</summary>

#### Input

```graphql
mutation {
  startBattle(teamA:[5,6], teamB:[7,9]){
    id
  }
}
```

#### Output

```json
{
  "data": {
	"startBattle": {
	  "id": 7
	}
  }
}
```

</details>

<details>
<summary>Getting the current battle state</summary>

#### Input

```graphql
{
  battle(id:7){
    activeA
    activeB
    teamA{
      stat(stat:ATK)
      pokemon{
        species{
          identifier
          pkmStats{
            atk{baseStat}
            spd{
              baseStat
            }
            hp{
              baseStat
            }
          }
        }
        move1{
          name
          accuracy
          power
        }
        move2{
          name
          accuracy
          power
        }
        move3{
          name
          accuracy
          power
        }
        move4{
          name
          accuracy
          power
        }
      }
      remainingHp
    }
    teamB{
      stat(stat:ATK)
      pokemon{
        species{
          identifier
          pkmStats{
            atk{baseStat}
            spd{
              baseStat
            }
            hp{
              baseStat
            }
          }
        }
        move1{
          name
          power
        }
        move2{
          name
          power
        }
        move3{
          name
          power
        }
        move4{
          name
          power
        }
      }
      remainingHp
    }
  }
}
```

#### Output

```json
{
  "data": {
	"battle": {
	  "activeA": 0,
	  "activeB": 0,
	  "teamA": [
		{
		  "stat": 95,
		  "pokemon": {
			"species": {
			  "identifier": "cinccino",
			  "pkmStats": {
				"atk": {
				  "baseStat": 95
				},
				"spd": {
				  "baseStat": 115
				},
				"hp": {
				  "baseStat": 75
				}
			  }
			},
			"move1": {
			  "name": "fling",
			  "accuracy": 100,
			  "power": null
			},
			"move2": {
			  "name": "thunder",
			  "accuracy": 70,
			  "power": 110
			},
			"move3": {
			  "name": "rest",
			  "accuracy": null,
			  "power": null
			},
			"move4": {
			  "name": "toxic",
			  "accuracy": 90,
			  "power": null
			}
		  },
		  "remainingHp": 75
		},
		{
		  "stat": 120,
		  "pokemon": {
			"species": {
			  "identifier": "luxray",
			  "pkmStats": {
				"atk": {
				  "baseStat": 120
				},
				"spd": {
				  "baseStat": 70
				},
				"hp": {
				  "baseStat": 80
				}
			  }
			},
			"move1": {
			  "name": "light-screen",
			  "accuracy": null,
			  "power": null
			},
			"move2": {
			  "name": "substitute",
			  "accuracy": null,
			  "power": null
			},
			"move3": {
			  "name": "substitute",
			  "accuracy": null,
			  "power": null
			},
			"move4": {
			  "name": "flash",
			  "accuracy": 100,
			  "power": null
			}
		  },
		  "remainingHp": 80
		}
	  ],
	  "teamB": [
		{
		  "stat": 73,
		  "pokemon": {
			"species": {
			  "identifier": "swalot",
			  "pkmStats": {
				"atk": {
				  "baseStat": 73
				},
				"spd": {
				  "baseStat": 55
				},
				"hp": {
				  "baseStat": 100
				}
			  }
			},
			"move1": {
			  "name": "water-pulse",
			  "power": 60
			},
			"move2": {
			  "name": "thunder-punch",
			  "power": 75
			},
			"move3": {
			  "name": "sleep-talk",
			  "power": null
			},
			"move4": {
			  "name": "substitute",
			  "power": null
			}
		  },
		  "remainingHp": 100
		},
		{
		  "stat": 52,
		  "pokemon": {
			"species": {
			  "identifier": "bastiodon",
			  "pkmStats": {
				"atk": {
				  "baseStat": 52
				},
				"spd": {
				  "baseStat": 30
				},
				"hp": {
				  "baseStat": 60
				}
			  }
			},
			"move1": {
			  "name": "blizzard",
			  "power": 110
			},
			"move2": {
			  "name": "round",
			  "power": 60
			},
			"move3": {
			  "name": "mud-slap",
			  "power": 20
			},
			"move4": {
			  "name": "block",
			  "power": null
			}
		  },
		  "remainingHp": 60
		}
	  ]
	}
  }
}
```

</details>

<details>
<summary>Playing a turn</summary>

#### Input

```graphql
mutation{
  playTurn(id: 7, moveA: SELECT_MOVE_2, moveB: SWITCH_2){
    switchPhase{
      side,
      into
    }
    attackPhase{
      type
      damageDealt
      effectTriggered
    }
  }
}
```

#### Output

```json
{
  "data": {
	"playTurn": {
	  "switchPhase": [
		{
		  "side": "TEAM_B",
		  "into": 1
		}
	  ],
	  "attackPhase": [
		{
		  "type": "DAMAGE",
		  "damageDealt": 22,
		  "effectTriggered": false
		}
	  ]
	}
  }
}
```

#### Updated battle state output

```json
{
  "data": {
	"battle": {
	  "activeA": 0,
	  "activeB": 1,
	  "teamA": [
		{
		  "stat": 95,
		  "pokemon": {
			"species": {
			  "identifier": "cinccino",
			  "pkmStats": {
				"atk": {
				  "baseStat": 95
				},
				"spd": {
				  "baseStat": 115
				},
				"hp": {
				  "baseStat": 75
				}
			  }
			},
			"move1": {
			  "name": "fling",
			  "accuracy": 100,
			  "power": null
			},
			"move2": {
			  "name": "thunder",
			  "accuracy": 70,
			  "power": 110
			},
			"move3": {
			  "name": "rest",
			  "accuracy": null,
			  "power": null
			},
			"move4": {
			  "name": "toxic",
			  "accuracy": 90,
			  "power": null
			}
		  },
		  "remainingHp": 75
		},
		{
		  "stat": 120,
		  "pokemon": {
			"species": {
			  "identifier": "luxray",
			  "pkmStats": {
				"atk": {
				  "baseStat": 120
				},
				"spd": {
				  "baseStat": 70
				},
				"hp": {
				  "baseStat": 80
				}
			  }
			},
			"move1": {
			  "name": "light-screen",
			  "accuracy": null,
			  "power": null
			},
			"move2": {
			  "name": "substitute",
			  "accuracy": null,
			  "power": null
			},
			"move3": {
			  "name": "substitute",
			  "accuracy": null,
			  "power": null
			},
			"move4": {
			  "name": "flash",
			  "accuracy": 100,
			  "power": null
			}
		  },
		  "remainingHp": 80
		}
	  ],
	  "teamB": [
		{
		  "stat": 73,
		  "pokemon": {
			"species": {
			  "identifier": "swalot",
			  "pkmStats": {
				"atk": {
				  "baseStat": 73
				},
				"spd": {
				  "baseStat": 55
				},
				"hp": {
				  "baseStat": 100
				}
			  }
			},
			"move1": {
			  "name": "water-pulse",
			  "power": 60
			},
			"move2": {
			  "name": "thunder-punch",
			  "power": 75
			},
			"move3": {
			  "name": "sleep-talk",
			  "power": null
			},
			"move4": {
			  "name": "substitute",
			  "power": null
			}
		  },
		  "remainingHp": 100
		},
		{
		  "stat": 52,
		  "pokemon": {
			"species": {
			  "identifier": "bastiodon",
			  "pkmStats": {
				"atk": {
				  "baseStat": 52
				},
				"spd": {
				  "baseStat": 30
				},
				"hp": {
				  "baseStat": 60
				}
			  }
			},
			"move1": {
			  "name": "blizzard",
			  "power": 110
			},
			"move2": {
			  "name": "round",
			  "power": 60
			},
			"move3": {
			  "name": "mud-slap",
			  "power": 20
			},
			"move4": {
			  "name": "block",
			  "power": null
			}
		  },
		  "remainingHp": 38
		}
	  ]
	}
  }
}
```

</details>