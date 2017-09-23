FROM rust:1.20.0

RUN apt-get update && apt-get install -y libssl-dev openssl
WORKDIR /usr/app
COPY . .

RUN cargo build --release
RUN cp -rf ./target/release/pg-dispatcher /usr/local/bin/

