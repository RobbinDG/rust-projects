FROM rust:1.87 as builder

WORKDIR /app

RUN apt-get update && apt-get install -y pkg-config libssl-dev build-essential

COPY Cargo.toml Rocket.toml ./
COPY .env ./
COPY src src/
COPY migrations migrations/

RUN cargo install sqlx-cli
RUN touch transactions.db
RUN cargo sqlx migrate run
RUN cargo build --release

# Runtime image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/api .
COPY --from=builder /app/.env .

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8000

CMD ["./api"]
