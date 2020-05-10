FROM rust

RUN rustup default stable  

WORKDIR /var/BUGOUT/botlink

COPY . /var/BUGOUT/botlink/.

RUN cargo install --path .

ENV RUST_LOG info

CMD ["botlink"]
