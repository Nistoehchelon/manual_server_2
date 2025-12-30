use db_manager;
use env_reader::get_env;
use session;


#[test]
fn test_read_write() {
    let env = get_env();
    let pool = db_manager::get_pool(env.database_str);
    let mut client = db_manager::get_client(pool);
    let session = session::Session {
        id: format!("32ed32"),
        user_public_inapp_id: format!("user_id_1232"),
        logged_in: false,
    };
    client.execute("INSERT INTO public.sessions (id, user_id, logged_in) VALUES ($1, $2, $3)", &[&session.id, &session.user_public_inapp_id, &session.logged_in]).unwrap();
    // now test for retrieving?;

     let retrieved = client.query_one(
        "SELECT id, user_id, logged_in FROM public.sessions WHERE id = $1",
        &[&session.id]
    ).unwrap();

     let retrieved_session = session::Session {
        id: retrieved.get("id"),
        user_public_inapp_id: retrieved.get("user_id"),
        logged_in: retrieved.get("logged_in"),
    };

    assert_eq!(session.id, retrieved_session.id);
    assert_eq!(session.user_public_inapp_id, retrieved_session.user_public_inapp_id);
    assert_eq!(session.logged_in, retrieved_session.logged_in);

        client.execute(
        "DELETE FROM public.sessions WHERE id = $1",
        &[&session.id]
    ).unwrap();
    
}