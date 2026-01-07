use http_stuff::http_response::{HttpResponse, internal_server_error_500, redirect_to};
use session::{Session, edit_session};

pub async fn get(db_pool: &sqlx::PgPool, mut session: Session) -> HttpResponse {
    session.logged_in = false;
    session.user_public_id = "".to_string();
    match edit_session(db_pool, session.clone().public_id, session.clone()).await {
        Ok(_) => {},
        Err(e) => {
            dbg!(e);
            return internal_server_error_500();
        }
    };

    redirect_to("/".to_string())
}
