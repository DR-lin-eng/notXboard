FROM rust:1.89-bookworm AS builder
WORKDIR /workspace/rust-gateway
COPY Cargo.toml ./Cargo.toml
COPY Cargo.lock ./Cargo.lock
COPY src ./src
COPY resources ./resources
ENV CARGO_BUILD_JOBS=1
RUN cargo build --release -j 1

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates default-mysql-client tzdata wget \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /workspace/rust-gateway/target/release/notxboard-gateway /usr/local/bin/notxboard-gateway
COPY resources /app/runtime/resources
COPY resources/public /app/runtime/public
RUN rm -f /app/runtime/public/theme/Maintainable/dashboard.blade.php /app/runtime/public/theme/Maintainable/config.json \
    && mkdir -p /app/state/theme /app/state/plugins
ENV RUST_LOG=info
ENV RUST_RUNTIME_ROOT=/app/runtime
ENV RUST_STATE_ROOT=/app/state
EXPOSE 8000
ENTRYPOINT ["/usr/local/bin/notxboard-gateway"]
