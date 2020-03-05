# Not a very useful example
FROM debian:stable-slim
RUN apt-get update && apt-get upgrade
RUN apt-get install -y git  libzip-dev libboost-filesystem-dev
RUN apt-get install -y opencl-headers ocl-icd-dev ocl-icd-opencl-dev
WORKDIR /var/BUGOUT
RUN git clone https://github.com/lightvector/KataGo
WORKDIR /var/BUGOUT/KataGo/cpp
RUN apt-get install -y cmake
RUN cmake . -DBUILD_MCTS=1 -DUSE_BACKEND=OPENCL -DUSE_TCMALLOC=0
CMD [ "./katago" ]
