FROM rust

RUN rustup default stable  

WORKDIR /var/BUGOUT/bugle

COPY . /var/BUGOUT/bugle/.

RUN cargo install --path .

CMD ["bugle"]
