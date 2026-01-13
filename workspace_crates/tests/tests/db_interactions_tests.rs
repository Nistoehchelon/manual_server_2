use config_reader::read_config;
use session;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_read_write() {
    let env = read_config();
    let pool = sqlx::PgPool::connect(&env.database_str)
        .await
        .expect("No pool :(");
    let session = session::Session {
        id: format!("32ed32"),
        user_id: format!("user_id_1232"),
        logged_in: false,
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
        id: retrieved.id,
        user_id: retrieved.user_id,
        logged_in: retrieved.logged_in,
    };

    assert_eq!(session.id, retrieved_session.id);
    assert_eq!(session.user_id, retrieved_session.user_id);
    assert_eq!(session.logged_in, retrieved_session.logged_in);

    sqlx::query!("DELETE FROM public.sessions WHERE id = $1", session.id)
        .execute(&pool)
        .await
        .unwrap();
}
