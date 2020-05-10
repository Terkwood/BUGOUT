FROM rust

RUN rustup default stable  

RUN apt-get update -y 
RUN apt-get install -y clang 

WORKDIR /var/

# supports the librdkafka submodule
RUN git clone https://github.com/Terkwood/BUGOUT
WORKDIR /var/BUGOUT/gateway
RUN git submodule update --init

RUN git fetch && git pull
RUN git rev-parse HEAD

COPY . /var/BUGOUT/gateway/.

RUN cargo install --path .

ENV RUST_LOG info

CMD ["gateway"]
