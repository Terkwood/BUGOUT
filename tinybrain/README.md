# Notes on KataGo

We successfully built and ran KataGo on an [NVIDIA Jetson Nano developer board](https://developer.nvidia.com/embedded/jetson-nano-developer-kit). Here's how.

## Running

```sh
cargo install --path . 
target/release/tinybrain
```

## Prereq: Set up the Jetson Nano

[You can follow the NVIDIA guide](https://developer.nvidia.com/embedded/learn/get-started-jetson-nano-devkit).

## Prereq: Use python3 pip to install cmake

You need a recent version of cmake in order to build KataGo on the Jetson Nano.  You can obtain this using pip3.

This step will take _literally forever_.  Don't worry, it will eventually complete.

```sh
sudo apt install python3-pip libssl-dev
pip3 install scikit-build
pip3 install cmake
```

...years later...

```text
Successfully built cmake
Installing collected packages: cmake
Successfully installed cmake-3.16.3
```

Add to your .bashrc:

```sh
export PATH=$HOME/.local/lib/python3.6/site-packages/cmake/data/bin:$PATH

```

- [see this post for a hint](https://askubuntu.com/questions/952429/is-there-a-good-ppa-for-cmake-backports)      
- [and also see this github reference](https://github.com/clab/dynet/issues/1457)


## Prereq: Hack One Little Entry In CMakeLists.txt

In order to build KataGo, you must disable `mfpmath=sse` in `CMakeLists.txt `...

```text
if(CMAKE_COMPILER_IS_GNUCC)
        # HACK
        #if(NOT (${CMAKE_SYSTEM_PROCESSOR} MATCHES "arm"))
        #set(CMAKE_CXX_FLAGS  "${CMAKE_CXX_FLAGS} -mfpmath=sse")
        #endif()
```

## Build steps

[Follow the instructions for linux build](https://github.com/lightvector/KataGo)

```sh
 sudo apt install zlib1g-dev libzip-dev libboost-filesystem-dev
 sudo apt install libgoogle-perftools-dev  # for TCMALLOC
 git clone https://github.com/lightvector/KataGo.git
 cd KataGo/cpp
 export CUDACXX=/usr/local/cuda-10.0/bin/nvcc 
 rm -rf CMakeFiles/
 cmake . -DBUILD_MCTS=1 -DUSE_BACKEND=CUDA -DUSE_TCMALLOC=1 
```

You should see:

```text
...
[ 45%] Building CUDA object CMakeFiles/katago.dir/neuralnet/cudahelpers.cu.o
...
```

### Hint: Make sure you know where CUDA is

https://devtalk.nvidia.com/default/topic/1050692/jetson-nano/how-to-do-cuda-programming-on-jetson-nano-/

`/usr/local/cuda-10.0/bin/nvcc`

```sh
export CUDACXX=/usr/local/cuda-10.0/bin/nvcc
```

 

## Run the benchmark, create config file, run analysis

### Benchmark

As affordable as this system is, the GPU isn't exactly going to break any records for speed.  You need to make fewer visits in order to complete the benchmark in a reasonable amount of time.

```sh
./katago benchmark -model /path/to/g170e-b20c256x2-s2430231552-d525879064.bin.gz -config /path/to/analysis_example.cfg  -visits 80
```

### Config file


FP16.  Use it.  It's available on the NVIDIA Jetson Nano.

In your katago cfg file, you need to set this value:

```text
cudaUseFP16 = true
```

### Analysis

```sh
./katago analysis -model /path/to/g170e-b20c256x2-s2430231552-d525879064.bin.gz -config /path/to/analysis.cfg -analysis-threads 2

```



## Notes from KataGo docs

> per compute time, the 20-block extended-training "s2.43G" is likely the strongest net

## Enabling Edimax EW-7811Un Wifi adapter on the Jetson Nano

This probably isn't specific to our dev board, but we had to do some extra legwork to enable the inexpensive EW-7811Un wifi adapter

See:
- https://github.com/pvaret/rtl8192cu-fixes
- https://askubuntu.com/a/989723


Once you follow all of those steps, including managing power config settings, log in to Gnome and make sure the wifi adapter is "available to all users" via the networking GUI.  Then you'll see the system automatically log in at boot-up.
