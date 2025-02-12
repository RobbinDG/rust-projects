## Setup
```bash
python load_data.py  # Load primary database
cargo install sqlx-cli  # Install `sqlx`
sqlx migrate run  # Add system tables
```

### TODO
- [ ] Integrate nature in damage calculation.
- [ ] Add ability to realised pokemon