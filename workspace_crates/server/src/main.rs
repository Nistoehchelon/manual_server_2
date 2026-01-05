use db_manager;
use env_reader::{Env, get_env};
use http_stuff::{http_request_parsing::parse_http_request, http_response::ok_html};
use server::{thread_pool, token_buckets_for_requests::{GlobalBucket, IpBuckets, TokenBucket}};

use std::{collections::HashMap, io::Write, net::TcpListener, sync::{Arc, Mutex}};



fn main() {
    let env = get_env();
    let Env {
        ip_and_port,
        database_str,
        server_global_requests_token_bucket_capacity,
        server_global_requests_token_bucket_refill_rate_per_millis,
        per_ip_token_bucket_capacity,
        per_ip_token_bucket_refill_rate_per_millis
    } = env;
    let listener = TcpListener::bind(ip_and_port.clone()).expect(
        &format!("I expected to bind to address {ip_and_port}")
    );
    let db_pool = db_manager::get_pool(database_str).expect("I expected to get a db pool :)");
    
    println!("Server listening on http://{}", ip_and_port.clone());
    
    let thread_pool = thread_pool::ThreadPool::new(8);

    let global_bucket: GlobalBucket = Arc::new(Mutex::new(TokenBucket::new(server_global_requests_token_bucket_capacity, server_global_requests_token_bucket_refill_rate_per_millis)));

    let ip_buckets: IpBuckets = Arc::new(Mutex::new(HashMap::new()));
    
    for stream in listener.incoming() {

        let mut stream = match stream {
            Ok(s) => s,
            Err(e) => {
                dbg!("Got error on unwrapping stream. I wonder why? This never happened in the past so far.", e);
                continue;
            }
        };

        let global_bucket = global_bucket.clone();
        let ip_buckets = ip_buckets.clone();

        let db_pool = db_pool.clone();
        thread_pool.execute(move || {
            let peer_ip = match stream.peer_addr() {
                Ok(a) => a.ip(), 
                Err(e) => {
                    dbg!("Could not get ip address :( Dropping stream, I guess? Wonder what happens if that fails, too?..", e);
                    let _ = stream.shutdown(std::net::Shutdown::Both);
                    drop(stream);
                    return;
                }
            };
            dbg!("Peer with IP ", peer_ip, "just issued a request");
            // Check global bucket
            if !global_bucket.lock().unwrap().try_to_consume_one_token() {
                dbg!("Server bucket empty");
            let _ = stream.write_all(&ok_html("Hello".to_string()));
            return;
            }
            // ip bucket
            {
            let mut ip_buckets = ip_buckets.lock().unwrap();
            let ip_bucket = ip_buckets.entry(peer_ip.clone())
                .or_insert_with(|| TokenBucket::new(per_ip_token_bucket_capacity, per_ip_token_bucket_refill_rate_per_millis));
            
            if !ip_bucket.try_to_consume_one_token() {
                dbg!(&format!("Bucket for IP {peer_ip} is empty"));
                let _ = stream.write_all(&ok_html("Hello".to_string()));
                let _ = stream.shutdown(std::net::Shutdown::Both);
                return;
            }
        }

            let request_data = match parse_http_request(&mut stream) {
                Ok(data) => data,
                Err(e) => {
                    match e {
                        _ => {
                            eprintln!("main error, could not parse request {:?}", e);
                            let _ = stream.write_all(&ok_html(format!("Hmm")));
                        }
                    };
                    let _ = stream.shutdown(std::net::Shutdown::Both);
                    drop(stream);
                    return;
                }
            };

            let response = routing::generate_response(request_data, &db_pool);
            stream.write_all(&response).ok();
            let _ = stream.shutdown(std::net::Shutdown::Both);
            drop(stream);
        });
    }

    println!("Shutting down server.");
}
