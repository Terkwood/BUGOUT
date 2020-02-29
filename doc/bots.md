# Notes on KataGo

## Installing VC4C and VC4CL on Raspberry Pi 3 B+ 

The following script is adapted from [this found gist](https://gist.github.com/senshu/671ecb1e68729c5e1a897c62251e00cf):

```sh
sudo apt install git cmake clang-3.9 opencl-headers ocl-icd-dev ocl-icd-opencl-dev
sudo update-alternatives --install /usr/bin/clang clang /usr/bin/clang-3.9 100


git clone https://github.com/doe300/VC4C.git
git clone https://github.com/doe300/VC4CL.git
git clone https://github.com/doe300/VC4CLStdLib.git  # ??

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
