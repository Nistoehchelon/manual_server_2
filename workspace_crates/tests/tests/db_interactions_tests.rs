use db_manager;
use config_reader::read_config;
use session;

#[tokio::test(flavor="multi_thread", worker_threads = 2)]
async fn test_read_write() {
    let env = read_config();
    let pool = db_manager::get_pool(env.database_str).expect("Could not connect to DB");
    let client = db_manager::get_client(&pool).await.expect("Could not get client conn from db pool");
    let session = session::Session {
        public_id: format!("32ed32"),
        user_public_id: format!("user_id_1232"),
        logged_in: false,
    };
    client
        .execute(
            "INSERT INTO public.sessions (id, user_id, logged_in) VALUES ($1, $2, $3)",
            &[
                &session.public_id,
                &session.user_public_id,
                &session.logged_in,
            ],
        ).await
        .unwrap();
    // now test for retrieving?;

    let retrieved = client
        .query_one(
            "SELECT id, user_id, logged_in FROM public.sessions WHERE id = $1",
            &[&session.public_id],
        ).await
        .unwrap();

    let retrieved_session = session::Session {
        public_id: retrieved.get("public_id"),
        user_public_id: retrieved.get("user_id"),
        logged_in: retrieved.get("logged_in"),
    };

    assert_eq!(session.public_id, retrieved_session.public_id);
    assert_eq!(
        session.user_public_id,
        retrieved_session.user_public_id
    );
    assert_eq!(session.logged_in, retrieved_session.logged_in);

    client
        .execute("DELETE FROM public.sessions WHERE id = $1", &[&session.public_id])
        .await
        .unwrap();
}
