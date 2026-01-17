# syntax=docker/dockerfile:1.4
# docker buildx build --platform linux/amd64,linux/arm64 -t krissssz/rozsdhabot --push .

# === BUILD STAGE
FROM rust:latest AS build
WORKDIR /src

COPY ./src ./src
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
RUN cargo build --release


# === RELEASE ===
FROM debian:trixie-slim as release
WORKDIR /rozsdhabot

RUN <<EOF
apt-get update 
apt-get install -y --no-install-recommends ca-certificates
rm -rf /var/lib/apt/lists/*
EOF

COPY ./coconut.jpg /rozsdhabot/
COPY --from=build /src/target/release/rozsdhabot /rozsdhabot/
CMD ["/rozsdhabot/rozsdhabot"]



