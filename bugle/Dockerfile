FROM rust

WORKDIR /var/BUGOUT/bugle

RUN rustup default stable  

COPY . /var/BUGOUT/bugle/.

#TODO
#RUN cargo install --path .

#CMD ["bugle"]

CMD ["tail","-f","/dev/null"]
