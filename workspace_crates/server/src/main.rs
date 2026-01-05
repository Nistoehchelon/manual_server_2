use db_manager;
use env_reader::{Env, get_env};
use http_stuff::{http_request_parsing::parse_http_request, http_response::ok_html};
use server::thread_pool;

use std::{io::Write, net::TcpListener};



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
    
    for stream in listener.incoming() {

        let mut stream = match stream {
            Ok(s) => s,
            Err(e) => {
                dbg!("Got error on unwrapping stream. I wonder why? This never happened in the past so far.", e);
                continue;
            }
        };

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
