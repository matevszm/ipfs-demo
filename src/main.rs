use std::io::Cursor;
use std::{str};
use std::time::{Duration, SystemTime};
use ipfs_api_backend_hyper::{IpfsApi, IpfsClient};
use futures::{FutureExt};

static TOPIC: &str = "test";
static MESSAGE: &str = "Example";

#[tokio::main]
async fn main() {
    let publish_client  = IpfsClient::default();

    let start_time = SystemTime::now();

    for _ in 0..10000 {
        publish_client
            .pubsub_pub(TOPIC, Cursor::new(MESSAGE))
            .boxed_local().await.unwrap_or(());
    }

    let duration = SystemTime::now().duration_since(start_time).unwrap();
    println!("Total time: {:?}", duration);

    let per_second = 10000f64 / duration.as_secs_f64();
    println!("Requests per second {}", per_second)
}