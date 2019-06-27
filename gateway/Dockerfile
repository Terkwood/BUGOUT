FROM rust

RUN rustup default nightly

WORKDIR /var/gateway
COPY . .

RUN cargo install --path .

CMD ["gateway"]
