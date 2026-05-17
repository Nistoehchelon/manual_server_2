#[tokio::main(flavor = "multi_thread")]
async fn main() -> std::io::Result<()>{
    let handler: server::run::HandlerFn = std::sync::Arc::new(move |req, pool, conf| {
        Box::pin(async move {
            routing::generate_response_data(req, pool, conf).await
        })
    });
    server::run::run(handler).await
}
