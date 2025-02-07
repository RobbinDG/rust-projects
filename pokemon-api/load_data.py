import sqlite3

import pandas as pd

data_sources = {
    'stats': 'https://raw.githubusercontent.com/veekun/pokedex/refs/heads/master/pokedex/data/csv/stats.csv',
    'types': 'https://raw.githubusercontent.com/veekun/pokedex/refs/heads/master/pokedex/data/csv/types.csv',
    'type_efficacy': 'https://raw.githubusercontent.com/veekun/pokedex/refs/heads/master/pokedex/data/csv/type_efficacy.csv',
    'pokemon_species': 'https://raw.githubusercontent.com/veekun/pokedex/refs/heads/master/pokedex/data/csv/pokemon_species.csv',
    'pokemon_stats': 'https://raw.githubusercontent.com/veekun/pokedex/refs/heads/master/pokedex/data/csv/pokemon_stats.csv',
    'pokemon_types': 'https://raw.githubusercontent.com/veekun/pokedex/refs/heads/master/pokedex/data/csv/pokemon_types.csv',
    'languages': 'https://raw.githubusercontent.com/veekun/pokedex/refs/heads/master/pokedex/data/csv/languages.csv',
    'type_names': 'https://raw.githubusercontent.com/veekun/pokedex/refs/heads/master/pokedex/data/csv/type_names.csv',
}

with sqlite3.connect('pokemon.db') as conn:
    cur = conn.cursor()

    # Create data source table, in case the DB file gets shared.
    cur.execute('''
        CREATE TABLE IF NOT EXISTS data_sources (
            table_name VARCHAR PRIMARY KEY NOT NULL, 
            url VARCHAR NOT NULL
        );
    ''')

    # Load missing data sources
    for table_name, url in data_sources.items():
        # Check if data exists in DB. Skip if so, to avoid rate limiting.
        res = cur.execute(
            f"SELECT name FROM sqlite_master WHERE type='table' AND name='{table_name}';")

        if len(res.fetchall()) > 0:
            continue

        # Register data source
        cur.execute("""
            INSERT OR IGNORE INTO data_sources VALUES (?, ?) 
        """, [table_name, url])

        # Load data into DB
        df = pd.read_csv(url)
        df.to_sql(table_name, conn, if_exists='replace', index=False)

    conn.commit()
