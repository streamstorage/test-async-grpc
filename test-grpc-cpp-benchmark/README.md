```
mdkir build
cd build
cmake -DCMAKE_PREFIX_PATH=/usr/local/share/grpc-1.43.2 -DBOOST_ROOT=/usr/local/share/boost_1_78_0 ..
make
./test-benchmark
```