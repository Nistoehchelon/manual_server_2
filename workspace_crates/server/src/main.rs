use http_stuff::{
    http_request_parsing::{HttpRequestParsingError, parse_http_request},
    http_response::{StatusCode, standalone_response},
};
use server::token_buckets_for_requests::{GlobalBucket, IpBuckets, TokenBucket};

use std::{collections::HashMap, net::IpAddr, path::PathBuf, sync::Arc, time::Duration};

use tokio::{
    io::{AsyncRead, AsyncWrite, AsyncWriteExt 
    },
    net::{TcpListener},
    sync::Mutex, time::timeout,
};






const LISTENER_ACCEPT_TIMEOUT_SECS: u64 = 1;
const COMMON_STREAM_TIMEOUT_SECS: u64 = 1;


use rcgen::generate_simple_self_signed;

fn generate_self_signed_cert() -> std::io::Result<(Vec<u8>, Vec<u8>)> {
    let subject_alt_names = vec!["localhost".to_string(), "127.0.0.1".to_string()];
    let cert_key = generate_simple_self_signed(subject_alt_names).unwrap();
    
    // Serialize to DER format for rustls
    let cert_der = cert_key.cert.der().to_vec();
    let key_der = cert_key.signing_key.serialize_der();
    
    Ok((cert_der, key_der))
}




#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> std::io::Result<()> {

    let (cert_der, key_der) = generate_self_signed_cert().unwrap();
    use tokio_rustls::rustls::{ServerConfig, pki_types::{CertificateDer, PrivateKeyDer}};
    use tokio_rustls::TlsAcceptor;

    let config = ServerConfig::builder()
    .with_no_client_auth()
    .with_single_cert(
        vec![CertificateDer::from(cert_der)], 
        PrivateKeyDer::Pkcs8(key_der.into())
    )
    .expect("Invalid configuration");

    let acceptor = TlsAcceptor::from(Arc::new(config));


    let env = config_reader::read_config(PathBuf::from("config"));
    let config_reader::Config {
        ip_and_port,
        database_str,
        server_global_requests_token_bucket_capacity,
        server_global_requests_token_bucket_refill_rate_per_millis,
        per_ip_token_bucket_capacity,
        per_ip_token_bucket_refill_rate_per_millis,
    } = env;
    let listener = TcpListener::bind(ip_and_port.clone())
        .await
        .expect(&format!("I expected to bind to address {ip_and_port}"));
    let db_pool = sqlx::PgPool::connect(&database_str)
        .await
        .expect("Pool construction did not succeed");
    println!("Server listening on http://{}", ip_and_port.clone());

    let global_bucket: GlobalBucket = Arc::new(Mutex::new(TokenBucket::new(
        server_global_requests_token_bucket_capacity,
        server_global_requests_token_bucket_refill_rate_per_millis,
    )));

    let ip_buckets: IpBuckets = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let acceptor = acceptor.clone();

        let (stream, peer_ip) = match timeout(Duration::from_secs(LISTENER_ACCEPT_TIMEOUT_SECS), listener.accept()).await {
            Ok(Ok((s, a))) => (s, a),
            Ok(Err(e)) => {
                dbg!("Could not unwrap stream? :( ", e);
                continue;
            },
            Err(_e) => {
                continue;
            }
        };
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
                per_ip_token_bucket_capacity,
                per_ip_token_bucket_refill_rate_per_millis,
            )
            .await;
            let f = timeout(Duration::from_secs(COMMON_STREAM_TIMEOUT_SECS), stream.flush()).await;
            if f.is_err() {
                dbg!("main.rs flush timeout, dropping stream immediately");
                drop(stream);
                return;
            }
            let s = timeout(Duration::from_secs(COMMON_STREAM_TIMEOUT_SECS), stream.shutdown()).await;
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
        match timeout(Duration::from_secs(COMMON_STREAM_TIMEOUT_SECS), stream.write_all(&error_response)).await {
            Err(_e) => {
                dbg!("stream.write_all after bucket check time out");
            },
            _ => {},
        }
        return Ok(());
    }

    let request_data = match timeout(Duration::from_secs(COMMON_STREAM_TIMEOUT_SECS), parse_http_request(stream)).await {
        Ok(Ok(data)) => data,
        Ok(Err(e)) => {
            match e {
                HttpRequestParsingError::UriTooLong => {
                    let err_response = standalone_response(
                        StatusCode::UriTooLong414,
                        format!("414 Uri Too Long ☺️"),
                    );
                    let _ = timeout(Duration::from_secs(COMMON_STREAM_TIMEOUT_SECS), stream.write_all(&err_response)).await;
                }
                HttpRequestParsingError::HttpVersionNotSupported => {
                    let err_response = standalone_response(
                        StatusCode::HttpVersionNotSupported505,
                        format!("505 HTTP Version Not Supported"),
                    );
                    let _ = timeout(Duration::from_secs(COMMON_STREAM_TIMEOUT_SECS), stream.write_all(&err_response)).await;
                }
                HttpRequestParsingError::TooManyHeaders
                | HttpRequestParsingError::TooManyCookies => {
                    let err_response = standalone_response(
                        StatusCode::RequestHeaderFieldsTooLarge431,
                        format!("431 Header fields too large, and/or too many of them 😴"),
                    );
                    let _ = timeout(Duration::from_secs(COMMON_STREAM_TIMEOUT_SECS), stream.write_all(&err_response)).await;
                }
                _ => {
                    eprintln!("main error, could not parse request {:?}", e);
                    let err_response = standalone_response(
                        StatusCode::BadRequest400,
                        format!("400 Bad Request 🙀"),
                    );
                    let _ = timeout(Duration::from_secs(COMMON_STREAM_TIMEOUT_SECS), stream.write_all(&err_response)).await;
                }
            };

            return Ok(());
        },
        Err(_e) => {
            dbg!("main.rs parse_http_request timed out");
            return Ok(());
        }
    };

    let response = routing::generate_response(request_data, db_pool).await;
    if let Err(_e) = timeout(Duration::from_secs(COMMON_STREAM_TIMEOUT_SECS), stream.write_all(&response)).await {
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
