
use std::ops::Deref;

use id_generator::generate_id;

#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub logged_in: bool,
}

#[derive(Clone, Debug)]
pub struct SessionNotification {
    pub session_id: String,
    pub message: String,
}

pub fn notifications_to_str(notifications: Vec<SessionNotification>) -> String {
    let mut notifications_str = String::from("<div id=\"notifications\">");
    for notification in notifications {
        notifications_str.push_str(notification.message.as_str());
        notifications_str.push_str("<br>");
    }
    notifications_str.push_str("</div>");
    notifications_str
}

pub fn edit_session(modified_session: Session) {
    storage_specific_edit_session(modified_session);
}

pub fn add_session_notification(session_id: String, message: String, db_pool: &db_manager::DbPool) {
    storage_specific_add_session_notification(session_id, message, db_pool);
}

pub fn read_and_delete_session_notifications(session_id: String) -> Vec<SessionNotification> {
    let notifications_associated_with_the_session_id =
        storage_specific_read_and_delete_session_notifications(session_id);
    notifications_associated_with_the_session_id
}

pub fn session_with_specified_id_exists(session_id: String, db_pool: &db_manager::DbPool) -> bool {
    if get_session_by_session_id(session_id.clone(), db_pool).is_none() {
        return false;
    }
    true
}

pub fn get_session_by_session_id(session_id: String, db_pool: &db_manager::DbPool) -> Option<Session> {
    storage_specific_get_session_by_session_id(session_id, db_pool)
}

pub fn generate_session_id() -> String {
    generate_id()
}

pub fn create_new_session_with_id(new_id: String, db_pool: &db_manager::DbPool) -> Session {
    let new_session = Session {
        id: new_id.clone(),
        user_id: String::new(),
        logged_in: false,
    };

    storage_specific_add_session(new_session.clone(), db_pool);
    new_session
}

// separation between general functions above and storage specific stuff (txt vs mysql)

fn storage_specific_add_session(session: Session, db_pool: &db_manager::DbPool) {
    let mut client = db_manager::get_client(db_pool.clone());
    client.execute("INSERT INTO sessions (id, user_id, logged_in) VALUES ($1, $2, $3)", &[&session.id, &session.user_id, &session.logged_in]).unwrap();
}

fn storage_specific_get_session_by_session_id(session_id: String, db_pool: &db_manager::DbPool) -> Option<Session> {
 todo!()
}

fn storage_specific_read_and_delete_session_notifications(
    session_id: String,
) -> Vec<SessionNotification> {
    let all_session_notifications = read_session_notifications();
    let mut notifications_associated_with_the_session_id: Vec<SessionNotification> = Vec::new();
    let mut new_all_session_notifications: Vec<SessionNotification> = Vec::new();
    for msg in all_session_notifications.clone() {
        if msg.session_id != session_id {
            new_all_session_notifications.push(msg);
            continue;
        }
        notifications_associated_with_the_session_id.push(msg.clone());
    }
    write_session_notifications(new_all_session_notifications);
    notifications_associated_with_the_session_id
}

fn storage_specific_edit_session(modified_session: Session) {
    let sessions = read_sessions();
    let mut new_sessions: Vec<Session> = Vec::new();
    for old_session in sessions.clone() {
        if modified_session.id == old_session.id {
            new_sessions.push(modified_session.clone());
            continue;
        }
        new_sessions.push(old_session);
    }
    write_sessions(new_sessions);
}

fn storage_specific_add_session_notification(session_id: String, message: String, db_pool: &db_manager::DbPool) {
    let notification = SessionNotification {
        session_id,
        message,
    };
    let mut all_notifications = read_session_notifications();
    all_notifications.push(notification);
    write_session_notifications(all_notifications);
}

fn write_session_notifications(session_notifications: Vec<SessionNotification>) {
    todo!();
}

fn read_session_notifications() -> Vec<SessionNotification> {
    todo!();
}

fn write_sessions(sessions: Vec<Session>) {
    todo!();
}

fn read_sessions() -> Vec<Session> {
    todo!();
}
