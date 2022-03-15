## Prerequisite
1. Follow https://grpc.io/docs/languages/cpp/quickstart/ to install gRPC in C++
2. Download boost from https://www.boost.org/users/download/

## Build & Run
```
mdkir build
cd build
cmake -DCMAKE_PREFIX_PATH=/usr/local/share/grpc-1.43.2 -DBOOST_ROOT=/usr/local/share/boost_1_78_0 ..
make
./test or ./test 10.247.97.19:9090
```
