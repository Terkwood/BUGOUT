FROM rust

WORKDIR /var/gateway
COPY . .

RUN cargo install --path .

CMD ["gateway"]
