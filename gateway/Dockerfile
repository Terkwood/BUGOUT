# syntax=docker/dockerfile:experimental
FROM rust as builder

RUN rustup default stable  

WORKDIR /app-src

COPY Cargo.toml /app-src/.
COPY src/ /app-src/src

RUN --mount=type=cache,target=/app-src/target \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/usr/local/cargo/registry \
    [ "cargo", "build", "--release" ]

RUN --mount=type=cache,target=/app-src/target \
    ["cp", "/app-src/target/release/gateway", "/usr/local/bin/gateway"]

FROM debian:stable-slim

COPY --from=builder /usr/local/bin/gateway /usr/local/bin/gateway

WORKDIR /BUGOUT

ENV RUST_LOG info

CMD ["gateway"]
