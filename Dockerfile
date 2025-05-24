FROM rust:1.70-bookworm AS rust-build

WORKDIR /app

COPY src ./src
RUN cargo build --release && \
    strip target/release/ddnet-exporter


FROM alpine:latest

WORKDIR /tw

COPY --from=rust-build /app_build/target/release/ddnet-exporter /tw/ddnet-exporter

CMD ["/tw/ddnet-exporter"]