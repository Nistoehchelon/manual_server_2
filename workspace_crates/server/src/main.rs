use config_reader::{Config, config_reader::Environment};
use http_stuff::{
    http_request_parsing::{
        HttpParserConfig, HttpRequestData, HttpRequestParsingError, parse_http_request,
    },
    http_response::{HttpResponseData, KeepAliveOrClose, StatusCode, standalone_response},
};

use routing::router::response_to_bytes;

use server::{
    buckets_for_requests::{
        GlobalBuckets, IpBuckets, TimeBucket, TokenAndTimeBuckets, TokenAndTimeBucketsAndLastSeen,
        TokenBucket,
    },
    check_and_consume_buckets::check_and_consume_buckets,
    http_to_https_redirect::http_to_https_redirect,
    load_local_cert::load_local_tls_config,
};

use tokio_rustls::{LazyConfigAcceptor, TlsAcceptor};

use std::{
    collections::HashMap,
    net::IpAddr,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use rustls_acme::{AcmeConfig, AcmeState, caches::DirCache, is_tls_alpn_challenge};
use tokio::{
    io::{AsyncRead, AsyncWrite, AsyncWriteExt, BufReader},
    sync::Mutex,
    time::timeout,
};
use tokio_stream::StreamExt;

use std::io::Error;

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> std::io::Result<()> {
    let server_config = config_reader::read_config(PathBuf::from(".env"));

    let state = generate_acme_state(server_config.clone());
    let challenge_rustls_config = state.challenge_rustls_config();
    let default_rustls_config = state.default_rustls_config();
    if server_config.environment == Environment::Production {
        cert_auto_renewal(state);
    }
    let listener = tokio::net::TcpListener::bind(server_config.ip_and_port.clone())
        .await
        .unwrap_or_else(|_| {
            panic!(
                "I expected to bind to address {}",
                server_config.ip_and_port.clone()
            )
        });
    println!(
        "Server listening on https://{}",
        server_config.ip_and_port.clone()
    );

    // for rate limiting
    let global_bucket: GlobalBuckets = Arc::new(Mutex::new(TokenAndTimeBuckets {
        token_bucket: TokenBucket::new(
            server_config.server_global_requests_token_bucket_capacity,
            server_config.server_global_requests_token_bucket_refill_rate_per_millis,
        ),
        time_bucket: TimeBucket::new(
            server_config.server_time_credits_capacity,
            server_config.server_time_credits_refill_rate_per_millis,
        ),
    }));
    let ip_buckets: IpBuckets = Arc::new(Mutex::new(HashMap::new()));

    let db_pool = sqlx::PgPool::connect(&server_config.database_str)
        .await
        .expect("I expected pool construction to succeed :)");

    // redirect from http  to https
    // but only if the main port is 443, since if it isn't then there is no point, I think? Not sure
    if server_config.port == 443 {
        let server_config = server_config.clone();
        let global_bucket = global_bucket.clone();
        let ip_buckets = ip_buckets.clone();
        tokio::spawn(async move {
            match http_to_https_redirect(server_config, global_bucket, ip_buckets).await {
                Ok(_) => {}
                Err(e) => {
                    dbg!(e);
                }
            };
        });
    }

    loop {
        let global_bucket = global_bucket.clone();
        let ip_buckets = ip_buckets.clone();
        let db_pool = db_pool.clone();
        let server_config = server_config.clone();
        let (tcp, peer_ip) =
            match timeout(server_config.common_stream_timeout, listener.accept()).await {
                Ok(Ok((s, a))) => (s, a),
                Ok(Err(e)) => {
                    dbg!("Could not unwrap stream? :( ", e);
                    continue;
                }
                Err(_e) => {
                    continue;
                }
            };

        match (server_config.environment.clone(), server_config.use_tls) {
            (Environment::Local, true) => {
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
            (Environment::Production, true) => {
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
                        tls.shutdown().await.unwrap();
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
                    }
                });
            }
            (_, false) => {
                tokio::spawn(async move {
                    let stream = tcp;
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
        }
    }
}

pub async fn handle_stream<S: AsyncRead + AsyncWrite + Unpin>(
    reader: &mut BufReader<S>,
    db_pool: sqlx::PgPool,
    peer_ip: IpAddr,
    global_bucket: GlobalBuckets,
    ip_buckets: IpBuckets,
    server_config: Config,
) {
    let keep_alive_max_requests_per_connection =
        server_config.keep_alive_max_requests_per_connection;
    let mut requests_count = 0;
    loop {
        dbg!("Peer with IP ", peer_ip, "just issued a request");
        if let Err(error_response) = check_and_consume_buckets(
            Instant::now(),
            &peer_ip,
            &global_bucket,
            &ip_buckets,
            server_config.clone(),
        )
        .await
        {
            if let Err(_e) = timeout(
                server_config.common_stream_timeout,
                reader.write_all(&error_response),
            )
            .await
            {
                dbg!("stream.write_all after bucket check time out");
            }
            break;
        }

        let start = Instant::now();
        let request_data = match timeout(
            server_config.common_stream_timeout,
            parse_http_request(reader, HttpParserConfig::from(server_config.clone())),
        )
        .await
        {
            Ok(Ok(data)) => data,
            Ok(Err(e)) => {
                dbg!(e.clone());
                write_http_parsing_error(e, reader, server_config.common_stream_timeout).await;
                break;
            }
            Err(e) => {
                dbg!("main.rs parse_http_request timed out", e);
                write_http_parsing_error(
                    HttpRequestParsingError::StreamTimeoutError,
                    reader,
                    server_config.common_stream_timeout,
                )
                .await;
                break;
            }
        };
        requests_count += 1;
        let response_data = generate_response(
            request_data,
            db_pool.clone(),
            requests_count > keep_alive_max_requests_per_connection,
            server_config.clone(),
        )
        .await;
        let response_bytes = response_to_bytes(response_data.clone()).await;
        if let Err(e) =
            write_response(reader, response_bytes, server_config.common_stream_timeout).await
        {
            dbg!("Write response returned", e);
            break;
        }

        consume_time_spent_from_time_credits_bucket(
            start,
            peer_ip,
            ip_buckets.clone(),
            server_config.clone(),
        )
        .await;

        let server_wants_to_close = 'sc: {
            let conn_h = response_data.headers.get("connection");
            if let Some(conn_h) = conn_h
                && conn_h == KeepAliveOrClose::KeepAlive.to_string()
            {
                break 'sc false;
            }
            true
        };
        if server_wants_to_close {
            dbg!("server told me to close, closing",);
            break;
        }
    }
}

pub async fn consume_time_spent_from_time_credits_bucket(
    start: Instant,
    peer_ip: IpAddr,
    ip_buckets: IpBuckets,
    server_config: Config,
) {
    let time_elapsed = start.elapsed();
    let mut ip_buckets = ip_buckets.lock().await;
    let ip_bucket = ip_buckets.entry(peer_ip).or_insert_with(|| {
        TokenAndTimeBucketsAndLastSeen::default_with_settings(server_config.clone())
    });
    ip_bucket.time_bucket.consume_time(time_elapsed);
}
pub async fn generate_response(
    request_data: HttpRequestData,
    db_pool: sqlx::PgPool,
    keep_alive_max_requests_reached: bool,
    server_config: Config,
) -> HttpResponseData {
    let mut response_data =
        routing::generate_response_data(request_data.clone(), db_pool, server_config).await;
    let close_or_keep_alive = if keep_alive_max_requests_reached {
        KeepAliveOrClose::Close
    } else {
        let conn = 'c: {
            let hc = response_data.headers.get("connection");
            if let Some(c) = hc.clone() {
                match c.to_ascii_lowercase().as_str() {
                    "close" => break 'c KeepAliveOrClose::Close,
                    "keep-alive" => break 'c KeepAliveOrClose::KeepAlive,
                    _ => {
                        dbg!(format!("{hc:#?}, did not expect that"));
                        break 'c KeepAliveOrClose::Close;
                    }
                }
            }
            dbg!(
                "response_data contains no connection header, I suppose I will close in this case.",
                hc.clone()
            );
            KeepAliveOrClose::Close
        };
        match conn {
            KeepAliveOrClose::Close => KeepAliveOrClose::Close,
            KeepAliveOrClose::KeepAlive => {
                if let Some(connection_header) = request_data.headers.get("connection")
                    && connection_header == "close"
                {
                    KeepAliveOrClose::Close
                } else {
                    KeepAliveOrClose::KeepAlive
                }
            }
        }
    };
    response_data
        .headers
        .insert("connection", close_or_keep_alive.to_string());
    response_data
}
pub async fn write_response<S: AsyncRead + AsyncWrite + Unpin>(
    reader: &mut BufReader<S>,
    response_bytes: Vec<u8>,
    common_stream_timeout: Duration,
) -> Result<(), ()> {
    if let Err(_e) = timeout(common_stream_timeout, reader.write_all(&response_bytes)).await {
        dbg!("main.rs response write timeout");
        return Err(());
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
            let err_response = standalone_response(
                StatusCode::Code414UriTooLong,
                "414 Uri Too Long ☺️".to_string(),
            );
            let _ = timeout(
                common_stream_timeout_duration,
                stream.write_all(&err_response),
            )
            .await;
        }
        HttpRequestParsingError::HttpVersionNotSupported => {
            let err_response = standalone_response(
                StatusCode::Code505HttpVersionNotSupported,
                "505 HTTP Version Not Supported".to_string(),
            );
            let _ = timeout(
                common_stream_timeout_duration,
                stream.write_all(&err_response),
            )
            .await;
        }
        HttpRequestParsingError::TooManyHeaders | HttpRequestParsingError::TooManyCookies => {
            let err_response = standalone_response(
                StatusCode::Code431RequestHeaderFieldsTooLarge,
                "431 Header fields too large, and/or too many of them 😴".to_string(),
            );
            let _ = timeout(
                common_stream_timeout_duration,
                stream.write_all(&err_response),
            )
            .await;
        }

        HttpRequestParsingError::StreamTimeoutError => {
            let err_response = standalone_response(
                StatusCode::Code400BadRequest,
                "400 Bad Request 🙀".to_string(),
            );
            let _ = timeout(
                common_stream_timeout_duration,
                stream.write_all(&err_response),
            )
            .await;
        }
        HttpRequestParsingError::ClientTimedOutBeforeItSentAnyBytes => {
            println!(
                "{}",
                HttpRequestParsingError::ClientTimedOutBeforeItSentAnyBytes
            );
            println!(
                "Doing nothing, I guess, since the client sending nothing probably means they won't receive anything either."
            );
        }
        HttpRequestParsingError::ClientSentNothing => {
            println!("{}", HttpRequestParsingError::ClientSentNothing);
            println!("Doing nothing, because client connected but sent nothing...");
        }
        hm => {
            eprintln!("main error, could not parse request {:?}", hm);
            let err_response = standalone_response(
                StatusCode::Code400BadRequest,
                "400 Bad Request 🙀".to_string(),
            );
            let _ = timeout(
                common_stream_timeout_duration,
                stream.write_all(&err_response),
            )
            .await;
        }
    }
}

pub fn generate_acme_state(server_config: Config) -> AcmeState<Error> {
    AcmeConfig::new([server_config.domain_name.clone()])
        .contact([format!(
            "mailto:{}",
            server_config.lets_encrypt_contact_email.clone()
        )])
        .cache_option(Some(DirCache::new("./rustls_acme_cache")))
        .directory_lets_encrypt(server_config.lets_encrypt_production)
        .state()
}
pub fn cert_auto_renewal(mut state: AcmeState<Error>) {
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
