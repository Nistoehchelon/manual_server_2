use error_structs::InternalAppError;
use http_stuff::http_response::{ ResponseData, redirect_to};
use session::{Session, edit_session};

pub async fn get(db_pool: &sqlx::PgPool, mut session: Session) -> Result<ResponseData, error_structs::InternalAppError> {
    session.logged_in = false;
    session.user_id = "".to_string();
    edit_session(db_pool, session.clone().id, session.clone()).await.map_err(InternalAppError::new)?;

    Ok(redirect_to("/".to_string()))
}
