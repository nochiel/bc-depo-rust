# https://hub.docker.com/_/rust
FROM rust:latest AS dependencies

RUN apt update
RUN apt install -y curl build-essential pkg-config openssl libssl-dev

WORKDIR /depo
# COPY . .

FROM dependencies

EXPOSE 5332

# RUN cargo build
