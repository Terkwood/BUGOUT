FROM arm64v8/ubuntu:18.04


# includes path to libc -- find it with gcc --print-file-name=libc.a
ENV LD_LIBRARY_PATH=/usr/lib/aarch64-linux-gnu/tegra:/usr/local/cuda-10.0/targets/aarch64-linux/lib:/usr/lib/gcc/aarch64-linux-gnu/7/../../../aarch64-linux-gnu

WORKDIR /var/BUGOUT/tinybrain

RUN apt-get update -y
RUN apt-get upgrade -y
RUN apt-get dist-upgrade -y

COPY katago /var/BUGOUT/tinybrain/.
COPY g170e-b20c256x2-s2430231552-d525879064.bin.gz /var/BUGOUT/tinybrain/.
COPY analysis.cfg /var/BUGOUT/tinybrain/.

RUN ldd --version

CMD [ "./katago", "analysis", "-model" ,"g170e-b20c256x2-s2430231552-d525879064.bin.gz", "-config", "analysis.cfg", "-analysis-threads", "2" ]
