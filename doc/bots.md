# Notes on KataGo

## Grabbing DEB packages

[Follow this guide... for the most part](https://github.com/doe300/VC4CL/wiki/How-to-get).

We found that we needed this artifact, specifically:

- https://circleci.com/api/v1.1/project/github/doe300/VC4C/1730/artifacts

```sh
curl "https://circleci.com/api/v1.1/project/github/doe300/VC4C/1730/artifacts" --output /tmp/dump
```

## Building VC4C and VC4CL on Raspberry Pi 3 B+ 

[You can run OpenCL on a Raspberry Pi GPU](https://hackaday.com/2019/01/24/running-opencl-on-a-raspberry-pi-gpu/), which could potentially open a path for us to inexpensively compute moves using KataGo.

We need VC4CL to compile KataGo.

The following script is adapted from [this found gist](https://gist.github.com/senshu/671ecb1e68729c5e1a897c62251e00cf):

```sh
sudo apt install git cmake clang-3.9 opencl-headers ocl-icd-dev ocl-icd-opencl-dev
sudo apt install libhwloc-dev  libglew-dev libedit-dev zlib1g-dev
sudo update-alternatives --install /usr/bin/clang clang /usr/bin/clang-3.9 100


git clone https://github.com/doe300/VC4C.git
git clone https://github.com/doe300/VC4CL.git
git clone https://github.com/doe300/VC4CLStdLib.git  

cd VC4CLStdLib
cmake 
make
sudo make install
cd -

cd VC4C
cmake -DBUILD_TESTING=OFF -DSPIRV_FRONTEND=OFF 
make
sudo make install
cd -

cd VC4CL
cmake -DBUILD_TESTING=OFF 
make
sudo make install
cd -

mkdir VC4CL-test
cd VC4CL-test
wget https://raw.githubusercontent.com/doe300/VC4C/master/example/fibonacci.cl

export LD_LIBRARY_PATH=/usr/local/lib
VC4C --llvm --hex -o fibonacci.hex fibonacci.cl
```

## Native compilation environment in docker

We may be able to cross-compile using [this Docker image](https://hub.docker.com/r/nomaddo/cross-rpi/):

```sh
docker pull nomaddo/cross-rpi
```
