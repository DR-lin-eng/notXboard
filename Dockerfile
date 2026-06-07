FROM rust:1.89-bookworm AS builder
WORKDIR /workspace/rust-gateway
COPY rust-gateway/Cargo.toml ./Cargo.toml
COPY rust-gateway/Cargo.lock ./Cargo.lock
COPY rust-gateway/src ./src
COPY rust-gateway/resources ./resources
ENV CARGO_BUILD_JOBS=1
RUN cargo build --release -j 1

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates default-mysql-client tzdata wget \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /workspace/rust-gateway/target/release/notxboard-gateway /usr/local/bin/notxboard-gateway
COPY rust-gateway/resources /app/runtime/resources
COPY rust-gateway/resources/public /app/runtime/public
RUN mkdir -p /app/state/theme /app/state/plugins
ENV RUST_LOG=info
ENV RUST_RUNTIME_ROOT=/app/runtime
ENV RUST_STATE_ROOT=/app/state
EXPOSE 8000
ENTRYPOINT ["/usr/local/bin/notxboard-gateway"]
