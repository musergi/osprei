FROM rust:latest as build

RUN rustup component add rustfmt

WORKDIR /usr/src
COPY . .

RUN cargo install --path ./osprei-server --root /

FROM rust:latest

COPY example/config.json /etc/osprei.json
COPY --from=build /bin/osprei-server /bin/osprei-server

CMD ["/bin/osprei-server", "--config-path", "/etc/osprei.json"]
