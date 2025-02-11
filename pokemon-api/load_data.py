import sqlite3
from sqlite3 import Connection

import pandas as pd

data_sources = {
    'stats': 'https://raw.githubusercontent.com/veekun/pokedex/refs/heads/master/pokedex/data/csv/stats.csv',
    'types': 'https://raw.githubusercontent.com/veekun/pokedex/refs/heads/master/pokedex/data/csv/types.csv',
    'type_efficacy': 'https://raw.githubusercontent.com/veekun/pokedex/refs/heads/master/pokedex/data/csv/type_efficacy.csv',
    'moves': 'https://raw.githubusercontent.com/veekun/pokedex/refs/heads/master/pokedex/data/csv/moves.csv',
    'move_names': 'https://raw.githubusercontent.com/veekun/pokedex/refs/heads/master/pokedex/data/csv/move_names.csv',
    'move_effects': 'https://raw.githubusercontent.com/veekun/pokedex/refs/heads/master/pokedex/data/csv/move_effect_prose.csv',
    'pokemon_moves': 'https://raw.githubusercontent.com/veekun/pokedex/refs/heads/master/pokedex/data/csv/pokemon_moves.csv',
    'pokemon_species': 'https://raw.githubusercontent.com/veekun/pokedex/refs/heads/master/pokedex/data/csv/pokemon_species.csv',
    'pokemon_stats': 'https://raw.githubusercontent.com/veekun/pokedex/refs/heads/master/pokedex/data/csv/pokemon_stats.csv',
    'pokemon_types': 'https://raw.githubusercontent.com/veekun/pokedex/refs/heads/master/pokedex/data/csv/pokemon_types.csv',
    'languages': 'https://raw.githubusercontent.com/veekun/pokedex/refs/heads/master/pokedex/data/csv/languages.csv',
    'type_names': 'https://raw.githubusercontent.com/veekun/pokedex/refs/heads/master/pokedex/data/csv/type_names.csv',
}


def create_table_definition(table_name: str, df: pd.DataFrame, conn: Connection):
    non_null_columns = [col for col in df.columns if not df[col].isnull().any()]

    cursor = conn.cursor()

    # Construct CREATE TABLE statement with NOT NULL constraints
    columns_definitions = []

    for col, dtype in df.dtypes.items():
        sql_type = "TEXT"  # Default type (you may refine this based on dtype)
        if pd.api.types.is_numeric_dtype(dtype):
            if (df[col].dropna() % 1 == 0).all():
                dtype = pd.Int64Dtype()  # Convert to nullable integer
            sql_type = "INTEGER" if pd.api.types.is_integer_dtype(dtype) else "REAL"

        not_null = "NOT NULL" if col in non_null_columns else ""
        columns_definitions.append(f'"{col}" {sql_type} {not_null}')

    create_table_query = f"CREATE TABLE {table_name} ({', '.join(columns_definitions)});"
    cursor.execute(create_table_query)


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
        create_table_definition(table_name, df, conn)
        df.to_sql(table_name, conn, if_exists='append', index=False)

    conn.commit()
