FROM rust

WORKDIR /var/BUGOUT/bugle

RUN rustup default stable  

COPY . /var/BUGOUT/bugle/.

RUN cargo install --path .

CMD ["bugle"]
