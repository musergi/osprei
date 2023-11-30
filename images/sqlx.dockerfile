FROM rust:latest as build

RUN cargo install --locked sqlx-cli --root /usr

FROM debian:latest

RUN apt-get update && apt-get install -y \
  libssl3 \
  && rm -rf /var/lib/apt/lists/*
COPY --from=build /usr/bin/sqlx /usr/bin/sqlx
