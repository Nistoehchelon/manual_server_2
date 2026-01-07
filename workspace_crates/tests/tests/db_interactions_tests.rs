use config_reader::read_config;
use session;

#[tokio::test(flavor="multi_thread", worker_threads = 2)]
async fn test_read_write() {
    let env = read_config();
    let pool = sqlx::PgPool::connect(&env.database_str).await.expect("No pool :(");
    let session = session::Session {
        public_id: format!("32ed32"),
        user_public_id: format!("user_id_1232"),
        logged_in: false,
    };
    sqlx::query!(
            "INSERT INTO public.sessions (public_id, user_id, logged_in) VALUES ($1, $2, $3)",
                session.public_id,
                session.user_public_id,
                session.logged_in).execute(&pool).await.unwrap();
    // now test for retrieving?;

    let retrieved = sqlx::query!(
            "SELECT public_id, user_id, logged_in FROM public.sessions WHERE public_id = $1",
            session.public_id
        ).fetch_one(&pool).await
        .unwrap();

    let retrieved_session = session::Session {
        public_id: retrieved.public_id,
        user_public_id: retrieved.user_id,
        logged_in: retrieved.logged_in,
    };

    assert_eq!(session.public_id, retrieved_session.public_id);
    assert_eq!(
        session.user_public_id,
        retrieved_session.user_public_id
    );
    assert_eq!(session.logged_in, retrieved_session.logged_in);

   sqlx::query!("DELETE FROM public.sessions WHERE public_id = $1", session.public_id).execute(&pool)
        .await
        .unwrap();
}
