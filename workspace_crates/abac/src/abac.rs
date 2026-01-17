use error_structs::SomeSortOfError;
use session::get_session_by_session_id;
use session_structs_and_so_on::Session;
use user::get_user_by_user_id;

pub async fn can_add_forum_thread_message(session: Session) -> Result<bool, SomeSortOfError> {
  return Ok(session.logged_in);
}


#[derive(Debug)]
pub struct CanCreateForumInput {
    pub db_pool: sqlx::PgPool,
    pub session_id: String,
}


pub async fn can_create_forum(
    CanCreateForumInput {
        db_pool,
        session_id,
    }: CanCreateForumInput,
) -> Result<bool, SomeSortOfError> {
    let session = match get_session_by_session_id(session_id, db_pool.clone())
        .await
        .map_err(SomeSortOfError::new)?
    {
        Ok(s) => s,
        Err(_) => return Ok(false),
    };
    let user = match get_user_by_user_id(db_pool.clone(), session.user_id.clone())
        .await
        .map_err(SomeSortOfError::new)?
    {
        Ok(u) => u,
        Err(_) => {
            return Ok(false);
        }
    };
    let user_id = user.id.clone();

    let is_superuser: bool = match sqlx::query!(
        "SELECT name FROM access_attributes WHERE subject = $1 AND subject_id = $2 AND name = $3",
        "user",
        user_id,
        "superuser"
    )
    .fetch_optional(&db_pool)
    .await
    {
        Ok(opt) => match opt {
            Some(_) => true,
            None => false,
        },
        Err(e) => return Err(SomeSortOfError::new(e)),
    };
    if is_superuser {
        return Ok(true);
    }

    let b = match sqlx::query!(
        "SELECT name FROM access_attributes WHERE subject = $1 AND subject_id = $2 AND name = $3",
        "user",
        user_id,
        "create_forum"
    )
    .fetch_optional(&db_pool)
    .await
    .map_err(SomeSortOfError::new)?
    {
        Some(_) => true,
        None => false,
    };
    return Ok(b);
}

pub struct CanCreateThreadInput {
    pub db_pool: sqlx::PgPool,
    pub session_id: String,
}
pub async fn can_create_thread(
    CanCreateThreadInput {
        db_pool,
        session_id,
    }: CanCreateThreadInput,
) -> Result<bool, error_structs::SomeSortOfError> {
    let session = match get_session_by_session_id(session_id, db_pool.clone())
        .await
        .map_err(SomeSortOfError::new)?
    {
        Ok(s) => s,
        Err(_) => return Ok(false),
    };
    let user = match get_user_by_user_id(db_pool.clone(), session.user_id.clone())
        .await
        .map_err(SomeSortOfError::new)?
    {
        Ok(u) => u,
        Err(_) => {
            return Ok(false);
        }
    };
    let user_id = user.id.clone();

    let is_superuser: bool = match sqlx::query!(
        "SELECT name FROM access_attributes WHERE subject = $1 AND subject_id = $2 AND name = $3",
        "user",
        user_id,
        "superuser"
    )
    .fetch_optional(&db_pool)
    .await
    {
        Ok(opt) => match opt {
            Some(_) => true,
            None => false,
        },
        Err(e) => return Err(SomeSortOfError::new(e)),
    };
    if is_superuser {
        return Ok(true);
    }

    let b = match sqlx::query!(
        "SELECT name FROM access_attributes WHERE subject = $1 AND subject_id = $2 AND name = $3",
        "user",
        user_id,
        "create_thread"
    )
    .fetch_optional(&db_pool)
    .await
    .map_err(SomeSortOfError::new)?
    {
        Some(_) => true,
        None => false,
    };
    return Ok(b);
}
