FROM rust

WORKDIR /var/BUGOUT/botlink

RUN rustup default stable  

COPY . /var/BUGOUT/botlink/.

#TODO
#RUN cargo install --path .

ENV RUST_LOG info

# TODO
#CMD ["botlink"]
CMD ["tail","-f","/dev/null"]
