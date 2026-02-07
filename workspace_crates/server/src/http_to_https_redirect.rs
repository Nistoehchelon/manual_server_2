use std::time::{Duration, Instant};

use config_reader::Config;
use tokio::{io::AsyncWriteExt, net::TcpListener, time::timeout};

use crate::{
    check_and_consume_buckets::check_and_consume_buckets,
    token_buckets_for_requests::{GlobalBuckets, IpBuckets},
};

pub async fn http_to_https_redirect(
    config: Config,
    global_bucket: GlobalBuckets,
    ip_buckets: IpBuckets,
    common_stream_timeout_duration: Duration,
) -> tokio::io::Result<()> {
    let Config {
        domain_name: domain,
        ip,
        ..
    } = config.clone();
    let listener_at = format!("{ip}:80");
    let listener = match TcpListener::bind(&listener_at).await {
        Ok(l) => {
            dbg!(format!(
                "Started http_to_https_redirect listener on http://{listener_at}"
            ));
            l
        }
        Err(e) => {
            dbg!("Error starting http_to_https_redirect at ", listener_at);
            return Err(e);
        }
    };
    loop {
        let domain = domain.clone();
        let (mut stream, peer_ip) = match listener.accept().await {
            Ok((s, pi)) => (s, pi),
            Err(e) => {
                dbg!(e);
                continue;
            }
        };
        let peer_ip = peer_ip.ip();
        if let Err(error_response) = check_and_consume_buckets(
            Instant::now(),
            &peer_ip,
            &global_bucket,
            &ip_buckets,
            config.clone(),
        )
        .await
        {
            match timeout(
                Duration::from_millis(200),
                stream.write_all(&error_response),
            )
            .await
            {
                Err(e) => {
                    dbg!("Timeout", e);
                }
                Ok(Err(e)) => {
                    dbg!("Stream write error", e);
                }
                Ok(Ok(_)) => {}
            }
            let _ = stream.shutdown().await;
            continue;
        }
        tokio::spawn(async move {
            let mut location = String::from("https://");
            location.push_str(domain.as_str());
            location.push_str("/");

            let response = format!(
                "HTTP/1.1 308 Permanent Redirect\r\nLocation: {}\r\nConnection: close\r\n\r\n",
                location
            );

            let _ = timeout(
                common_stream_timeout_duration,
                stream.write_all(response.as_bytes()),
            )
            .await;
            let _ = timeout(common_stream_timeout_duration, stream.shutdown()).await;
        });
    }
}
