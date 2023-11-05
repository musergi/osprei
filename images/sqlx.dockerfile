FROM rust:latest

RUN cargo install --locked sqlx-cli
