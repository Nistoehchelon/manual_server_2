use config_reader::{Config, config_reader::Environment};
use http_stuff::{
    http_request_parsing::{HttpRequestParsingError, parse_http_request},
    http_response::{StatusCode, standalone_response},
};

use server::{
    http_to_https_redirect::http_to_https_redirect,
    load_local_cert::load_local_tls_config,
    token_buckets_for_requests::{GlobalBucket, IpBuckets, TokenBucket},
};
use tokio_rustls::{LazyConfigAcceptor, TlsAcceptor};

use std::{collections::HashMap, net::IpAddr, path::PathBuf, sync::Arc, time::Duration};

use rustls_acme::{AcmeConfig, caches::DirCache, is_tls_alpn_challenge};
use tokio::{
    io::{AsyncRead, AsyncWrite, AsyncWriteExt, BufReader},
    sync::Mutex,
    time::timeout,
};
use tokio_stream::StreamExt;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> std::io::Result<()> {
    let server_config = config_reader::read_config(PathBuf::from(".env"));

    let mut state = AcmeConfig::new([server_config.domain_name.clone()])
        .contact([format!(
            "mailto:{}",
            server_config.lets_encrypt_contact_email.clone()
        )])
        .cache_option(Some(DirCache::new("./rustls_acme_cache")))
        .directory_lets_encrypt(server_config.lets_encrypt_production.clone())
        .state();

    let challenge_rustls_config = state.challenge_rustls_config();
    let default_rustls_config = state.default_rustls_config();

    if server_config.environment == Environment::Production {
        // Spawns... the loop that updates the certs from Let's Encrypt if needed and so on, I think?
        tokio::spawn(async move {
            loop {
                match state.next().await.unwrap() {
                    Ok(ok) => println!("event: {:#?}", ok),
                    Err(err) => println!("error: {:#?}", err),
                }
            }
        });
    }

    let listener = tokio::net::TcpListener::bind(server_config.ip_and_port.clone())
        .await
        .expect(&format!(
            "I expected to bind to address {}",
            server_config.ip_and_port.clone()
        ));
    println!(
        "Server listening on https://{}",
        server_config.ip_and_port.clone()
    );

    // for rate limiting
    let global_bucket: GlobalBucket = Arc::new(Mutex::new(TokenBucket::new(
        server_config.server_global_requests_token_bucket_capacity,
        server_config.server_global_requests_token_bucket_refill_rate_per_millis,
    )));
    let ip_buckets: IpBuckets = Arc::new(Mutex::new(HashMap::new()));

    let db_pool = sqlx::PgPool::connect(&server_config.database_str)
        .await
        .expect("I expected pool construction to succeed :)");
    let common_stream_timeout_duration =
        Duration::from_millis(server_config.common_stream_timeout_millis.clone());

    // redirect from http  to https

    let server_config_for_task = server_config.clone();
    tokio::spawn(async move {
        match http_to_https_redirect(
            server_config_for_task.domain_name,
            server_config_for_task.ip,
            server_config_for_task.port,
            Duration::from_millis(server_config_for_task.common_stream_timeout_millis),
        )
        .await
        {
            Ok(_) => {}
            Err(e) => {
                dbg!(e);
            }
        };
    });

    loop {
        let global_bucket = global_bucket.clone();
        let ip_buckets = ip_buckets.clone();
        let db_pool = db_pool.clone();
        let server_config = server_config.clone();
        let (tcp, peer_ip) =
            match timeout(common_stream_timeout_duration.clone(), listener.accept()).await {
                Ok(Ok((s, a))) => (s, a),
                Ok(Err(e)) => {
                    dbg!("Could not unwrap stream? :( ", e);
                    continue;
                }
                Err(_e) => {
                    continue;
                }
            };

        match server_config.environment {
            Environment::Local => {
                let local_tls_config = load_local_tls_config();
                let local_tls_acceptor = TlsAcceptor::from(local_tls_config);

                tokio::spawn(async move {
                    let stream = match local_tls_acceptor.accept(tcp).await {
                        Ok(s) => s,
                        Err(e) => {
                            dbg!("Local TLS fail", e);
                            return;
                        }
                    };
                    let mut reader = BufReader::new(stream);

                    handle_stream(
                        &mut reader,
                        db_pool,
                        peer_ip.ip(),
                        global_bucket,
                        ip_buckets,
                        server_config.clone(),
                    )
                    .await;
                });
            }
            Environment::Production => {
                let challenge_rustls_config = challenge_rustls_config.clone();
                let default_rustls_config = default_rustls_config.clone();

                tokio::spawn(async move {
                    let start_handshake = LazyConfigAcceptor::new(Default::default(), tcp)
                        .await
                        .unwrap();

                    if is_tls_alpn_challenge(&start_handshake.client_hello()) {
                        println!(
                            "Yay, received a TLS-ALPN-01 validation request. Time to destroy the planet."
                        );
                        let mut tls = start_handshake
                            .into_stream(challenge_rustls_config)
                            .await
                            .unwrap();
                        tls.flush().await.unwrap();
                        tls.shutdown().await.unwrap();
                        return;
                    } else {
                        let stream = start_handshake
                            .into_stream(default_rustls_config)
                            .await
                            .unwrap();
                        let mut reader = BufReader::new(stream);
                        handle_stream(
                            &mut reader,
                            db_pool,
                            peer_ip.ip(),
                            global_bucket,
                            ip_buckets,
                            server_config.clone(),
                        )
                        .await;
                        return;
                    }
                });
            }
        }
    }
}

pub async fn handle_stream<S: AsyncRead + AsyncWrite + Unpin>(
    reader: &mut BufReader<S>,
    db_pool: sqlx::PgPool,
    peer_ip: IpAddr,
    global_bucket: GlobalBucket,
    ip_buckets: IpBuckets,
    server_config: Config,
) {
    let common_stream_timeout_duration =
        Duration::from_millis(server_config.common_stream_timeout_millis.clone());

    let _ = generate_and_write_response(
        reader,
        db_pool,
        peer_ip,
        global_bucket,
        ip_buckets,
        server_config.clone(),
    )
    .await;
    let f = timeout(common_stream_timeout_duration.clone(), reader.flush()).await;
    if f.is_err() {
        dbg!("main.rs flush timeout");
        return;
    }
    let s = timeout(common_stream_timeout_duration.clone(), reader.shutdown()).await;
    if s.is_err() {
        dbg!("main.rs stream shutdown timed out");
        return;
    }
}
pub async fn generate_and_write_response<S: AsyncRead + AsyncWrite + Unpin>(
    reader: &mut BufReader<S>,
    db_pool: sqlx::PgPool,
    peer_ip: IpAddr,
    global_bucket: GlobalBucket,
    ip_buckets: IpBuckets,
    server_config: Config,
) -> std::io::Result<()> {
    dbg!("Peer with IP ", peer_ip, "just issued a request");
    let Config {
        per_ip_token_bucket_capacity,
        per_ip_token_bucket_refill_rate_per_millis,
        common_stream_timeout_millis,
        max_header_name_len,
        max_header_value_len,
        ..
    } = server_config.clone();
    let common_stream_timeout_duration = Duration::from_millis(common_stream_timeout_millis);
    if let Err(error_response) = check_and_consume_buckets(
        &peer_ip,
        &global_bucket,
        &ip_buckets,
        per_ip_token_bucket_capacity,
        per_ip_token_bucket_refill_rate_per_millis,
    )
    .await
    {
        match timeout(
            common_stream_timeout_duration.clone(),
            reader.write_all(&error_response),
        )
        .await
        {
            Err(_e) => {
                dbg!("stream.write_all after bucket check time out");
            }
            _ => {}
        }
        return Ok(());
    }

    let request_data = match timeout(
        common_stream_timeout_duration.clone(),
        parse_http_request(
            reader,
            max_header_name_len + max_header_value_len,
            common_stream_timeout_duration.clone(),
        ),
    )
    .await
    {
        Ok(Ok(data)) => data,
        Ok(Err(e)) => {
            write_http_parsing_error(e, reader, common_stream_timeout_duration.clone()).await;
            return Ok(());
        }
        Err(_e) => {
            dbg!("main.rs parse_http_request timed out");
            return Ok(());
        }
    };

    let response = routing::generate_response(request_data, db_pool).await;
    if let Err(_e) = timeout(
        common_stream_timeout_duration.clone(),
        reader.write_all(&response),
    )
    .await
    {
        dbg!("main.rs response write timeout");
    }

    Ok(())
}

async fn write_http_parsing_error<S: AsyncRead + AsyncWrite + Unpin>(
    e: HttpRequestParsingError,
    stream: &mut BufReader<S>,
    common_stream_timeout_duration: Duration,
) {
    match e {
        HttpRequestParsingError::UriTooLong => {
            let err_response =
                standalone_response(StatusCode::UriTooLong414, format!("414 Uri Too Long ☺️"));
            let _ = timeout(
                common_stream_timeout_duration.clone(),
                stream.write_all(&err_response),
            )
            .await;
        }
        HttpRequestParsingError::HttpVersionNotSupported => {
            let err_response = standalone_response(
                StatusCode::HttpVersionNotSupported505,
                format!("505 HTTP Version Not Supported"),
            );
            let _ = timeout(
                common_stream_timeout_duration.clone(),
                stream.write_all(&err_response),
            )
            .await;
        }
        HttpRequestParsingError::TooManyHeaders | HttpRequestParsingError::TooManyCookies => {
            let err_response = standalone_response(
                StatusCode::RequestHeaderFieldsTooLarge431,
                format!("431 Header fields too large, and/or too many of them 😴"),
            );
            let _ = timeout(
                common_stream_timeout_duration.clone(),
                stream.write_all(&err_response),
            )
            .await;
        }
        _ => {
            eprintln!("main error, could not parse request {:?}", e);
            let err_response =
                standalone_response(StatusCode::BadRequest400, format!("400 Bad Request 🙀"));
            let _ = timeout(
                common_stream_timeout_duration.clone(),
                stream.write_all(&err_response),
            )
            .await;
        }
    };
}

async fn check_and_consume_buckets(
    peer_ip: &IpAddr,
    global_bucket: &GlobalBucket,
    ip_buckets: &IpBuckets,
    per_ip_token_bucket_capacity: f64,
    per_ip_token_bucket_refill_rate_per_millis: f64,
) -> Result<(), Vec<u8>> {
    // Check global bucket
    if !global_bucket.lock().await.try_to_consume_one_token() {
        let response = standalone_response(
            StatusCode::ServiceUnavailable503,
            "503 Service Unavailable 🫨".to_string(),
        );
        return Err(response);
    }

    // Check per-IP bucket
    let mut ip_buckets_guard = ip_buckets.lock().await;
    let ip_bucket = ip_buckets_guard.entry(*peer_ip).or_insert_with(|| {
        TokenBucket::new(
            per_ip_token_bucket_capacity,
            per_ip_token_bucket_refill_rate_per_millis,
        )
    });

    if !ip_bucket.try_to_consume_one_token() {
        let response = standalone_response(
            StatusCode::TooManyRequests429,
            "429 Too Many Requests 😈".to_string(),
        );
        return Err(response);
    }

    Ok(())
}
