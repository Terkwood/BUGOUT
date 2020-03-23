### THIS DOESN'T WORK
###     but you could probably salvage it
###        after moving it into the parent directory


FROM rust AS builder
WORKDIR /var/BUGOUT
COPY src src
COPY Cargo.toml Cargo.toml
RUN cargo install --path .


FROM arm64v8/ubuntu:18.04

# includes path to libc -- find it with gcc --print-file-name=libc.a
ENV LD_LIBRARY_PATH=/usr/lib/aarch64-linux-gnu/tegra:/usr/local/cuda-10.0/targets/aarch64-linux/lib:/usr/lib/gcc/aarch64-linux-gnu/7/../../../aarch64-linux-gnu

WORKDIR /var/BUGOUT

RUN apt-get update -y
RUN apt-get upgrade -y
RUN apt-get dist-upgrade -y

COPY --from=builder /usr/local/cargo/bin/tinybrain /var/BUGOUT/tinybrain
COPY katago /var/BUGOUT/.
COPY g170e-b20c256x2-s2430231552-d525879064.bin.gz /var/BUGOUT/.
COPY analysis.cfg /var/BUGOUT/.

RUN ldd --version

ENV RUST_LOG=info

CMD [ "./tinybrain" ]
