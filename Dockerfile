RUN apt-get update && \
    apt-get upgrade -y && \
    apt-get install pkg-config libssl-dev libssl3 ca-certificates -y && \
    rm -rf /var/lib/apt/lists/* && \
    cargo build --release

COPY . .

RUN apt-get update && \
    apt-get upgrade -y && \
    apt-get install pkg-config libssl-dev libssl3 ca-certificates -y && \
    rm -rf /var/lib/apt/lists/* && \
    cargo build --release

FROM debian:bookworm-slim

WORKDIR /tw

COPY --from=rust-build /app_build/target/release/ddnet-exporter /tw/ddnet-exporter

ENTRYPOINT ["/tw/ddnet-exporter"]