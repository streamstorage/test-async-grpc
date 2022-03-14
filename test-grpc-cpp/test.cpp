#include <iostream>
#include <memory>
#include <thread>

#include <agrpc/asioGrpc.hpp>
#include <boost/asio.hpp>
#include <grpcpp/grpcpp.h>

#include "controller.grpc.pb.h"

template<class T>
using co_async = boost::asio::awaitable<T>;
using boost::asio::use_awaitable;
using std::string;
using std::cout;
using std::endl;

class Context {
 public:
  Context()
    : grpc_ctx_(std::move(std::make_unique<grpc::CompletionQueue>())),
      grcp_guard_(grpc_ctx_.get_executor()),
      grpc_runner_([this] { grpc_ctx_.run(); }),
      io_guard_(io_ctx_.get_executor()),
      io_runner_([this] { io_ctx_.run(); }) {}

  ~Context() {
    grcp_guard_.reset();
    grpc_runner_.join();
    io_guard_.reset();
    io_runner_.join();
  }

  grpc::CompletionQueue* get_completion_queue() {
    return grpc_ctx_.get_completion_queue();
  }

  template<class Response>
  inline auto finish(grpc::ClientAsyncResponseReader<Response>& reader,
                    Response& response,
                    grpc::Status& status) {
    return agrpc::finish(reader, response, status, 
                         boost::asio::bind_executor(grpc_ctx_, use_awaitable));
  }

  template<class T>
  std::future<T> spawn(co_async<T> async_function) {
    return boost::asio::co_spawn(
        io_ctx_,
        std::move(async_function),
        boost::asio::use_future);
  }

 private:
  using GrpcExecutorType = agrpc::GrpcContext::executor_type;
  using IoExecutorType = boost::asio::io_context::executor_type;
  using GrpcGuard = boost::asio::executor_work_guard<GrpcExecutorType>;
  using IoGuard = boost::asio::executor_work_guard<IoExecutorType>;

  agrpc::GrpcContext grpc_ctx_;
  boost::asio::io_context io_ctx_;
  GrpcGuard grcp_guard_;
  IoGuard io_guard_;
  std::thread grpc_runner_, io_runner_;
};

using grpc::Channel;
using grpc::ClientContext;
using grpc::Status;
namespace pravega_grpc = io::pravega::controller::stream::api::grpc::v1;

class RpcClient {
 public:
  RpcClient(const string& host, std::shared_ptr<Context> context)
      : context_(context) {
    const auto channel = grpc::CreateChannel(host, grpc::InsecureChannelCredentials());
    stub_ = pravega_grpc::ControllerService::NewStub(channel);
  }

  ~RpcClient() {
    google::protobuf::ShutdownProtobufLibrary();
  }

  co_async<bool> create_scope_async(const string& scope) const {
    ClientContext client_context;
    Status status;

    pravega_grpc::ScopeInfo request; 
    request.set_scope(scope);
    pravega_grpc::CreateScopeStatus reply;

    try {
      auto reader = this->stub_->AsynccreateScope(&client_context, request, 
                                                  context_->get_completion_queue());
      co_await context_->finish(*reader, reply, status);
      if (!status.ok()) {
        throw std::runtime_error("RPC failed");
      }
      switch (reply.status()) {
        case pravega_grpc::CreateScopeStatus::SUCCESS:
          co_return true;
        case pravega_grpc::CreateScopeStatus::SCOPE_EXISTS:
          co_return false;
        case pravega_grpc::CreateScopeStatus::FAILURE:
          throw std::runtime_error("Operation failed");
        case pravega_grpc::CreateScopeStatus::INVALID_SCOPE_NAME:
          throw std::runtime_error("Invalid scope");
      }
    } catch (std::exception &e) {
      throw std::runtime_error(e.what());
    }
  }

 private:
  std::unique_ptr<pravega_grpc::ControllerService::Stub> stub_;
  std::shared_ptr<Context> context_;
};

co_async<void> test_grpc_call(std::shared_ptr<RpcClient> client, const string& scope, int no) {
  bool reply = co_await client->create_scope_async(scope);
  cout << "call " << no << ": " << reply << endl;
}

int main() {
  auto context = std::make_shared<Context>();
  auto client = std::make_shared<RpcClient>("127.0.0.1:9090", context);

  int test_grpc_call_num = 10000;
  std::future<void> futures[test_grpc_call_num];
  
  const auto start = std::chrono::steady_clock::now();
  for (int i = 0; i < test_grpc_call_num; i++) {
    futures[i] = context->spawn(test_grpc_call(client, "hello5", i));
  }
  for (int j = 0; j < test_grpc_call_num; j++) {
    futures[j].get();
  }
  const auto end = std::chrono::steady_clock::now();
  const auto milliseconds = std::chrono::duration_cast<std::chrono::milliseconds>(end - start).count();
  cout << "in total: " << double(milliseconds) << "ms requests/s: " << test_grpc_call_num / double(milliseconds) * 1000.0 << endl;
}