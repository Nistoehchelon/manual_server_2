use std::{
    fmt::{self, Display},
    str::FromStr,
};

pub const NAME_OF_COOKIE_CONTAINING_SESSION_ID: &str = "session_id";

pub fn get_name_of_cookie_containing_session_id() -> String {
    NAME_OF_COOKIE_CONTAINING_SESSION_ID.to_string()
}
use db_manager::{DbError, DbSuccess};
use id_generator::generate_id;

#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub user_public_inapp_id: String,
    pub logged_in: bool,
}

#[derive(Clone, Debug)]
pub enum SessionNotificationType {
    Information,
    Error,
    Success,
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

pub fn edit_session(db_pool: &db_manager::DbPool, session_id: String, modified_session: Session) {
    let _ = storage_specific_edit_session(db_pool, session_id, modified_session);
}

pub fn add_session_notification(
    db_pool: &db_manager::DbPool,
    session_notification: SessionNotification,
) -> Result<bool, DbError> {
    match storage_specific_add_session_notification(db_pool, session_notification) {
        Ok(_) => Ok(true),
        _ => Err(DbError {
            message: format!(":("),
        }),
    }
}

pub fn read_and_delete_session_notifications(
    db_pool: &db_manager::DbPool,
    session_id: String,
) -> Result<Vec<SessionNotification>, DbError> {
    let notifs = storage_specific_read_and_delete_session_notifications(db_pool, session_id)?;
    Ok(notifs)
}

pub fn session_with_specified_id_exists(
    session_id: String,
    db_pool: &db_manager::DbPool,
) -> Result<bool, DbError> {
    let result = get_session_by_session_id(session_id.clone(), db_pool)?;
    if result.is_some() {
        return Ok(true);
    }
    return Ok(false);
}

pub fn get_session_by_session_id(
    session_id: String,
    db_pool: &db_manager::DbPool,
) -> Result<Option<Session>, DbError> {
    let sess = storage_specific_get_session_by_session_id(db_pool, session_id)?;
    return Ok(sess);
}

pub fn generate_session_id() -> String {
    generate_id()
}

pub fn create_new_session_with_id(
    new_id: String,
    db_pool: &db_manager::DbPool,
) -> Result<Session, DbError> {
    let new_session = Session {
        id: new_id.clone(),
        user_public_inapp_id: String::new(),
        logged_in: false,
    };

    storage_specific_add_session(new_session.clone(), db_pool)?;
    Ok(new_session)
}

// separation between general functions above and storage specific stuff (txt vs mysql)

fn storage_specific_add_session(
    session: Session,
    db_pool: &db_manager::DbPool,
) -> Result<db_manager::DbSuccess, db_manager::DbError> {
    let client = db_manager::get_client(db_pool.clone())?;
    db_manager::execute(
        client,
        "INSERT INTO public.sessions (id, user_id, logged_in) VALUES ($1, $2, $3)",
        &[
            &session.id,
            &session.user_public_inapp_id,
            &session.logged_in,
        ],
    )?;

    Ok(db_manager::DbSuccess::new("Ok"))
}

fn storage_specific_get_session_by_session_id(
    db_pool: &db_manager::DbPool,
    session_id: String,
) -> Result<Option<Session>, db_manager::DbError> {
    let client = db_manager::get_client(db_pool.clone())?;
    let retrieved = db_manager::query_opt(
        client,
        "SELECT id, user_id, logged_in FROM sessions WHERE id = $1",
        &[&session_id],
    )?;

    match retrieved {
        Some(r) => {
            let retrieved_session = Session {
                id: r.get("id"),
                user_public_inapp_id: r.get("user_id"),
                logged_in: r.get("logged_in"),
            };
            return Ok(Some(retrieved_session));
        }
        None => {
            return Ok(None);
        }
    }
}

fn storage_specific_read_and_delete_session_notifications(
    db_pool: &db_manager::DbPool,
    session_id: String,
) -> Result<Vec<SessionNotification>, DbError> {
    let mut notifications: Vec<SessionNotification> = vec![];
    let mut client = db_manager::get_client(db_pool.clone())?;
    let mut transaction = match client.transaction() {
        Ok(tx) => tx,
        Err(e) => return Err(DbError::new(e)),
    };
    let rows = match transaction.query(
        "SELECT session_id, message, notification_type
  FROM public.session_notifications WHERE session_id = $1",
        &[&session_id],
    ) {
        Ok(r) => r,
        Err(e) => return Err(DbError::new(e)),
    };
    if rows.len() == 0 {
        return Ok(notifications);
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

    match transaction.execute(
        "DELETE FROM  public.session_notifications WHERE session_id = $1 ",
        &[&session_id],
    ) {
        Err(e) => return Err(DbError::new(e)),
        Ok(_) => {}
    }
    match transaction.commit() {
        Err(e) => return Err(DbError::new(e)),
        Ok(_) => {}
    }

    return Ok(notifications);
}

fn storage_specific_edit_session(
    db_pool: &db_manager::DbPool,
    session_id: String,
    modified_session: Session,
) -> Result<DbSuccess, DbError> {
    let client = db_manager::get_client(db_pool.clone())?;
    let query = "UPDATE public.sessions SET user_id = $1, logged_in = $2, id = $3 WHERE id = $4";

    let rows_modified = db_manager::execute(
        client,
        query,
        &[
            &modified_session.user_public_inapp_id,
            &modified_session.logged_in,
            &modified_session.id,
            &session_id,
        ],
    )?;
    if rows_modified != 0 {
        return Ok(DbSuccess {
            message: format!("Success"),
        });
    }
    return Err(DbError::new("No rows modified"));
}

fn storage_specific_add_session_notification(
    db_pool: &db_manager::DbPool,
    session_notification: SessionNotification,
) -> Result<DbSuccess, DbError> {
    let notification_type = session_notification.notification_type.to_string();

    let client = db_manager::get_client(db_pool.clone())?;
    let query = "INSERT INTO public.session_notifications (session_id, message, notification_type) VALUES ($1, $2, $3)";
    let _ = db_manager::execute(
        client,
        query,
        &[
            &session_notification.session_id,
            &session_notification.message,
            &notification_type,
        ],
    )?;
    return Ok(DbSuccess::new("okay"));
}
