use http_stuff::{http_request_parsing::parse_http_request, http_response::{StatusCode, standalone_response}};
use server::token_buckets_for_requests::{GlobalBucket, IpBuckets, TokenBucket};



use std::{collections::HashMap, net::IpAddr, sync::{Arc}};


use tokio::{sync::Mutex, io::AsyncWriteExt, net::{TcpListener, TcpStream}};


#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> std::io::Result<()>{
    let env = config_reader::read_config();
    let config_reader::Config {
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
    let db_pool = sqlx::PgPool::connect(&database_str).await.expect("Pool construction did not succeed");
    println!("Server listening on http://{}", ip_and_port.clone());
    

    let global_bucket: GlobalBucket = Arc::new(Mutex::new(TokenBucket::new(server_global_requests_token_bucket_capacity, server_global_requests_token_bucket_refill_rate_per_millis)));

    let ip_buckets: IpBuckets = Arc::new(Mutex::new(HashMap::new()));
    
    loop {
        let (stream, peer_ip) = match listener.accept().await {
            Ok((s, a)) => (s, a),
            Err(e) => {
                dbg!("Could not unwrap stream? :( ", e);
                continue;
            }
        };

        let global_bucket = global_bucket.clone();
        let ip_buckets = ip_buckets.clone();

        let db_pool = db_pool.clone();
        tokio::spawn(async move {
            let _ = handle(stream, &db_pool, peer_ip.ip(), global_bucket, ip_buckets, per_ip_token_bucket_capacity, per_ip_token_bucket_refill_rate_per_millis).await;
        });
    }

}

pub async fn handle(mut stream: TcpStream, db_pool: &sqlx::PgPool, peer_ip: IpAddr, global_bucket: GlobalBucket, ip_buckets: IpBuckets, per_ip_token_bucket_capacity: f64, per_ip_token_bucket_refill_rate_per_millis: f64,) -> std::io::Result<()> {
            
            dbg!("Peer with IP ", peer_ip, "just issued a request");
            // Check global bucket
            if !global_bucket.lock().await.try_to_consume_one_token() {
                dbg!("Server bucket empty");
            let _ = stream.write_all(&standalone_response(StatusCode::ServiceUnavailabe503, format!("503 Service Unavailable 🫨"))).await;
            return Ok(());
            }
            // ip bucket
            {   
            let mut ip_buckets = ip_buckets.lock().await;
            let ip_bucket = ip_buckets.entry(peer_ip.clone())
                .or_insert_with(|| TokenBucket::new(per_ip_token_bucket_capacity, per_ip_token_bucket_refill_rate_per_millis));
            
            if !ip_bucket.try_to_consume_one_token() {
                dbg!(&format!("Bucket for IP {peer_ip} is empty"));
                let error_response = standalone_response(StatusCode::TooManyRequests429,format!("429 Too Many Requests 😈"));
                let _ = stream.write_all(&error_response).await;
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
                            let err_response = standalone_response(StatusCode::BadRequest400, format!("400 Bad Request 🙀"));
                            let _ = stream.write_all(&err_response).await;
                        }
                    };
                    let _ = stream.shutdown().await;
                    drop(stream);
                    return Ok(());
                }
            };

            let response = routing::generate_response(request_data, &db_pool).await;
            let _ = stream.write_all(&response).await;
            let _ = stream.shutdown().await;
            drop(stream);

            Ok(())
}