use std::time::Duration;

use tokio::{
    io::{ AsyncWriteExt},
    net::TcpListener, time::timeout,
};

pub async fn http_to_https_redirect(domain: String, ip: String, port: u64, common_stream_timeout_duration: Duration) -> tokio::io::Result<()> {
    let listener = TcpListener::bind(format!("{ip}:{port}").as_str()).await?;
    loop {
        let domain = domain.clone();
        let (mut stream, _addr) = listener.accept().await?;
        tokio::spawn(async move {
            let mut location = format!("https://{}", domain);
            if port != 80 {
                location.push_str(format!(":{port}").as_str());
            }
            location.push_str("/");
            

            let response = format!(
                "HTTP/1.1 308 Permanent Redirect\r\n\
                 Location: {}\r\n\
                 Strict-Transport-Security: max-age=999999999; includeSubDomains; preload\r\n\
                 Connection: close\r\n\r\n",
                location
            );

            let _ = timeout(common_stream_timeout_duration, stream.write_all(response.as_bytes())).await;
            let _ = timeout(common_stream_timeout_duration, stream.shutdown()).await;

        });
    }
}
