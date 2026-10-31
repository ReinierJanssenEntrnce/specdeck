# Signalling service image. Build from the repository root:
#   docker build -f docker/signalling.Dockerfile -t specdeck-signalling .
FROM rust:1.99-slim-trixie AS build
WORKDIR /src
COPY . .
RUN cargo build --release --locked -p specdeck-signalling

FROM debian:trixie-slim
RUN useradd --system --uid 10001 specdeck
COPY --from=build /src/target/release/specdeck-signalling /usr/local/bin/specdeck-signalling
# Numeric, so Kubernetes can check runAsNonRoot.
USER 10001
EXPOSE 8080
ENV SPECDECK_LISTEN=0.0.0.0:8080 RUST_LOG=info
ENTRYPOINT ["specdeck-signalling"]
