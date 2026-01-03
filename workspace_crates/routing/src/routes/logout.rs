use http_stuff::http_response::{HttpResponse, redirect_to};
use session::{Session, edit_session};

pub fn get(db_pool: &db_manager::DbPool, mut session: Session) -> HttpResponse {
    session.logged_in = false;
    session.user_public_inapp_id = "".to_string();
    edit_session(db_pool, session.clone().id, session.clone());

    redirect_to("/".to_string())
}
