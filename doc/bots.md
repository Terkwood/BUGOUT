# Notes on KataGo

You can follow the instructions for installing VC4CL on RPI, below... but we still need to figure out how/if we can build KataGo on a Raspberry Pi.

## Grabbing DEB packages

[Follow this guide... for the most part](https://github.com/doe300/VC4CL/wiki/How-to-get).

We found that we needed this artifact, specifically:

- https://circleci.com/api/v1.1/project/github/doe300/VC4C/1730/artifacts

```sh
curl "https://circleci.com/api/v1.1/project/github/doe300/VC4C/1730/artifacts" --output /tmp/dump
```

After you install all the DEB files, make sure to run ldconfig.

```sh
sudo ldconfig
```

## Building KataGo

### Installing cmake on raspbian stretch

You need a more recent version of CMake than what's in raspbian stretch.  Add `stretch-backports` in your sources.list: https://backports.debian.org/Instructions/

```text
deb https://deb.debian.org/debian buster-backports main
```

[You need GPG keys for stretch-backports](https://rolfje.wordpress.com/2017/06/09/installing-gpg-keys-for-debian-backports/)

```sh
sudo su -
gpg --keyserver pgp.mit.edu --recv-keys 7638D0442B90D010 
gpg --armor --export 7638D0442B90D010 | apt-key add -
exit
sudo apt-get update
sudo apt-get -t stretch-backports install cmake
```

Then try installing [this version of cmake](https://packages.debian.org/stretch-backports/cmake).

### Building KataGo

Check out some options in `KataGo/cpp/CMakeFiles/3.13.2/CMakeCCompiler.cmake `

WE HACKED UP `CMakeLists.txt` PRETTY DARNED GOOD!  CHECK IT OUT.

The following chunk will enable pthreads for g++.

```text
  # On g++ it seems like we need to explicitly link threads as well.
  # It seems sometimes this is implied by other options automatically like when 
we enable CUDA, but we get link errors
  # if we don't explicitly require threads it when attempting to build without C
UDA
  # HACK
  #if(CMAKE_COMPILER_IS_GNUCC AND (NOT USE_BACKEND STREQUAL "CUDA"))
    find_package (Threads REQUIRED)
    target_link_libraries(katago Threads::Threads)
    #HACK endif()
```

...and later, to fulfill C++14 reqs...

```text

if(CMAKE_COMPILER_IS_GNUCC)
  if(NOT (${CMAKE_SYSTEM_PROCESSOR} MATCHES "arm"))
    set(CMAKE_CXX_FLAGS  "${CMAKE_CXX_FLAGS}   -mfpmath=sse")
  endif()

  if(USE_TCMALLOC)
    ### Hack flags rejected by gcc 6.3
    set(CMAKE_CXX_FLAGS  "${CMAKE_CXX_FLAGS} -g -O2 -pedantic -Wall -Wextra -Wno-sign-compare -Wcast-align -Wcast-qual -Wctor-dtor-privacy -Wdisabled-optimization -Wformat=2 -Wlogical-op -Wmissing-declarations -Wmissing-include-dirs -Wnoexcept -Woverloaded-virtual -Wredundant-decls -Wshadow -Wstrict-null-sentinel -Wstrict-overflow=1 -Wswitch-default -Wfloat-conversion -Wnull-dereference -Wunused -Wdiv-by-zero -Wduplicated-cond -Wduplicated-cond -mrestrict-it -fno-builtin-malloc -fno-builtin-calloc -fno-builtin-realloc -fno-builtin-free")
    ### HACK to include linking to pthreads
    set(CMAKE_EXE_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS} -lpthread -fno-builtin-malloc -fno-builtin-calloc -fno-builtin-realloc -fno-builtin-free")
  else()
    ### Hack flags rejected by gcc 6.3
    set(CMAKE_CXX_FLAGS  "${CMAKE_CXX_FLAGS}  -g -O2 -pedantic -Wall -Wextra -Wno-sign-compare -Wcast-align -Wcast-qual -Wctor-dtor-privacy -Wdisabled-optimization -Wformat=2 -Wlogical-op -Wmissing-declarations -Wmissing-include-dirs -Wnoexcept -Woverloaded-virtual -Wredundant-decls -Wshadow -Wstrict-null-sentinel -Wstrict-overflow=1 -Wswitch-default -Wfloat-conversion -Wnull-dereference -Wunused -Wdiv-by-zero -Wduplicated-cond -Wduplicated-cond -mrestrict-it")
    ### HACK to include linking to pthread
    set(CMAKE_EXE_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS} -lpthread")
  endif()
endif()
```

```sh
sudo apt install libzip-dev libboost-filesystem-dev
git clone git@github.com:lightvector/KataGo.git
cd KataGo/cpp

# use g++
export CMAKE_CXX_FLAGS=-std=gnu++11 
rm -rf CMakeFiles
cmake . -DUSE_BACKEND=OPENCL  -DOpenCL_LIBRARY=/usr/lib/arm-linux-gnueabihf/libOpenCL.so -DZLIB_LIBRARY=/usr/lib/arm-linux-gnueabihf/libz.so -DBOOST_LIBRARYDIR=/usr/lib/arm-linux-gnueabihf/ -DCMAKE_C_COMPILER=/usr/bin/gcc -DCMAKE_CXX_COMPILER=/usr/bin/g++ -DCMAKE_C_COMPILER_ID=gnu -DCMAKE_CXX_COMPILER_ID=gnu -DUSE_TCMALLOC=0
make
```

## Open question: LLVM config

There is some text in the KataGo README about LLVM config settings, are those helpful for us?


## Failed attempt: Building VC4C and VC4CL on Raspberry Pi 3 B+ 

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
