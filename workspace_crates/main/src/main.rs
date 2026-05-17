use std::path::PathBuf;


#[tokio::main(flavor = "multi_thread")]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    dotenvy::dotenv().ok();
    let site_config = config_reader::read_config::<site_config::SiteConfig>(PathBuf::from(".env"));

    let db_pool = sqlx::PgPool::connect(&site_config.database_url)
        .await
        .expect("I expected pool construction to succeed :)");
    let handler: server::run::HandlerFn =
        std::sync::Arc::new(move |request_data, server_config| {
            let db_pool = db_pool.clone();
            let site_config = site_config.clone();
            Box::pin(async move {
                routing::generate_response_data(request_data, db_pool, server_config, site_config)
                    .await
            })
        });
    server::run(handler, app_executable_version::VERSION.to_string()).await
}
