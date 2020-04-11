FROM rust

WORKDIR /var/BUGOUT/micro-judge

RUN rustup default stable  

COPY . /var/BUGOUT/micro-judge/.

#TODO
#RUN cargo install --path .

ENV RUST_LOG info

# TODO
#CMD ["micro_judge"]

CMD [ "tail", "-f", "/dev/null" ]
