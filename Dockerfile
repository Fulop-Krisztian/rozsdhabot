FROM rust:latest AS build
WORKDIR /src

COPY . .
RUN cargo build --release


FROM debian:trixie-slim
WORKDIR /rozsdhabot

RUN <<EOF
apt-get update 
apt-get install -y --no-install-recommends ca-certificates
rm -rf /var/lib/apt/lists/*
EOF

COPY ./coconut.jpg /rozsdhabot/
COPY --from=build /src/target/release/rozsdhabot /rozsdhabot/
CMD ["/rozsdhabot/rozsdhabot"]



