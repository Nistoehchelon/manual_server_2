use crate::{
    redirect::to,

};
use session::{Session, edit_session};

pub fn get(db_pool: &db_manager::DbPool, mut session: Session) -> String {
    session.logged_in = false;
    session.user_id = "".to_string();
    edit_session(db_pool, session.clone().id, session.clone());

    to("/".to_string())
}
