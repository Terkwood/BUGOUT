FROM rust

RUN rustup default nightly

WORKDIR /var/
RUN git clone https://github.com/Terkwood/BUGOUT
WORKDIR /var/BUGOUT/gateway
RUN git submodule update --init

RUN apt-get update -y 
RUN apt-get install -y clang

RUN cargo install --path .

CMD ["gateway"]
