FROM rust:1.28-slim

WORKDIR /usr/app
COPY . .

RUN apt update
RUN apt install -y pkg-config
RUN apt-get install -y libudev-dev libssl-dev

RUN cargo build --release
RUN cp -rf ./target/release/pg-dispatcher /usr/local/bin/

