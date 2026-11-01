use std::path::PathBuf;

use server::{ServerBuilder, server_config};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    let site_config = config_reader::read_config::<site_config::SiteConfig>(PathBuf::from(".env"));
    let server_config = server_config::ServerConfig::new(
        site_config.ip.clone(),
        site_config.port,
        site_config.rust_log_directory_path.clone(),
    );
    tracing::debug!("Trying to connect to DB");
    let db_pool = sqlx::PgPool::connect(&site_config.database_url)
        .await
        .expect("I expected pool construction to succeed :)");
    tracing::debug!("Constructed DB Pool successfuly! Yay!");

    let handler_function = server::HandlerFn::new(move |request_data, server_config| {
        let db_pool = db_pool.clone();
        let site_config = site_config.clone();
        async move {
            routing::generate_response_data(request_data, db_pool, server_config, site_config).await
        }
    });
    tracing::debug!("Starting server: Running server handler");
    ServerBuilder::new(server_config.clone(), app_executable_version::VERSION.to_string(), handler_function).run_handler().await
}
