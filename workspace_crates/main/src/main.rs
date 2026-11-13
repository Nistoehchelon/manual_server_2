use std::{path::PathBuf, time::Duration};

use server::{ServerBuilder, server_config};
use tokio::time::timeout;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    let site_config = config_reader::read_config::<site_config::SiteConfig>(PathBuf::from(".env"));
    let server_config = server_config::ServerConfig::new(
        site_config.ip.clone(),
        site_config.port,
        site_config.rust_log_directory_path.clone(),
    );

    // Doing this before constructing DB pool since I might sometimes want to call --v without setting up any db just to see if the executable can start.
    if let Ok(args) = server::passing_arguments_to_executable::parse_args()
        && args.version
    {
        println!("App version: {}", app_executable_version::VERSION);
        return Ok(());
    }
    tracing::debug!("Trying to connect to DB");
    let db_pool = match timeout(
        Duration::from_secs(5),
        sqlx::PgPool::connect(&site_config.database_url),
    )
    .await
    {
        Err(timeout_err) => {
            panic!("DB Connection timed out: {timeout_err}");
        }
        Ok(Ok(pool)) => pool,
        Ok(Err(db_error)) => {
            panic!("DB Error: {db_error}");
        }
    };
    tracing::debug!("Constructed DB Pool successfuly! Yay!");

    let handler_function = server::HandlerFn::new(move |request_data, server_config| {
        let db_pool = db_pool.clone();
        let site_config = site_config.clone();
        async move {
            routing::generate_response_data(request_data, db_pool, server_config, site_config).await
        }
    });
    tracing::debug!("Starting server: Running server handler");
    ServerBuilder::new(
        server_config.clone(),
        app_executable_version::VERSION.to_string(),
        handler_function,
    )
    .run_handler()
    .await
}
