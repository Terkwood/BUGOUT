FROM rust

WORKDIR /var/BUGOUT/micro-judge

RUN rustup default stable  

COPY . /var/BUGOUT/micro-judge/.

RUN cargo install --path .

ENV RUST_LOG info

CMD ["micro_judge"]
