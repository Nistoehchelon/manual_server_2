use config_reader::read_config;
use site_config::SiteConfig;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_read_write() {
    use std::env;
    use std::path::PathBuf;

    let config_path = env::var("WORKSPACE_ROOT_CONFIG_FILE")
        .map(PathBuf::from)
        .expect("WORKSPACE_ROOT_CONFIG_FILE must be set");

    let env = read_config::<SiteConfig>(config_path);
    let pool = sqlx::PgPool::connect(&env.database_url)
        .await
        .expect("No pool :(");
    let session = session::Session {
        previous_get_page: "".to_string(),
        id: "32ed32".to_string(),
        user_id: "user_id_1232".to_string(),
        logged_in: false,
        previous_page_load_time_ms: 0,
    };

    sqlx::query!(
        "INSERT INTO public.sessions (id, user_id, logged_in) VALUES ($1, $2, $3)",
        session.id,
        session.user_id,
        session.logged_in
    )
    .execute(&pool)
    .await
    .unwrap();
    // now test for retrieving?;

    let retrieved = sqlx::query!(
        "SELECT id, user_id, logged_in FROM public.sessions WHERE id = $1",
        session.id
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    let retrieved_session = session::Session {
        previous_get_page: "".to_string(),
        id: retrieved.id,
        user_id: retrieved.user_id,
        logged_in: retrieved.logged_in,
        previous_page_load_time_ms: 0,
    };

    assert_eq!(session.id, retrieved_session.id);
    assert_eq!(session.user_id, retrieved_session.user_id);
    assert_eq!(session.logged_in, retrieved_session.logged_in);

    sqlx::query!("DELETE FROM public.sessions WHERE id = $1", session.id)
        .execute(&pool)
        .await
        .unwrap();
}
