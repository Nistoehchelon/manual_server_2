use std::path::PathBuf;

use server::{BackgroundSpawnTasks, server_config};


#[tokio::main(flavor = "multi_thread")]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    let site_config = config_reader::read_config::<site_config::SiteConfig>(PathBuf::from(".env"));
    let server_config = server_config::ServerConfig::new(site_config.ip.clone(), site_config.port, site_config.rust_log_directory_path.clone());
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
    server::run(server_config.clone(), app_executable_version::VERSION.to_string(), handler, BackgroundSpawnTasks { new_tasks_to_spawn: None, cancellation_token: None}).await
}
