use db_manager::{self, DbPool};
use config_reader::{Config, read_config};
use http_stuff::{http_request_parsing::parse_http_request, http_response::ok_html};
use server::{thread_pool, token_buckets_for_requests::{GlobalBucket, IpBuckets, TokenBucket}};



use std::{collections::HashMap, io::Write, net::{IpAddr, SocketAddr}, sync::{Arc, Mutex}};


use tokio::{io::AsyncWriteExt, net::{TcpListener, TcpStream}};


#[tokio::main(flavor = "multi_thread", worker_threads = 8)]
async fn main() -> std::io::Result<()>{
    let env = read_config();
    let Config {
        ip_and_port,
        database_str,
        server_global_requests_token_bucket_capacity,
        server_global_requests_token_bucket_refill_rate_per_millis,
        per_ip_token_bucket_capacity,
        per_ip_token_bucket_refill_rate_per_millis
    } = env;
    let listener = TcpListener::bind(ip_and_port.clone()).await.expect(
        &format!("I expected to bind to address {ip_and_port}")
    );
    let db_pool = db_manager::get_pool(database_str).expect("I expected to get a db pool :)");
    
    println!("Server listening on http://{}", ip_and_port.clone());
    

    let global_bucket: GlobalBucket = Arc::new(Mutex::new(TokenBucket::new(server_global_requests_token_bucket_capacity, server_global_requests_token_bucket_refill_rate_per_millis)));

    let ip_buckets: IpBuckets = Arc::new(Mutex::new(HashMap::new()));
    
    loop {
        let (stream, peer_ip) = match listener.accept().await {
            Ok((s, a)) => (s, a),
            Err(e) => {
                dbg!("Could not unwrap stream? :(");
                continue;
            }
        };

        let global_bucket = global_bucket.clone();
        let ip_buckets = ip_buckets.clone();

        let db_pool = db_pool.clone();
        tokio::spawn(async move {
            handle(stream, db_pool, peer_ip.ip(), global_bucket, ip_buckets, per_ip_token_bucket_capacity, per_ip_token_bucket_refill_rate_per_millis)
        });
    }

    println!("Shutting down server.");
    Ok(())
}

pub async fn handle(mut stream: TcpStream, db_pool: DbPool, peer_ip: IpAddr, global_bucket: GlobalBucket, ip_buckets: IpBuckets, per_ip_token_bucket_capacity: f64, per_ip_token_bucket_refill_rate_per_millis: f64,) -> std::io::Result<()> {
            
            dbg!("Peer with IP ", peer_ip, "just issued a request");
            // Check global bucket
            if !global_bucket.lock().unwrap().try_to_consume_one_token() {
                dbg!("Server bucket empty");
            let _ = stream.write_all(&ok_html("Hello".to_string()));
            return Ok(());
            }
            // ip bucket
            {
            let mut ip_buckets = ip_buckets.lock().unwrap();
            let ip_bucket = ip_buckets.entry(peer_ip.clone())
                .or_insert_with(|| TokenBucket::new(per_ip_token_bucket_capacity, per_ip_token_bucket_refill_rate_per_millis));
            
            if !ip_bucket.try_to_consume_one_token() {
                dbg!(&format!("Bucket for IP {peer_ip} is empty"));
                let _ = stream.write_all(&ok_html("Hello".to_string()));
                let _ = stream.shutdown().await;
                return Ok(());
            }
        }

            let request_data = match parse_http_request(&mut stream).await {
                Ok(data) => data,
                Err(e) => {
                    match e {
                        _ => {
                            eprintln!("main error, could not parse request {:?}", e);
                            let _ = stream.write_all(&ok_html(format!("Hmm")));
                        }
                    };
                    let _ = stream.shutdown().await;
                    drop(stream);
                    return Ok(());
                }
            };

            let response = routing::generate_response(request_data, &db_pool);
            stream.write_all(&response).await;
            let _ = stream.shutdown().await;
            drop(stream);

            Ok(())
}