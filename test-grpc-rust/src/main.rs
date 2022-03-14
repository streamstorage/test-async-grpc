use futures::stream::FuturesUnordered;
use futures::stream::StreamExt;
use std::time::{Instant};

use std::convert::Into;
use std::result::Result as StdResult;
use std::str::FromStr;
use tokio::runtime::Handle;
use tokio::sync::RwLock;
use tonic::transport::{Channel, Endpoint, Uri};

use controller::{
    controller_service_client::ControllerServiceClient, create_scope_status, ScopeInfo, CreateScopeStatus
};

#[allow(non_camel_case_types)]
pub mod controller {
    tonic::include_proto!("io.pravega.controller.stream.api.grpc.v1");
    // this is the rs file name generated after compiling the proto file, located inside the target folder.
}

async fn get_channel(host: &String) -> Channel {
    let s = format!("http://{}", host);
    let uri_result = Uri::from_str(&s).unwrap();

    let endpoints = (0..3)
        .map(|_a| Channel::builder(uri_result.clone()))
        .collect::<Vec<Endpoint>>();

    async { Channel::balance_list(endpoints.into_iter()) }.await
}

pub struct RpcClient {
    channel: RwLock<ControllerServiceClient<Channel>>,
}

impl RpcClient {
    pub fn new(handle: &Handle, host: &String) -> Self {
        // actual connection is established lazily.
        let ch = handle.block_on(get_channel(&host));
        let client = ControllerServiceClient::new(ch);
        RpcClient {
            channel: RwLock::new(client),
        }
    }

    async fn get_controller_client(&self) -> ControllerServiceClient<Channel> {
        self.channel.read().await.clone()
    }

    async fn create_scope(&self, scope: &String) -> Result<bool, String> {
        use create_scope_status::Status;
        let request = tonic::Request::new(ScopeInfo {
            scope: scope.clone(),
        });
    
        let op_status: StdResult<tonic::Response<CreateScopeStatus>, tonic::Status> = self
            .get_controller_client()
            .await
            .create_scope(request)
            .await;
        match op_status {
            Ok(code) => match code.into_inner().status() {
                Status::Success => Ok(true),
                Status::ScopeExists => Ok(false),
                Status::InvalidScopeName => Err("Invalid scope".into()),
                _ => Err("Operation failed".into()),
            },
            _ => Err("Grpc error".into()),
        }
    }
}

fn main() {
    let scope = "hello5".to_string();
    let host = "127.0.0.1:9090".to_string();
    let runtime = tokio::runtime::Runtime::new().expect("create runtime");
    let client = RpcClient::new(runtime.handle(), &host);

    // test multiple requests without pooling
    let mut futures = FuturesUnordered::new();
    for _ in 0..10000_i32 {
        let c = &client;
        let s = &scope;
        futures.push(async move { 
            c.create_scope(&s).await
        });
    }
    let start_total = Instant::now();
    runtime.block_on(async move {
        loop { 
            let start = Instant::now();
            if let Some(res) = futures.next().await {
                match res {
                    //Ok(resp) => println!("{:?}", resp),
                    Err(e) => {
                        println!("Errant response; err = {:?}", e);
                    },
                    _ => (),
                }
                let duration = start.elapsed();
                println!("call: {:?}", duration);
            } else {
                break;
            }
        }
    });

    let duration_total = start_total.elapsed();
    println!("in total: {:?} requests/s: {}", duration_total, 10000.0 / duration_total.as_millis() as f64 * 1000.0);
}
