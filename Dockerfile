FROM rust:latest AS rust-build

WORKDIR /app_build

COPY . .

RUN apt-get update && \
    apt-get upgrade -y && \
    apt-get install -y pkg-config libssl-dev libssl3 ca-certificates && \
    rm -rf /var/lib/apt/lists/* && \
    cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y libssl3 ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /tw

COPY --from=rust-build /app_build/target/release/ddnet-exporter /tw/ddnet-exporter

ENTRYPOINT ["/tw/ddnet-exporter"]
