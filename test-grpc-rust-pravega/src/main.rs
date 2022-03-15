use futures::stream::FuturesUnordered;
use futures::stream::StreamExt;
use std::env;
use std::time::Instant;

use pravega_client::client_factory::ClientFactory;
use pravega_client_config::ClientConfigBuilder;
use pravega_client_shared::Scope;


fn main() {
    let args: Vec<String> = env::args().collect();
    let host = if args.len() > 1 {
        args[1].clone()
    } else {
        "127.0.0.1:9090".to_string()
    };
    let client_config = ClientConfigBuilder::default()
        .controller_uri(host)
        .build()
        .expect("creating config");
    let client_factory = ClientFactory::new(client_config);
    let runtime = client_factory.runtime();
    let scope = Scope::from("hello5".to_string());
    let controller_client = client_factory.controller_client();

    // test multiple requests without pooling
    let mut futures = FuturesUnordered::new();
    for _ in 0..10000_i32 {
        let c = &controller_client;
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
