use db_manager;
use env_reader::{Env, get_env};
use http_stuff::http_request_parsing::parse_request_data;
use server::thread_pool;


use std::{io::Write, net::TcpListener};

fn main() {
    let env = get_env();
    let Env {
        ip_and_port,
        database_str,
    } = env;

    let db_pool = db_manager::get_pool(database_str).expect("Could not connect to db");

    let listener = TcpListener::bind(ip_and_port.clone()).expect("Failed to bind to address");
    println!("Server listening on http://{}", ip_and_port.clone());
    let thread_pool = thread_pool::ThreadPool::new(8);
    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let request_data = match parse_request_data(&mut stream) {
            Some(data) => data,
            None => continue,
        };
        if routing::is_requesting_static_resource(request_data.path.clone()) {
            thread_pool.execute(move || {
                let response = routing::generate_static_response(request_data);
                stream.write_all(&response).ok();
            });
            continue;
        }

        let db_pool = db_pool.clone();

        thread_pool.execute(move || {
            let response = routing::generate_response(request_data, &db_pool);
            stream.write_all(&response).ok();
        });
    }

    println!("Shutting down server.");
}
