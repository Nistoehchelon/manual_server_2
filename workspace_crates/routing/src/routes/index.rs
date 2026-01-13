use error_structs::InternalAppError;
use html_templating::html_template_structs::{
    self, IndexLoggedInTemplate, IndexNotLoggedInTemplate, MainInfo,
};

use into_vec::into_vec;
use session::{Session, SessionNotification, read_and_delete_session_notifications};
use user::{User, get_user_by_user_id};

use http_stuff::http_response::ResponseData;

use crate::RouterData;

pub async fn get(
    RouterData {
        db_pool,
        session,
        request_data,
    }: RouterData,
) -> Result<ResponseData, error_structs::InternalAppError> {
    let notifs = read_and_delete_session_notifications(db_pool.clone(), session.id.clone())
        .await
        .map_err(InternalAppError::new)?;
    if session.logged_in {
        let user = get_user_by_user_id(db_pool.clone(), session.user_id.clone())
            .await
            .map_err(InternalAppError::new)?
            .map_err(InternalAppError::new)?;

        return Ok(http_stuff::http_response::ok_html(get_html(
            session.clone(),
            notifs,
            user,
        )));
    }
    Ok(http_stuff::http_response::ok_html(get_html(
        session.clone(),
        notifs,
        User::default(),
    )))
}

fn get_html(session: Session, notifications: Vec<SessionNotification>, user: User) -> String {
    if session.logged_in {
        let index_template = IndexLoggedInTemplate {
            main_info: MainInfo {
                title: format!("Welcome back!"),
                notifications: into_vec(notifications),
            },
            user: html_template_structs::User {
                username: user.username.clone(),
            },
        };
        html_templating::render(index_template)
    } else {
        let index_template = IndexNotLoggedInTemplate {
            main_info: MainInfo {
                title: format!("Welcome!"),
                notifications: into_vec(notifications),
            },
        };
        html_templating::render(index_template)
    }
}
