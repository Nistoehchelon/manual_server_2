use config_reader::Config;
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
    log_tracing,
};

use std::{
    collections::HashMap,
    net::IpAddr,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use tokio::{
    io::{AsyncRead, AsyncWrite, AsyncWriteExt, BufReader},
    sync::Mutex,
    time::timeout,
};

use tracing::{Span, field, info, span::Id};
use tracing_appender::rolling;


const VERSION: &str = "0.0.0.0.2"; // increase when I want to compare which one I am running on prod

#[tokio::main(flavor = "multi_thread")]
async fn main() -> std::io::Result<()> {

    dotenvy::dotenv().ok();

    let server_config = config_reader::read_config(PathBuf::from(".env"));

    // File appender (rotates daily)
    let file_appender =
        rolling::daily(std::env::var("RUST_LOG_DIRECTORY_PATH").unwrap(), "app.log");
    let _guard = self::log_tracing::init_tracing(true, file_appender);

    let tokio_metrics = tokio::runtime::Handle::current().metrics();
    tracing::info!("Num of Tokio worker threads: {}", tokio_metrics.num_workers());
    let listener = tokio::net::TcpListener::bind(server_config.ip_and_port.clone())
        .await
        .unwrap_or_else(|_| {
            panic!(
                "I expected to bind to address {}",
                server_config.ip_and_port.clone()
            )
        });
    tracing::info!("Server version: {}", VERSION);
    tracing::info!("Server listening on {}", server_config.ip_and_port.clone());

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

#[tracing::instrument(skip_all, target = "shared_context", fields(path = field::Empty, remote_ip = field::Empty, user_id = field::Empty, span_id = field::Empty, method = field::Empty))]
pub async fn handle_stream<S: AsyncRead + AsyncWrite + Unpin>(
    reader: &mut BufReader<S>,
    db_pool: sqlx::PgPool,
    peer_ip: IpAddr,
    global_bucket: GlobalBuckets,
    ip_buckets: IpBuckets,
    server_config: Config,
) {
    Span::current().record(
        "span_id",
        Span::current()
            .id()
            .unwrap_or(Id::from_u64(1))
            .to_owned()
            .into_u64()
            .to_string(),
    );
    info!("New Connection...");
    let keep_alive_max_requests_per_connection =
        server_config.keep_alive_max_requests_per_connection;
    let mut requests_count = 0;
    let mut ip: Option<IpAddr> = None;
    loop {
        //dbg!("Peer with IP ", peer_ip, "just issued a request");
        tracing::info!("Processing a request");
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
                info!("{}", e.clone());
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
        match ip {
            None => {
                ip = Some(request_data.x_real_ip);
                info!("Retrieved the user IP {}", request_data.x_real_ip);
                Span::current().record("remote_ip", request_data.x_real_ip.to_string());
            }
            Some(ip_unwrapped) => {
                if request_data.x_real_ip != ip_unwrapped {
                    // I mean, I dunno if this can even happen in one kept-alive connection or if it will just drop and a new one, unrelated to current will be init'ed... But this can't hurt? Maybe? I dunno
                    info!(
                        "User ip changed from {} to {}!",
                        ip_unwrapped, request_data.x_real_ip
                    );
                    ip = Some(request_data.x_real_ip);
                    Span::current().record("remote_ip", ip_unwrapped.to_string());
                }
            }
        }
        info!(
            "User is accessing method {} and path {}",
            request_data.method.clone(),
            request_data.path_and_query.clone()
        );
        Span::current().record("path", request_data.path_and_query.clone());
        Span::current().record("method", request_data.method.clone().to_string());

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
            tracing::info!("Server instructed me to close, so closing the connection");
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
            info!("Doing nothing, because client connected but sent nothing...");
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
