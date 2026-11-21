use session::get_session_by_session_id;
use session_structs_and_so_on::Session;
use user::get_user_by_user_id;

use crate::permissions::Permission::{self, CanEditSiteVariables, CanViewSiteVariables};

pub async fn user_can_manage_custom_site_variables(
    db_pool: sqlx::PgPool,
    user_id: String,
) -> anyhow::Result<bool> {
    Ok(get_bool_subject_subject_id_permission_name(
        db_pool.clone(),
        "user",
        &user_id,
        CanEditSiteVariables,
    )
    .await?
        && get_bool_subject_subject_id_permission_name(
            db_pool.clone(),
            "user",
            &user_id,
            CanViewSiteVariables,
        )
        .await?)
}

pub async fn user_can_view_invoice(
    db_pool: sqlx::PgPool,
    user_id: String,
    invoice_id: String,
) -> anyhow::Result<bool> {
    let mut tx = db_pool.begin().await?;
    let Ok(invoice_user_id) =
        invoice::get_invoice_data(&mut tx, invoice::InvoiceId(invoice_id), "user_id").await
    else {
        return Ok(false);
    };
    tx.commit().await?;
    Ok(user_id == invoice_user_id)
}
pub async fn user_can_create_invoice(
    _db_pool: sqlx::PgPool,
    _user_id: String,
) -> anyhow::Result<bool> {
    Ok(true)
}

#[derive(Debug, Clone)]
pub struct UserCanViewAppExecutableVersionInput {
    pub db_pool: sqlx::PgPool,
    pub user_id: String,
}

pub async fn get_bool_subject_subject_id_permission_name(
    db_pool: sqlx::PgPool,
    subject: &str,
    subject_id: &str,
    permission: Permission,
) -> anyhow::Result<bool> {
    let check = sqlx::query!(
        "SELECT name FROM access_attributes WHERE subject = $1 AND subject_id = $2 AND name = $3",
        subject,
        subject_id,
        permission.to_string(),
    )
    .fetch_optional(&db_pool)
    .await?
    .is_some();
    Ok(check)
}

pub async fn user_can_view_app_executable_version(
    input: UserCanViewAppExecutableVersionInput,
) -> anyhow::Result<bool> {
    let b = get_bool_subject_subject_id_permission_name(
        input.db_pool.clone(),
        "user",
        input.user_id.as_str(),
        Permission::ViewAppExecutableVersion,
    )
    .await?;
    Ok(b)
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
) -> anyhow::Result<bool> {
    let thread = forum::get_thread(db_pool.clone(), thread_id.clone()).await?;
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
) -> anyhow::Result<bool> {
    let forum = forum::get_forum_by_id(db_pool.clone(), forum_id.clone()).await?;
    match forum.id.as_str() {
        id if id == custom_site_pages::FORUM_ID
            || id == forum::NONE_ID
            || id == dynamic_stylesheets::FORUM_ID =>
        {
            can_manage_site_pages(CanManageSitePagesInput {
                db_pool: db_pool.clone(),
                user_id: user_id.clone(),
            })
            .await
        }
        _ => Ok(true),
    }
}

pub struct CanEditForumThreadMessageInput {
    pub user_id: String,
    pub original_message_poster_id: String,
}
pub async fn can_edit_forum_thread_message(
    input: CanEditForumThreadMessageInput,
) -> anyhow::Result<bool> {
    // todo... at least check if user and msg's OP are the same. Actually, let's do that now.
    Ok(input.user_id == input.original_message_poster_id)
}

pub struct CanSetCustomThreadIdWhenCreatingItInput {
    pub db_pool: sqlx::PgPool,
    pub session: Session,
}

pub async fn can_set_custom_thread_id_when_creating_it(
    CanSetCustomThreadIdWhenCreatingItInput { db_pool, session }: CanSetCustomThreadIdWhenCreatingItInput,
) -> anyhow::Result<bool> {
    let session = get_session_by_session_id(session.id.clone(), db_pool.clone()).await?;
    let user = match get_user_by_user_id(db_pool.clone(), session.user_id.clone()).await {
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
        Permission::SetCustomThreadIdWhenCreatingIt,
    )
    .await?;
    Ok(b)
}

pub async fn can_create_forum_thread_message(session: Session) -> anyhow::Result<bool> {
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
) -> anyhow::Result<bool> {
    let session = get_session_by_session_id(session_id, db_pool.clone()).await?;
    let user = match get_user_by_user_id(db_pool.clone(), session.user_id.clone()).await {
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
        Permission::Superuser,
    )
    .await?;
    if is_superuser {
        return Ok(true);
    }

    let b = get_bool_subject_subject_id_permission_name(
        db_pool.clone(),
        "user",
        user_id.as_str(),
        Permission::CanCreateForum,
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
) -> anyhow::Result<bool> {
    let session = get_session_by_session_id(session_id, db_pool.clone()).await?;
    let user = match get_user_by_user_id(db_pool.clone(), session.user_id.clone()).await {
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
        Permission::Superuser,
    )
    .await?;
    if is_superuser {
        return Ok(true);
    }

    let b = get_bool_subject_subject_id_permission_name(
        db_pool.clone(),
        "user",
        user_id.as_str(),
        Permission::CanCreateThread,
    )
    .await?;
    Ok(b)
}

pub async fn can_manage_site_pages(
    CanManageSitePagesInput { db_pool, user_id }: CanManageSitePagesInput,
) -> anyhow::Result<bool> {
    let user = match get_user_by_user_id(db_pool.clone(), user_id.clone()).await {
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
        Permission::ManageCustomSitePages,
    )
    .await?;
    Ok(b)
}

pub async fn can_edit_site_variables(
    db_pool: &sqlx::PgPool,
    user_id: String,
) -> anyhow::Result<bool> {
    let b = get_bool_subject_subject_id_permission_name(
        db_pool.clone(),
        "user",
        user_id.as_str(),
        Permission::CanEditSiteVariables,
    )
    .await?;
    Ok(b)
}

pub async fn can_view_site_variables(
    db_pool: &sqlx::PgPool,
    user_id: String,
) -> anyhow::Result<bool> {
    let b = get_bool_subject_subject_id_permission_name(
        db_pool.clone(),
        "user",
        user_id.as_str(),
        Permission::CanViewSiteVariables,
    )
    .await?;
    Ok(b)
}
