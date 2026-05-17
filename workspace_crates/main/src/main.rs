use std::path::PathBuf;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    let server_config = config_reader::read_config(PathBuf::from(".env"));

    let db_pool = sqlx::PgPool::connect(&server_config.database_url)
        .await
        .expect("I expected pool construction to succeed :)");
    let handler: server::run::HandlerFn = std::sync::Arc::new(move |req, conf| {
        let db_pool = db_pool.clone();
        Box::pin(async move { routing::generate_response_data(req, db_pool, conf).await })
    });
    server::run::run(handler).await
}
