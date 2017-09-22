FROM rust:1.20.0 AS build-env

WORKDIR /usr/app
COPY . .

RUN cargo build --release

FROM debian:jessie
COPY --from=build-env /usr/app/target/release/pg-dispatcher /usr/local/sbin
