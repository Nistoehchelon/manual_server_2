use http_stuff::{
    http_request_parsing::{HttpRequestParsingError, parse_http_request},
    http_response::{StatusCode, standalone_response},
};
use server::{
    ssl::generate_self_signed_cert,
    token_buckets_for_requests::{GlobalBucket, IpBuckets, TokenBucket},
};

use std::{collections::HashMap, net::IpAddr, path::PathBuf, sync::Arc, time::Duration};

use tokio::{
    io::{AsyncRead, AsyncWrite, AsyncWriteExt},
    net::TcpListener,
    sync::Mutex,
    time::timeout,
};

use tokio_rustls::TlsAcceptor;
use tokio_rustls::rustls::{
    ServerConfig,
    pki_types::{CertificateDer, PrivateKeyDer},
};

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> std::io::Result<()> {
    let server_config = config_reader::read_config(PathBuf::from("config"));

    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install aws-lc-rs provider");
    let (cert_der, key_der) = generate_self_signed_cert().unwrap();

    let tls_config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(
            vec![CertificateDer::from(cert_der)],
            PrivateKeyDer::Pkcs8(key_der.into()),
        )
        .expect("Invalid configuration");
    dbg!(tls_config.crypto_provider());
    let acceptor = TlsAcceptor::from(Arc::new(tls_config));
    let listener = TcpListener::bind(server_config.ip_and_port.clone())
        .await
        .expect(&format!(
            "I expected to bind to address {}",
            server_config.ip_and_port.clone()
        ));
    println!(
        "Server listening on http://{}",
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
        .expect("Pool construction did not succeed");
    let common_stream_timeout_duration =
        Duration::from_millis(server_config.common_stream_timeout_millis.clone());

    loop {
        let (stream, peer_ip) =
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

        let acceptor = acceptor.clone();
        let global_bucket = global_bucket.clone();
        let ip_buckets = ip_buckets.clone();
        let db_pool = db_pool.clone();
        tokio::spawn(async move {
            let mut stream = match acceptor.accept(stream).await {
                Ok(ts) => ts,
                Err(e) => {
                    dbg!(e);
                    return;
                }
            };
            let _ = handle(
                &mut stream,
                db_pool,
                peer_ip.ip(),
                global_bucket,
                ip_buckets,
                server_config.per_ip_token_bucket_capacity.clone(),
                server_config
                    .per_ip_token_bucket_refill_rate_per_millis
                    .clone(),
                common_stream_timeout_duration.clone(),
            )
            .await;
            let f = timeout(common_stream_timeout_duration.clone(), stream.flush()).await;
            if f.is_err() {
                dbg!("main.rs flush timeout, dropping stream immediately");
                drop(stream);
                return;
            }
            let s = timeout(common_stream_timeout_duration.clone(), stream.shutdown()).await;
            if s.is_err() {
                dbg!("main.rs stream shutdown timed out, dropping it I guess?");
                drop(stream);
                return;
            }
        });
    }
}

pub async fn handle<S: AsyncRead + AsyncWrite + Unpin>(
    stream: &mut S,
    db_pool: sqlx::PgPool,
    peer_ip: IpAddr,
    global_bucket: GlobalBucket,
    ip_buckets: IpBuckets,
    per_ip_token_bucket_capacity: f64,
    per_ip_token_bucket_refill_rate_per_millis: f64,
    common_stream_timeout_duration: Duration,
) -> std::io::Result<()> {
    dbg!("Peer with IP ", peer_ip, "just issued a request");

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
            stream.write_all(&error_response),
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
        parse_http_request(stream, common_stream_timeout_duration.clone()),
    )
    .await
    {
        Ok(Ok(data)) => data,
        Ok(Err(e)) => {
            match e {
                HttpRequestParsingError::UriTooLong => {
                    let err_response = standalone_response(
                        StatusCode::UriTooLong414,
                        format!("414 Uri Too Long ☺️"),
                    );
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
                HttpRequestParsingError::TooManyHeaders
                | HttpRequestParsingError::TooManyCookies => {
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
                    let err_response = standalone_response(
                        StatusCode::BadRequest400,
                        format!("400 Bad Request 🙀"),
                    );
                    let _ = timeout(
                        common_stream_timeout_duration.clone(),
                        stream.write_all(&err_response),
                    )
                    .await;
                }
            };

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
        stream.write_all(&response),
    )
    .await
    {
        dbg!("main.rs response write timeout");
    }

    Ok(())
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
