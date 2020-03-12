FROM arm64v8/ubuntu:16.04

ENV LD_LIBRARY_PATH=/usr/lib/aarch64-linux-gnu/tegra

WORKDIR /var/BUGOUT/tinybrain

COPY katago /var/BUGOUT/tinybrain/.
COPY g170e-b20c256x2-s2430231552-d525879064.bin.gz /var/BUGOUT/tinybrain/.
COPY analysis.cfg /var/BUGOUT/tinybrain/.

CMD [ "./katago", "analysis", "-model" ,"g170e-b20c256x2-s2430231552-d525879064.bin.gz", "-config", "analysis.cfg", "-analysis-threads", "2" ]
