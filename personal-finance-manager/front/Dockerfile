FROM rust:1.87 as builder

# Install Trunk and other dependencies
RUN cargo install trunk wasm-bindgen-cli
RUN apt-get update && apt-get install -y libssl-dev pkg-config

WORKDIR /app
COPY . ./

RUN rustup target add wasm32-unknown-unknown

# Install tailwind
RUN apt-get install -y npm

# Build the Yew frontend
RUN trunk build --release

# Use a minimal web server to serve static files
FROM nginx:alpine

COPY --from=builder /app/dist /usr/share/nginx/html
