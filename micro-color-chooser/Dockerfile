# syntax=docker/dockerfile:experimental
FROM rust as builder

RUN rustup default stable  

WORKDIR /app-src

COPY . /app-src/.

RUN --mount=type=cache,target=/app-src/target \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/usr/local/cargo/registry \
    [ "cargo", "build", "--release" ]

RUN --mount=type=cache,target=/app-src/target \
    ["cp", "/app-src/target/release/micro-color-chooser", "/usr/local/bin/micro-color-chooser"]

FROM debian:stable-slim

COPY --from=builder /usr/local/bin/micro-color-chooser /usr/local/bin/micro-color-chooser

ENV RUST_LOG info

CMD ["micro-color-chooser"]
