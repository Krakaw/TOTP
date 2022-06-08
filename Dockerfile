FROM rust:1.61.0-slim-bullseye as builder
WORKDIR /usr/src/totp
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxcb1-dev libssl-dev ca-certificates wget gcc libc6-dev cmake && rm -rf /var/lib/apt/lists/*
COPY . .
RUN cargo build --release


FROM debian:buster-slim
RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/totp/target/release/totp /usr/local/bin/totp
EXPOSE 8080
