use std::collections::HashMap;
use std::io::Cursor;
use std::{env, str};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use ipfs_api_backend_hyper::{IpfsApi, IpfsClient};
use tokio::time;
use futures::{future, select, FutureExt, StreamExt, TryStreamExt};
use tokio_stream::wrappers::IntervalStream;
use uuid::{Bytes, Uuid};

static TOPIC: &str = "test";

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let number_of_messages = args.get(1).expect("Pass number of messages").parse::<usize>().unwrap();

    let sent_messages: Arc<Mutex<HashMap<Uuid, SystemTime>>> = Arc::new(Mutex::new(HashMap::new()));
    let received_messages: Arc<Mutex<HashMap<Uuid, SystemTime>>> = Arc::new(Mutex::new(HashMap::new()));

    let publish_client  = IpfsClient::default();

    let interval = time::interval(Duration::from_secs_f32(0.5));
    let publish_sent_messages = sent_messages.clone();
    let mut publish = IntervalStream::new(interval)
        .then(|_| future::ok(())) // Coerce the stream into a TryStream
        .try_for_each(|_| {
            println!(".");

            let uuid = Uuid::new_v4();

            let mut sent_messages = publish_sent_messages.lock().unwrap();
            sent_messages.insert(uuid, SystemTime::now());

            publish_client
                .pubsub_pub(TOPIC, Cursor::new(uuid))
                .boxed_local()
        })
        .boxed_local()
        .fuse();

    let subscript_received_messages = received_messages.clone();
    let mut subscribe = {
        let client = IpfsClient::default();

        client
            .pubsub_sub(TOPIC)
            .take(number_of_messages)
            .try_for_each(|msg| {
                let received = Uuid::from_bytes(Bytes::try_from(msg.data).unwrap());

                let mut received_messages = subscript_received_messages.lock().unwrap();
                received_messages.insert(received, SystemTime::now());

                future::ok(())
            })
            .fuse()
    };

    select! {
        res = subscribe => match res {
            Ok(_) => calc_results(number_of_messages, sent_messages.lock().unwrap().clone(), received_messages.lock().unwrap().clone()),
            Err(e) => eprintln!("error reading messages: {}", e)
        },
        res = publish => if let Err(e) = res {
            eprintln!("error publishing messages: {}", e);
        },
    }
}

fn calc_results(number_of_messages: usize, sent: HashMap<Uuid, SystemTime>, received: HashMap<Uuid, SystemTime>) {
    let mut results = vec![];

    received.iter().for_each(|(uuid, time)| {
        let sent = sent.get(uuid);
        let Some(sent_time) = sent else {return};

        let delay = time.duration_since(sent_time.clone()).unwrap();
        println!("{:?}", delay);
        results.push(delay);
    });

    let sum_time = results.iter().fold(Duration::ZERO, |acc, &d| acc + d);
    let average = sum_time / results.len() as u32;

    println!();
    println!("Average delay in {} messages is: {:?}", number_of_messages, average);
}