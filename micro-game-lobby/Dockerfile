# syntax=docker/dockerfile:experimental
FROM rust

RUN rustup default stable  

WORKDIR /var/BUGOUT/micro-game-lobby

COPY Cargo.toml /var/BUGOUT/micro-game-lobby/.
COPY src/ /var/BUGOUT/micro-game-lobby/src

RUN --mount=type=cache,target=/root/.cargo \
    [ "cargo", "install", "--path", "." ]

ENV RUST_LOG trace

CMD ["micro-game-lobby"]
