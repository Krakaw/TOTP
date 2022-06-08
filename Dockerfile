FROM rust:1.61.0-bullseye as builder
WORKDIR /usr/src/totp
RUN sed -i "s#http://deb.debian.org/#https://debian.mirror.ac.za/#g" /etc/apt/sources.list
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    gcc \
    libc6-dev \
    wget \
    git \
    pkg-config \
    libssl-dev \
    llvm \
    libclang-dev \
    clang \
    libssl-dev \
    libxcb-shape0-dev \ 
    libxcb-xfixes0-dev \
    && rm -rf /var/lib/apt/lists/*
COPY Cargo.lock .
COPY Cargo.toml .
RUN echo "pub fn main() {}" >> dummy.rs \
    && touch dummy_lib.rs \
    && sed -i 's#src/main.rs#dummy.rs#' Cargo.toml \
    && sed -i 's#src/lib.rs#dummy_lib.rs#' Cargo.toml \
    && cargo build --no-default-features --release \
    && sed -i 's#dummy.rs#src/main.rs#' Cargo.toml \
    && sed -i 's#dummy_lib.rs#src/lib.rs#' Cargo.toml
COPY ./src src
RUN cargo build --no-default-features --release


FROM debian:bullseye-slim
RUN sed -i "s#http://deb.debian.org/#https://debian.mirror.ac.za/#g" /etc/apt/sources.list && apt-get update
RUN apt-get install -y openssl && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/totp/target/release/totp /usr/local/bin/totp
EXPOSE 8080