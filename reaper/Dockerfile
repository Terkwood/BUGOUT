FROM rust

WORKDIR /var/
# git clone to support the submodule
RUN git clone https://github.com/Terkwood/BUGOUT
WORKDIR /var/BUGOUT/reaper
RUN git submodule update --init

RUN apt-get update -y 
RUN apt-get install -y clang

RUN curl -O https://raw.githubusercontent.com/vishnubob/wait-for-it/master/wait-for-it.sh

RUN chmod 755 wait-for-it.sh
RUN git fetch && git pull
RUN git rev-parse HEAD

RUN rustup default stable   

# make sure we have the most recent of everything,
# not just what's on the git branch
COPY . /var/BUGOUT/reaper/.

RUN cargo install --path .

CMD ["sh", "run.sh"]
