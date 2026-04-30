use error_structs::SomeSortOfError;
use session::get_session_by_session_id;
use session_structs_and_so_on::Session;
use user::get_user_by_user_id;

#[derive(Debug, Clone)]
pub struct UserCanViewAppExecutableVersionInput {
    pub db_pool: sqlx::PgPool,
    pub user_id: String,
}

pub async fn get_bool_subject_subject_id_permission_name(
    db_pool: sqlx::PgPool,
    subject: &str,
    subject_id: &str,
    permission_name: &str,
) -> Result<bool, SomeSortOfError> {
    let check = sqlx::query!(
        "SELECT name FROM access_attributes WHERE subject = $1 AND subject_id = $2 AND name = $3",
        subject,
        subject_id,
        permission_name
    )
    .fetch_optional(&db_pool)
    .await
    .map_err(SomeSortOfError::new)?
    .is_some();
    Ok(check)
}

pub async fn user_can_view_app_executable_version(
    input: UserCanViewAppExecutableVersionInput,
) -> Result<bool, SomeSortOfError> {
    let b = get_bool_subject_subject_id_permission_name(
        input.db_pool.clone(),
        "user",
        input.user_id.as_str(),
        "view_app_executable_version",
    )
    .await
    .map_err(SomeSortOfError::new)?;
    Ok(b)
}

pub async fn add_admin_permissions_to_user(db_pool: sqlx::PgPool, user_id: String) -> () {
    let perms = vec![
        "set_custom_thread_id_when_creating_it",
        "manage_custom_site_pages",
        "create_forum",
        "view_app_executable_version",
    ];
    for perm in perms {
        let perm = perm.to_string();
        // I don't think I really care to handle the result since this is to be used just for startup the admin user for convenience
        let _ = sqlx::query!(
            "INSERT INTO access_attributes (name, subject, subject_id) VALUES ($1, $2, $3)",
            perm,
            "user",
            user_id.clone()
        )
        .execute(&db_pool)
        .await;
    }
}

pub struct CanAccessThisForumThreadInput {
    pub db_pool: sqlx::PgPool,
    pub user_id: String,
    pub thread_id: String,
}
pub async fn can_access_this_forum_thread(
    CanAccessThisForumThreadInput {
        db_pool,
        user_id,
        thread_id,
    }: CanAccessThisForumThreadInput,
) -> Result<bool, SomeSortOfError> {
    let thread = forum::get_thread(db_pool.clone(), thread_id.clone())
        .await
        .map_err(SomeSortOfError::new)?;
    can_access_this_forum(CanAccessThisForumInput {
        db_pool,
        user_id,
        forum_id: thread.forum_id,
    })
    .await
}
pub struct CanAccessThisForumInput {
    pub db_pool: sqlx::PgPool,
    pub user_id: String,
    pub forum_id: String,
}
pub async fn can_access_this_forum(
    CanAccessThisForumInput {
        db_pool,
        user_id,
        forum_id,
    }: CanAccessThisForumInput,
) -> Result<bool, SomeSortOfError> {
    let forum = forum::get_forum_by_id(db_pool.clone(), forum_id.clone())
        .await
        .map_err(SomeSortOfError::new)?;
    match forum.id.as_str() {
        id if id == custom_site_pages::FORUM_ID => can_manage_site_pages(CanManageSitePagesInput {
            db_pool: db_pool.clone(),
            user_id: user_id.clone(),
        })
        .await
        .map_err(SomeSortOfError::new),
        _ => Ok(true),
    }
}

pub struct CanEditForumThreadMessageInput {
    pub user_id: String,
    pub original_message_poster_id: String,
}
pub async fn can_edit_forum_thread_message(
    input: CanEditForumThreadMessageInput,
) -> Result<bool, SomeSortOfError> {
    // todo... at least check if user and msg's OP are the same. Actually, let's do that now.
    Ok(input.user_id == input.original_message_poster_id)
}

pub struct CanSetCustomThreadIdWhenCreatingItInput {
    pub db_pool: sqlx::PgPool,
    pub session: Session,
}

pub async fn can_set_custom_thread_id_when_creating_it(
    CanSetCustomThreadIdWhenCreatingItInput { db_pool, session }: CanSetCustomThreadIdWhenCreatingItInput,
) -> Result<bool, SomeSortOfError> {
    let session = match get_session_by_session_id(session.id.clone(), db_pool.clone())
        .await
        .map_err(SomeSortOfError::new)?
    {
        Ok(s) => s,
        Err(_) => return Ok(false),
    };
    let user = match get_user_by_user_id(db_pool.clone(), session.user_id.clone())
        .await
        .map_err(SomeSortOfError::new)
    {
        Ok(u) => u,
        Err(_) => {
            return Ok(false);
        }
    };
    let user_id = user.id.clone();
    let b = get_bool_subject_subject_id_permission_name(
        db_pool.clone(),
        "user",
        user_id.as_str(),
        "set_custom_thread_id_when_creating_it",
    )
    .await?;
    Ok(b)
}

pub async fn can_create_forum_thread_message(session: Session) -> Result<bool, SomeSortOfError> {
    Ok(session.logged_in)
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
        .map_err(SomeSortOfError::new)
    {
        Ok(u) => u,
        Err(_) => {
            return Ok(false);
        }
    };
    let user_id = user.id.clone();

    let is_superuser: bool = get_bool_subject_subject_id_permission_name(
        db_pool.clone(),
        "user",
        user_id.as_str(),
        "superuser",
    )
    .await?;
    if is_superuser {
        return Ok(true);
    }

    let b = get_bool_subject_subject_id_permission_name(
        db_pool.clone(),
        "user",
        user_id.as_str(),
        "create_forum",
    )
    .await?;
    Ok(b)
}

pub struct CanCreateThreadInput {
    pub db_pool: sqlx::PgPool,
    pub session_id: String,
}
pub struct CanManageSitePagesInput {
    pub db_pool: sqlx::PgPool,
    pub user_id: String,
}
pub async fn can_create_forum_thread(
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
        .map_err(SomeSortOfError::new)
    {
        Ok(u) => u,
        Err(_) => {
            return Ok(false);
        }
    };
    let user_id = user.id.clone();

    let is_superuser: bool = get_bool_subject_subject_id_permission_name(
        db_pool.clone(),
        "user",
        user_id.as_str(),
        "superuser",
    )
    .await?;
    if is_superuser {
        return Ok(true);
    }

    let b = get_bool_subject_subject_id_permission_name(
        db_pool.clone(),
        "user",
        user_id.as_str(),
        "create_thread",
    )
    .await?;
    Ok(b)
}

pub async fn can_manage_site_pages(
    CanManageSitePagesInput { db_pool, user_id }: CanManageSitePagesInput,
) -> Result<bool, error_structs::SomeSortOfError> {
    let user = match get_user_by_user_id(db_pool.clone(), user_id.clone())
        .await
        .map_err(SomeSortOfError::new)
    {
        Ok(u) => u,
        Err(_) => {
            return Ok(false);
        }
    };
    let user_id = user.id.clone();
    let b = get_bool_subject_subject_id_permission_name(
        db_pool.clone(),
        "user",
        user_id.as_str(),
        "manage_custom_site_pages",
    )
    .await?;
    Ok(b)
}
