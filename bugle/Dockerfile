FROM rust

WORKDIR /var/BUGOUT/bugle

RUN curl -O https://raw.githubusercontent.com/vishnubob/wait-for-it/master/wait-for-it.sh

RUN chmod 755 wait-for-it.sh

RUN rustup default nightly

RUN COPY . /var/BUGOUT/bugle/.

RUN cargo install --path .

CMD ["bugle"]
