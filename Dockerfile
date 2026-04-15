FROM rust:1.94.1

ARG TARGET_TRIPLE=x86_64-unknown-linux-musl

RUN apt-get update \
	&& apt-get install -y --no-install-recommends musl-tools zip build-essential pkg-config perl \
	&& rm -rf /var/lib/apt/lists/*

RUN rustup target add ${TARGET_TRIPLE}
