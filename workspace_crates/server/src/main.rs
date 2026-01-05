use db_manager;
use env_reader::{Env, get_env};
use http_stuff::{http_request_parsing::parse_http_request, http_response::ok_html};
use server::thread_pool;

use std::{collections::HashMap, io::Write, net::{IpAddr, TcpListener}, sync::{Arc, Mutex}, time::{Duration, Instant}};

const INTERVAL_BETWEEN_REQUESTS_MILLIS: Duration = Duration::from_millis(5000);
type IntervalLimiter = Arc<Mutex<HashMap<IpAddr, Instant>>>;

fn is_allowed(ip: IpAddr, limiter: &IntervalLimiter) -> bool {
    let mut map = limiter.lock().unwrap();
    let now = Instant::now();

    match map.get(&ip) {
        Some(&last) if now.duration_since(last) < INTERVAL_BETWEEN_REQUESTS_MILLIS => {
            false
        }
        _ => {
            map.insert(ip, now);
            true
        }
    }
}

fn main() {
    let env = get_env();
    let Env {
        ip_and_port,
        database_str,
    } = env;
    let listener = TcpListener::bind(ip_and_port.clone()).expect(
        &format!("I expected to bind to address {ip_and_port}")
    );
    let db_pool = db_manager::get_pool(database_str).expect("I expected to get a db pool :)");
    
    println!("Server listening on http://{}", ip_and_port.clone());
    
    let thread_pool = thread_pool::ThreadPool::new(8);
    
    let limiter: IntervalLimiter = Arc::new(Mutex::new(HashMap::new()));

    for stream in listener.incoming() {

        let mut stream = match stream {
            Ok(s) => s,
            Err(e) => {
                dbg!("Got error on unwrapping stream. I wonder why? This never happened in the past so far.", e);
                continue;
            }
        };
        // limit requests from same IP 
        let ip = stream.peer_addr().unwrap().ip();
        if !is_allowed(ip, &limiter) {
            dbg!("Dropping request from IP", ip, "Because it sent it too soon after the last one (less than ms ago)", INTERVAL_BETWEEN_REQUESTS_MILLIS);
            let _ = stream.write_all(
            b"HTTP/1.1 429 Too Many Requests\r\n\r\nRate limit exceeded",
            );  
            //let _ = stream.shutdown(std::net::Shutdown::Both);
            //drop(stream);
            continue;
        }
         


        let db_pool = db_pool.clone();
        thread_pool.execute(move || {
            let request_data = match parse_http_request(&mut stream) {
                Ok(data) => data,
                Err(e) => {
                    match e {
                        _ => {
                            eprintln!("main error {:?}", e);
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
