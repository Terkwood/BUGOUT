FROM rust

RUN rustup default nightly

WORKDIR /var/
RUN git clone https://github.com/Terkwood/BUGOUT
WORKDIR /var/BUGOUT
RUN git checkout feature/gateway
WORKDIR /var/BUGOUT/gateway
RUN git submodule update --init

RUN cargo install --path .

CMD ["gateway"]
