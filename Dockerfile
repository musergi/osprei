FROM rust:latest

RUN rustup component add rustfmt

WORKDIR /usr/src
COPY . .
COPY example/config.json /etc/osprei.json

RUN cargo install --path ./osprei-server
RUN cargo install --path ./osprei-cli

CMD ["osprei-server", "--config-path", "/etc/osprei.json"]
