
use std::{fmt::{self, Display}, str::FromStr};

use id_generator::generate_id;

#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub logged_in: bool,
}

#[derive(Clone, Debug)]
pub enum SessionNotificationType {
    Information,
    Error,
    Success
}
impl Display for SessionNotificationType {

    
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            SessionNotificationType::Information => "information",
            SessionNotificationType::Error => "error",
            SessionNotificationType::Success => "success",
        };
        write!(f, "{}", s)
    }
}
impl FromStr for SessionNotificationType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "error" => Ok(Self::Error),
            "information" => Ok(Self::Information),
            "success" => Ok(Self::Success),
            val => Err(format!("Invalid type {val}")),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SessionNotification {
    pub session_id: String,
    pub message: String,
    pub notification_type: SessionNotificationType,
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

pub fn edit_session(db_pool: &db_manager::DbPool, session_id: String, modified_session: Session) {
    storage_specific_edit_session(db_pool, session_id, modified_session);
}

pub fn add_session_notification(db_pool: &db_manager::DbPool, session_notification: SessionNotification) {
    storage_specific_add_session_notification(db_pool, session_notification);
}

pub fn read_and_delete_session_notifications(db_pool: &db_manager::DbPool, session_id: String) -> Vec<SessionNotification> {
    let notifications_associated_with_the_session_id =
        storage_specific_read_and_delete_session_notifications(db_pool, session_id);
    notifications_associated_with_the_session_id
}

pub fn session_with_specified_id_exists(session_id: String, db_pool: &db_manager::DbPool) -> bool {
    if get_session_by_session_id(session_id.clone(), db_pool).is_none() {
        return false;
    }
    true
}

pub fn get_session_by_session_id(session_id: String, db_pool: &db_manager::DbPool) -> Option<Session> {
    storage_specific_get_session_by_session_id(db_pool, session_id)
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
    client.execute("INSERT INTO public.sessions (id, user_id, logged_in) VALUES ($1, $2, $3)", &[&session.id, &session.user_id, &session.logged_in]).unwrap();
}

fn storage_specific_get_session_by_session_id(db_pool: &db_manager::DbPool, session_id: String, ) -> Option<Session> {
    let mut client = db_manager::get_client(db_pool.clone());
    let retrieved = client.query_one("SELECT id, user_id, logged_in FROM sessions WHERE id = $1", &[&session_id]);
    if retrieved.is_err() {
        return None;
    }
    let retrieved = retrieved.unwrap();

    let retrieved_session = Session {
        id: retrieved.get("id"),
        user_id: retrieved.get("user_id"),
        logged_in: retrieved.get("logged_in"),
    };
    return Some(retrieved_session);

}


fn storage_specific_read_and_delete_session_notifications(db_pool: &db_manager::DbPool,
    session_id: String,
) -> Vec<SessionNotification> {
let mut notifications: Vec<SessionNotification> = vec![];
 let mut client = db_manager::get_client(db_pool.clone());
 let mut transaction = client.transaction().unwrap();
 let rows = transaction.query("SELECT session_id, message, notification_type
  FROM public.session_notifications WHERE session_id = $1", &[&session_id]).unwrap();
  if rows.len() == 0 {
    return notifications;
  }
  for row in rows.iter() {
    let notif_type: SessionNotificationType = match row.get("notification_type") {
        "information" => SessionNotificationType::Information,
        "error" => SessionNotificationType::Error,
        "success" => SessionNotificationType::Success,
        val => panic!("Invalid {val} value in notification_type in DB"),
    };
    notifications.push(SessionNotification {
        notification_type: notif_type,
        message: row.get("message"),
        session_id: row.get("session_id"),
    });
  }

  transaction.execute("DELETE FROM  public.session_notifications WHERE session_id = $1 ", &[&session_id]).unwrap();
  transaction.commit().unwrap();

  return notifications;
}

fn storage_specific_edit_session(db_pool: &db_manager::DbPool, session_id: String, modified_session: Session) {
    if !get_session_by_session_id(session_id.clone(), db_pool).is_some() {
        return;
    }
    let mut client = db_manager::get_client(db_pool.clone());

   let query  = "UPDATE public.sessions SET user_id = $1, logged_in = $2, id = $3 WHERE id = $4";

   let _ = client.execute(query, &[&modified_session.user_id, &modified_session.logged_in, &modified_session.id, &session_id]).unwrap();

}

fn storage_specific_add_session_notification(db_pool: &db_manager::DbPool, session_notification: SessionNotification) {
    let notification_type = session_notification.notification_type.to_string();

    let mut client = db_manager::get_client(db_pool.clone());
    let query = "INSERT INTO public.session_notifications (session_id, message, notification_type) VALUES ($1, $2, $3)";
    client.execute(query, &[&session_notification.session_id, &session_notification.message, &notification_type]).unwrap();
}