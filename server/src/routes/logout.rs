use crate::{
    redirect::to,
    session::{Session, edit_session},
};

pub fn get(mut session: Session) -> String {
    session.logged_in = false;
    session.user_id = "".to_string();
    edit_session(session.clone());

    to("/".to_string())
}
