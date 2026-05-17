use std::path::PathBuf;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    let server_config = config_reader::read_config(PathBuf::from(".env"));

    let db_pool = sqlx::PgPool::connect(&server_config.database_url)
        .await
        .expect("I expected pool construction to succeed :)");
    let handler: server::run::HandlerFn = std::sync::Arc::new(move |request_data, server_config| {
        let db_pool = db_pool.clone();
        Box::pin(async move { routing::generate_response_data(request_data, db_pool, server_config).await })
    });
    server::run(handler).await
}
