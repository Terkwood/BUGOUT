# syntax=docker/dockerfile:experimental
FROM rust as builder

RUN rustup default stable  

WORKDIR /app-src

COPY . /app-src/.

RUN cat Cargo.toml

RUN --mount=type=cache,target=/app-src/target \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/usr/local/cargo/registry \
    [ "cargo", "build", "--release" ]

RUN ls target
RUN ls target/release
RUN ls target/release/micro-game-lobby

FROM debian:stable-slim

WORKDIR /app

COPY --from=builder /app-src/target/release/micro-game-lobby /app/.

ENV RUST_LOG trace


CMD ["micro-game-lobby"]
