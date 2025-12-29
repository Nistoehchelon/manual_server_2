
use 
    html_templating::html_template_structs::{IndexLoggedInTemplate, IndexNotLoggedInTemplate, MainInfo};

    use session::{Session, SessionNotification, read_and_delete_session_notifications};
    use user::{ User, get_user_by_user_id};
 
pub fn get(session: Session, db_pool: &db_manager::DbPool) -> String {

        let mut user = User::default();
        if session.logged_in {
            user = get_user_by_user_id(session.user_id.clone(), db_pool).unwrap();
        }
        crate::http_response::ok_html(get_html(
            session.clone(),
            read_and_delete_session_notifications(db_pool, session.id.clone()), user,
        ))
}

fn get_html(session: Session, notifications: Vec<SessionNotification>, user: User) -> String {
    if session.logged_in {
        let index_template = IndexLoggedInTemplate {
            main_info: MainInfo {
                title: format!("Welcome back!"),
                notifications,
            },
            user,
        };
        html_templating::render(index_template)
    } else {
        let index_template = IndexNotLoggedInTemplate {
            main_info: MainInfo {
                title: format!("Welcome!"),
                notifications,
            },
        };
        html_templating::render(index_template)
    }
}
