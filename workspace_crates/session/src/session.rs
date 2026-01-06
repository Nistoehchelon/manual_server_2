use std::collections::HashMap;
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
    pub public_id: String,
    pub user_public_id: String,
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

pub async fn edit_session(db_pool: &db_manager::DbPool, session_id: String, modified_session: Session) -> Result<DbSuccess, DbError> {
    let result = storage_specific_edit_session(db_pool, session_id, modified_session).await;
    return result;
}

pub async fn add_session_notification(
    db_pool: &db_manager::DbPool,
    session_notification: SessionNotification,
) -> Result<bool, DbError> {
    match storage_specific_add_session_notification(db_pool, session_notification).await {
        Ok(_) => Ok(true),
        _ => Err(DbError {
            message: format!(":("),
        }),
    }
}

pub async fn read_and_delete_session_notifications(
    db_pool: &db_manager::DbPool,
    session_id: String,
) -> Result<Vec<SessionNotification>, DbError> {
    let notifs = storage_specific_read_and_delete_session_notifications(db_pool, session_id).await?;
    Ok(notifs)
}

pub async fn session_with_specified_id_exists(
    session_id: String,
    db_pool: &db_manager::DbPool,
) -> Result<bool, DbError> {
    let result = get_session_by_session_id(session_id.clone(), db_pool).await?;
    if result.is_some() {
        return Ok(true);
    }
    return Ok(false);
}

pub async fn get_session_by_session_id(
    session_id: String,
    db_pool: &db_manager::DbPool,
) -> Result<Option<Session>, DbError> {
    let sess = storage_specific_get_session_by_session_id(db_pool, session_id).await?;
    return Ok(sess);
}

pub fn generate_session_id() -> String {
    generate_id()
}

pub async fn create_new_session_with_id(
    new_id: String,
    db_pool: &db_manager::DbPool,
) -> Result<Session, DbError> {
    let new_session = Session {
        public_id: new_id.clone(),
        user_public_id: String::new(),
        logged_in: false,
    };

    storage_specific_add_session(new_session.clone(), db_pool).await?;
    Ok(new_session)
}

pub async fn get_existing_or_make_new_session(
    db_pool: &db_manager::DbPool,
    cookies: HashMap<String, String>,
) -> Result<(Session, bool), String> {
    let mut created_new_session = false;
    let session: Session = {
        let session_id: String = match cookies
            .get(get_name_of_cookie_containing_session_id().as_str())
        {
            Some(session_id) => {
                let session_exists = match session_with_specified_id_exists(
                    session_id.clone(),
                    &db_pool,
                ).await {
                    Ok(answer) => answer,
                    Err(_) => {
                        return Err(format!(
                            "Initial id from cookie: Could not check whether session with such id exists due to DB error"
                        ));
                    }
                };
                if !session_exists {
                    created_new_session = true;
                    generate_session_id()
                } else {
                    session_id.to_string()
                }
            }
            None => {
                created_new_session = true;
                generate_session_id()
            }
        };

        let session_exists = match session_with_specified_id_exists(session_id.clone(), db_pool).await {
            Ok(b) => b,
            Err(_) => {
                return Err(format!(
                    "Checking session id to decide whether we will create it or reuse if exists: Could not check whether session was created due to DB Error"
                ));
            }
        };
        if session_exists {
            match get_session_by_session_id(session_id.clone(), db_pool).await {
                Ok(Some(sess)) => sess,
                _ => {
                    return Err(format!(
                        "Could not get existing session by id due to DB error"
                    ));
                }
            }
        } else {
            match create_new_session_with_id(session_id.clone(), db_pool).await {
                Ok(sess) => sess,
                Err(_) => return Err(format!("Could not create a session due to DB error")),
            }
        }
    };
    return Ok((session, created_new_session));
}

// separation between general functions above and storage specific stuff

async fn storage_specific_add_session(
    session: Session,
    db_pool: &db_manager::DbPool,
) -> Result<db_manager::DbSuccess, db_manager::DbError> {
    let client = db_manager::get_client(&db_pool).await?;
    db_manager::execute(
        &client,
        "INSERT INTO public.sessions (public_id, user_id, logged_in) VALUES ($1, $2, $3)",
        &[
            &session.public_id,
            &session.user_public_id,
            &session.logged_in,
        ],
    ).await?;

    Ok(db_manager::DbSuccess::new("Ok"))
}

async fn storage_specific_get_session_by_session_id(
    db_pool: &db_manager::DbPool,
    session_id: String,
) -> Result<Option<Session>, db_manager::DbError> {
    let client = db_manager::get_client(&db_pool).await?;
    let retrieved = db_manager::query_opt(
        &client,
        "SELECT public_id, user_id, logged_in FROM sessions WHERE public_id = $1",
        &[&session_id],
    ).await?;

    match retrieved {
        Some(r) => {
            let retrieved_session = Session {
                public_id: r.get("public_id"),
                user_public_id: r.get("user_id"),
                logged_in: r.get("logged_in"),
            };
            return Ok(Some(retrieved_session));
        }
        None => {
            return Ok(None);
        }
    }
}

async fn storage_specific_read_and_delete_session_notifications(
    db_pool: &db_manager::DbPool,
    session_id: String,
) -> Result<Vec<SessionNotification>, DbError> {
    let mut notifications: Vec<SessionNotification> = vec![];
    let mut client = db_manager::get_client(&db_pool).await?;
    let transaction = match client.transaction().await {
        Ok(tx) => tx,
        Err(e) => return Err(DbError::new(e)),
    };
    let rows = match transaction.query(
        "SELECT session_id, message, notification_type
  FROM public.session_notifications WHERE session_id = $1",
        &[&session_id],
    ).await {
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
    ).await {
        Err(e) => return Err(DbError::new(e)),
        Ok(_) => {}
    }
    match transaction.commit().await {
        Err(e) => return Err(DbError::new(e)),
        Ok(_) => {}
    }

    return Ok(notifications);
}

async fn storage_specific_edit_session(
    db_pool: &db_manager::DbPool,
    session_id: String,
    modified_session: Session,
) -> Result<DbSuccess, DbError> {
    let client = db_manager::get_client(&db_pool).await?;
    let query = "UPDATE public.sessions SET user_id = $1, logged_in = $2, public_id = $3 WHERE public_id = $4";

    let rows_modified = db_manager::execute(
        &client,
        query,
        &[
            &modified_session.user_public_id,
            &modified_session.logged_in,
            &modified_session.public_id,
            &session_id,
        ],
    ).await?;
    if rows_modified != 0 {
        return Ok(DbSuccess {
            message: format!("Success"),
        });
    }
    return Err(DbError::new("No rows modified"));
}

async fn storage_specific_add_session_notification(
    db_pool: &db_manager::DbPool,
    session_notification: SessionNotification,
) -> Result<DbSuccess, DbError> {
    let notification_type = session_notification.notification_type.to_string();

    let client = db_manager::get_client(&db_pool).await?;
    let query = "INSERT INTO public.session_notifications (session_id, message, notification_type) VALUES ($1, $2, $3)";
    let _ = db_manager::execute(
        &client,
        query,
        &[
            &session_notification.session_id,
            &session_notification.message,
            &notification_type,
        ],
    ).await?;
    return Ok(DbSuccess::new("okay"));
}
