FROM rust:1.64
RUN apt-get update
RUN apt-get install -y musl-tools zip
RUN rustup target add x86_64-unknown-linux-musl
