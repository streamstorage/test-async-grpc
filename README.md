# test-async-grpc
Test the async grpc call performance for different framework. Start grpc server by running standalone [pravega](https://github.com/pravega/pravega/blob/master/documentation/src/docs/deployment/run-local.md#from-installation-package).

# project
- test-grpc-cpp-benchmark

  C++ implementation from [example-vcpkg-grpc](https://github.com/Tradias/example-vcpkg-grpc/blob/asio-grpc-14/client.cpp) as benchmark.

- test-grpc-cpp

  C++ implementation using [asio-grpc](https://github.com/Tradias/asio-grpc) and std::future.

- test-grpc-rust-pravega

  Rust implementation with [pravega-client-rust](https://github.com/pravega/pravega-client-rust).

- test-grpc-rust-pravega

  Rust implementation with create scope logic only of [pravega-client-rust](https://github.com/pravega/pravega-client-rust) for comparison. 
  