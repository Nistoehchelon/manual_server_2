use db_manager::{DbError, DbSuccess};
use id_generator::generate_id;
use session_structs_and_so_on::*;
use std::collections::HashMap;

pub fn get_name_of_cookie_containing_session_id() -> String {
    NAME_OF_COOKIE_CONTAINING_SESSION_ID.to_string()
}

pub async fn edit_session(
    db_pool: sqlx::PgPool,
    session_id: String,
    modified_session: Session,
) -> Result<DbSuccess, DbError> {
    let result = storage_specific_edit_session(db_pool, session_id, modified_session).await;
    return result;
}

pub async fn add_session_notification(
    db_pool: sqlx::PgPool,
    session_notification: SessionNotification,
) -> Result<(), DbError> {
    match storage_specific_add_session_notification(db_pool, session_notification).await {
        Ok(_) => Ok(()),
        Err(e) => {
            dbg!("Could not create session notification", &e);
            return Err(e);
        }
    }
}

pub async fn read_and_delete_session_notifications(
    db_pool: sqlx::PgPool,
    session_id: String,
) -> Result<Vec<SessionNotification>, DbError> {
    let notifs =
        storage_specific_read_and_delete_session_notifications(db_pool, session_id).await?;
    Ok(notifs)
}

pub async fn session_with_specified_id_exists(
    session_id: String,
    db_pool: sqlx::PgPool,
) -> Result<bool, DbError> {
    let result = find_session_by_session_id(session_id.clone(), db_pool).await?;
    if result.is_some() {
        return Ok(true);
    }
    return Ok(false);
}

pub async fn find_session_by_session_id(
    session_id: String,
    db_pool: sqlx::PgPool,
) -> Result<Option<Session>, DbError> {
    let sess = storage_specific_get_session_by_session_id(db_pool, session_id).await?;
    return Ok(sess);
}
pub async fn get_session_by_session_id(
    session_id: String,
    db_pool: sqlx::PgPool,
) -> Result<Result<Session, error_structs::ExpectedToGetButFailed>, DbError> {
    let sess_opt = find_session_by_session_id(session_id, db_pool).await?;
    if let Some(sess) = sess_opt {
        return Ok(Ok(sess));
    }
    return Ok(Err(error_structs::ExpectedToGetButFailed::new("")));
}

pub fn generate_session_id() -> String {
    generate_id()
}

pub async fn create_new_session_with_id(
    new_id: String,
    db_pool: sqlx::PgPool,
) -> Result<Session, DbError> {
    let new_session = Session {
        id: new_id.clone(),
        user_id: String::new(),
        logged_in: false,
    };

    storage_specific_add_session(new_session.clone(), db_pool).await?;
    Ok(new_session)
}

pub async fn get_existing_or_make_new_session(
    db_pool: sqlx::PgPool,
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
                    db_pool.clone(),
                )
                .await
                {
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

        let session_exists = match session_with_specified_id_exists(session_id.clone(), db_pool.clone())
            .await
        {
            Ok(b) => b,
            Err(_) => {
                return Err(format!(
                    "Checking session id to decide whether we will create it or reuse if exists: Could not check whether session was created due to DB Error"
                ));
            }
        };
        if session_exists {
            match find_session_by_session_id(session_id.clone(), db_pool.clone()).await {
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
    db_pool: sqlx::PgPool,
) -> Result<db_manager::DbSuccess, db_manager::DbError> {
    match sqlx::query!(
        r#"INSERT INTO public.sessions (id, user_id, logged_in) VALUES ($1, $2, $3)"#,
        session.id,
        session.user_id,
        session.logged_in
    )
    .execute(&db_pool)
    .await
    {
        Ok(_) => {
            return Ok(db_manager::DbSuccess::new("Ok"));
        }
        Err(e) => {
            return Err(db_manager::DbError::new(e));
        }
    }
}

async fn storage_specific_get_session_by_session_id(
    db_pool: sqlx::PgPool,
    session_id: String,
) -> Result<Option<Session>, db_manager::DbError> {
    let retrieved = match sqlx::query_as!(
        Session,
        r#"SELECT id, user_id as user_id, logged_in FROM sessions WHERE id = $1"#,
        session_id
    )
    .fetch_optional(&db_pool)
    .await
    {
        Ok(opt) => opt,
        Err(e) => {
            return Err(DbError::new(e));
        }
    };

    return Ok(retrieved);
}

async fn storage_specific_read_and_delete_session_notifications(
    db_pool: sqlx::PgPool,
    session_id: String,
) -> Result<Vec<SessionNotification>, DbError> {
    let mut notifications: Vec<SessionNotification> = vec![];
    let mut transaction = match db_pool.begin().await {
        Ok(tx) => tx,
        Err(e) => return Err(DbError::new(e)),
    };
    let rows = match sqlx::query!(
        r#"SELECT session_id, message, notification_type
  FROM public.session_notifications WHERE session_id = $1"#,
        session_id
    )
    .fetch_all(&mut *transaction)
    .await
    {
        Ok(r) => r,
        Err(e) => return Err(DbError::new(e)),
    };
    if rows.len() == 0 {
        return Ok(notifications);
    }
    for row in rows.iter() {
        let notif_type: SessionNotificationType = match row.notification_type.as_str() {
            "information" => SessionNotificationType::Information,
            "error" => SessionNotificationType::Error,
            "success" => SessionNotificationType::Success,
            val => panic!("Invalid {val} value in notification_type in DB"),
        };
        notifications.push(SessionNotification {
            notification_type: notif_type,
            message: row.message.clone(),
            session_id: row.session_id.clone(),
        });
    }

    match sqlx::query!(
        "DELETE FROM  public.session_notifications WHERE session_id = $1 ",
        session_id,
    )
    .execute(&mut *transaction)
    .await
    {
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
    db_pool: sqlx::PgPool,
    session_id: String,
    modified_session: Session,
) -> Result<DbSuccess, DbError> {
    let rows_modified = match sqlx::query!(
        r#"UPDATE public.sessions SET user_id = $1, logged_in = $2, id = $3 WHERE id = $4"#,
        modified_session.user_id,
        modified_session.logged_in,
        modified_session.id,
        session_id,
    )
    .execute(&db_pool)
    .await
    {
        Ok(rm) => rm.rows_affected(),
        Err(e) => {
            return Err(DbError::new(e));
        }
    };

    if rows_modified != 0 {
        return Ok(DbSuccess {
            message: format!("Success"),
        });
    }
    return Err(DbError::new("No rows modified"));
}

async fn storage_specific_add_session_notification(
    db_pool: sqlx::PgPool,
    session_notification: SessionNotification,
) -> Result<DbSuccess, DbError> {
    let notification_type = session_notification.notification_type.to_string();

    match sqlx::query!(r#"INSERT INTO public.session_notifications (session_id, message, notification_type) VALUES ($1, $2, $3)"#, session_notification.session_id,
            session_notification.message,
            notification_type,
    ).execute(&db_pool).await {
        Err(e) => {
            dbg!("Could not add session notification");
            return Err(DbError::new(e));
        },
        Ok(_) =>{}
    };
    return Ok(DbSuccess::new("okay"));
}
