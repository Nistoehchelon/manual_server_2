use crate::{
    DEFAULT_FORUM_ID, ForumThreadMessage, NONE_ID, ValidForumThreadMessageContent,
    ValidForumThreadName,
    forum_structs::{Forum, ForumThread, ForumThreadMessageVersion},
};
use error_structs::ExpectedToGetButFailed;

pub enum ForumIdSource {
    Auto,
    Custom(UniqueForumId),
}
pub struct UniqueForumId {
    inner: String,
}
impl UniqueForumId {
    pub async fn parse(
        db_pool: sqlx::PgPool,
        id: String,
    ) -> anyhow::Result<UniqueForumId> {
        let forum = find_forum_by_id(db_pool.clone(), id.clone())
            .await
           ?;
        if let Some(forum) = forum {
            return Err(anyhow::anyhow!(
                "Could not parse String into UniqueForumId: the provided id is not unique, a forum with id  \"{}\" already exists",
                forum.id
            ));
        }
        Ok(UniqueForumId { inner: id })
    }
    pub fn get(self) -> String {
        self.inner.clone()
    }
}
pub async fn create_forum(
    db_pool: sqlx::PgPool,
    parent_forum_id: String,
    id_source: ForumIdSource,
    name: String,
) -> anyhow::Result<String> {
    let mut tx = db_pool.begin().await?;
    let id = match id_source {
        ForumIdSource::Auto => id_generator_unique_in_db::generate_unique_forum_id(&mut tx)
            .await
            ?,
        ForumIdSource::Custom(unique_id) => unique_id.get(),
    };

    sqlx::query!(
        "INSERT INTO forums (id, parent_id, name) VALUES ($1, $2, $3)",
        id,
        parent_forum_id,
        name
    )
    .execute(&mut *tx)
    .await
    ?;
    tx.commit().await?;

    Ok(id)
}
pub async fn find_forum_by_id(db_pool: sqlx::PgPool, id: String) -> anyhow::Result<Option<Forum>> {
    if id == DEFAULT_FORUM_ID {
        return Ok(Some(Forum {
            id: "0".to_string(),
            name: "Default forum".to_string(),
            parent_id: "0".to_string(),
        }));
    }
    if id == NONE_ID {
        return Ok(Some(Forum {
            id: "none".to_string(),
            name: "None".to_string(),
            parent_id: "none".to_string(),
        }));
    }
    storage_specific_find_forum_by_id(db_pool, id.clone()).await
}
pub async fn get_forum_by_id(db_pool: sqlx::PgPool, id: String) -> anyhow::Result<Forum> {
    let f = find_forum_by_id(db_pool, id.clone()).await?;
    match f {
        Some(f) => Ok(f),
        None => Err(anyhow::anyhow!(format!("Forum \"{id}\" not found"))),
    }
}

pub async fn get_count_thread_pages(
    db_pool: sqlx::PgPool,
    thread_id: String,
    messages_per_page: u64,
) -> anyhow::Result<u64> {
    let msgs_count = get_count_thread_messages(db_pool.clone(), thread_id.clone()).await?;
    let pages_count = msgs_count.div_ceil(messages_per_page);
    Ok(pages_count)
}

// let message_versions = forum::get_thread_message_versions(db_pool.clone, message_id.clone(), page.clone(), VERSIONS_PER_PAGE).await.map_err(InternalAppError::new)?;

pub async fn get_thread_message_versions(
    db_pool: sqlx::PgPool,
    message_id: String,
    page: u64,
    records_per_page: u64,
) -> anyhow::Result<Vec<ForumThreadMessageVersion>> {
    let limit = i64::try_from(records_per_page)?;
    let page = i64::try_from(page)?;
    let offset = (page - 1) * limit;
    let msg_vers = sqlx::query!("SELECT primary_key, id, user_id, message_id, content, created_at FROM forum_thread_message_versions WHERE message_id = $1 ORDER BY primary_key DESC LIMIT $2 OFFSET $3", message_id, limit, offset).fetch_all(&db_pool).await?;
    let msg_vers = {
        let mut converted: Vec<ForumThreadMessageVersion> = vec![];
        for r in msg_vers {
            converted.push(ForumThreadMessageVersion {
                primary_key: r.primary_key,
                id: r.id,
                message_id: r.message_id,
                user_id: r.user_id,
                content: r.content,
                created_at: r.created_at,
            });
        }
        converted
    };

    Ok(msg_vers)
}

pub async fn get_count_thread_message_versions(
    db_pool: sqlx::PgPool,
    message_id: String,
) -> anyhow::Result<u64> {
    let count = sqlx::query_scalar!(
        "SELECT COUNT(primary_key) as count FROM forum_thread_message_versions WHERE message_id = $1",
        message_id
    )
    .fetch_one(&db_pool)
    .await
   ?;
    if let Some(c) = count {
        let as_u64 = u64::try_from(c)?;
        return Ok(as_u64);
    }
    Err(anyhow::anyhow!("Error getting message versions count"))
}

pub async fn find_thread_message(
    db_pool: sqlx::PgPool,
    message_id: String,
) -> anyhow::Result<Option<ForumThreadMessage>> {
    let result = sqlx::query_as!(
        ForumThreadMessage,
        "SELECT primary_key, id, thread_id FROM forum_thread_messages WHERE id = $1",
        message_id
    )
    .fetch_optional(&db_pool)
    .await
   ?;
    Ok(result)
}

pub async fn get_count_thread_messages(
    db_pool: sqlx::PgPool,
    thread_id: String,
) -> anyhow::Result<u64> {
    let _ = get_thread(db_pool.clone(), thread_id.clone())
        .await
       ?;
    let count = sqlx::query_scalar!(
        "SELECT COUNT(primary_key) as count FROM forum_thread_messages WHERE thread_id = $1",
        thread_id
    )
    .fetch_one(&db_pool)
    .await
   ?;
    if let Some(c) = count {
        let as_u64 = u64::try_from(c)?;
        return Ok(as_u64);
    }
    Err(anyhow::anyhow!("Error getting thread messages count"))
}
pub async fn get_forum_threads(
    db_pool: sqlx::PgPool,
    forum_id: String,
) -> anyhow::Result<Vec<ForumThread>> {
    let mut threads: Vec<ForumThread> = vec![];
    match find_forum_by_id(db_pool.clone(), forum_id.clone()).await? {
        Some(f) => f,
        None => {
            return Ok(threads);
        }
    };

    let result_rows = match sqlx::query!(
        r#"SELECT name, id FROM public.forum_threads WHERE forum_id = $1"#,
        forum_id
    )
    .fetch_all(&db_pool)
    .await
    {
        Err(e) => {
            return Err(anyhow::anyhow!("{e:#?}"));
        }
        Ok(rr) => rr,
    };
    for row in result_rows {
        let id = row.id;
        let name = row.name;
        threads.push(ForumThread {
            id,
            name,
            forum_id: "".to_string(),
        });
    }
    Ok(threads)
}

pub async fn get_subforums(db_pool: sqlx::PgPool, forum_id: String) -> anyhow::Result<Vec<Forum>> {
    let mut subforums: Vec<Forum> = vec![];
    match find_forum_by_id(db_pool.clone(), forum_id.clone()).await? {
        Some(f) => f,
        None => {
            return Ok(subforums);
        }
    };

    let result_rows = match sqlx::query!(
        r#"SELECT name, id FROM public.forums WHERE parent_id = $1"#,
        forum_id
    )
    .fetch_all(&db_pool)
    .await
    {
        Err(e) => {
            return Err(anyhow::anyhow!("{e:#?}"));
        }
        Ok(rr) => rr,
    };
    for row in result_rows {
        let id = row.id;
        let name = row.name;
        subforums.push(Forum {
            id,
            name,
            parent_id: "".to_string(),
        });
    }
    Ok(subforums)
}

pub async fn find_thread(
    db_pool: sqlx::PgPool,
    id: String,
) -> anyhow::Result<Option<ForumThread>> {
    let result = sqlx::query_as!(
        ForumThread,
        "SELECT id, name, forum_id FROM forum_threads WHERE id = $1",
        id.clone()
    )
    .fetch_optional(&db_pool)
    .await
   ?;
    Ok(result)
}

pub async fn get_thread(
    db_pool: sqlx::PgPool,
    id: String,
) -> anyhow::Result<ForumThread> {
    let thread = match find_thread(db_pool.clone(), id)
        .await
        ?
    {
        Some(t) => t,
        None => {
            return Err(anyhow::anyhow!(
                "I called get_thread because I expected to get it, but it was not actually found.",
            ));
        }
    };
    Ok(thread)
}
pub async fn get_thread_by_forum_id_and_title(
    db_pool: sqlx::PgPool,
    forum_id: String,
    title: String,
) -> Result<ForumThread, ExpectedToGetButFailed> {
    match find_thread_by_forum_id_and_title(db_pool, forum_id, title)
        .await
        .map_err(ExpectedToGetButFailed::new)
    {
        Ok(Some(t)) => Ok(t),
        Ok(None) => Err(ExpectedToGetButFailed::new(
            "Thread with that id does not appear to exist",
        )),
        Err(e) => Err(e),
    }
}

pub async fn find_thread_by_forum_id_and_title(
    db_pool: sqlx::PgPool,
    forum_id: String,
    title: String,
) -> anyhow::Result<Option<ForumThread>> {
    let thread_id = match sqlx::query!(
        "SELECT id FROM forum_threads WHERE forum_id = $1 AND name = $2",
        forum_id,
        title
    )
    .fetch_optional(&db_pool)
    .await
    ?
    {
        Some(t) => t.id,
        None => return Ok(None),
    };

    let t = get_thread(db_pool, thread_id).await?;
    Ok(Some(t))
}

pub async fn get_latest_message_version(
    db_pool: sqlx::PgPool,
    msg_id: String,
) -> anyhow::Result<ForumThreadMessageVersion> {
    let result = sqlx::query!("SELECT primary_key, id, user_id, content, created_at, message_id FROM forum_thread_message_versions WHERE message_id = $1 ORDER BY created_at DESC LIMIT 1", msg_id).fetch_one(&db_pool).await?;
    let result = ForumThreadMessageVersion {
        primary_key: result.primary_key,
        id: result.id,
        message_id: result.message_id,
        user_id: result.user_id,
        content: result.content,
        created_at: result.created_at,
    };
    Ok(result)
}
pub async fn get_initial_message_version(
    db_pool: sqlx::PgPool,
    msg_id: String,
) -> anyhow::Result<ForumThreadMessageVersion> {
    let result = sqlx::query!("SELECT primary_key, id, user_id, content, created_at, message_id FROM forum_thread_message_versions WHERE message_id = $1 ORDER BY created_at ASC LIMIT 1", msg_id).fetch_one(&db_pool).await?;
    let result = ForumThreadMessageVersion {
        primary_key: result.primary_key,
        id: result.id,
        message_id: result.message_id,
        user_id: result.user_id,
        content: result.content,
        created_at: result.created_at,
    };
    Ok(result)
}

pub async fn add_message(
    db_pool: sqlx::PgPool,
    message_content: ValidForumThreadMessageContent,
    user_id: String,
    thread_id: String,
) -> anyhow::Result<()> {
    let mut tx = db_pool.begin().await?;

    let message_id_result = sqlx::query!("INSERT INTO forum_thread_messages (id, thread_id, user_id) VALUES (gen_random_uuid(), $1, $2) RETURNING ID", thread_id.clone(), user_id.clone()).fetch_one(&mut *tx).await?;

    let _message_version_id_result = sqlx::query!("INSERT INTO forum_thread_message_versions (id, user_id, content, message_id) VALUES (gen_random_uuid(), $1, $2, $3) RETURNING ID", user_id.clone(), message_content.to_string(), message_id_result.id).fetch_one(&mut *tx).await?;

    tx.commit().await?;
    Ok(())
}

pub async fn get_modifications_count(
    db_pool: sqlx::PgPool,
    msg_id: String,
) -> anyhow::Result<i64> {
    let result = sqlx::query_scalar!(
        "SELECT COUNT(*) as count FROM forum_thread_message_versions WHERE message_id = $1",
        msg_id
    )
    .fetch_one(&db_pool)
    .await
   ?;
    if let Some(c) = result {
        Ok(c)
    } else {
        Err(anyhow::anyhow!("Could not get modifications count"))
    }
}
pub async fn get_thread_message_ids(
    db_pool: sqlx::PgPool,
    thread_id: String,
    thread_page: u64,
    limit: u64,
) -> anyhow::Result<Vec<String>> {
    let limit = i64::try_from(limit)?;
    let thread_page = i64::try_from(thread_page)?;
    let offset = (thread_page - 1) * limit;
    let result = sqlx::query!(
        "SELECT id FROM forum_thread_messages WHERE thread_id = $1 LIMIT $2 OFFSET $3",
        thread_id,
        limit,
        offset
    )
    .fetch_all(&db_pool)
    .await?;
    let mut vec_string: Vec<String> = vec![];
    for r in result {
        vec_string.push(r.id.clone());
    }
    Ok(vec_string)
}
pub async fn create_thread(
    db_pool: sqlx::PgPool,
    id: String,
    name: ValidForumThreadName,
    content: ValidForumThreadMessageContent,
    user_id: String,
    forum_id: String,
) -> anyhow::Result<String> {
    storage_specific_create_thread(db_pool, id, name, content, user_id, forum_id).await
}

async fn storage_specific_create_thread(
    db_pool: sqlx::PgPool,
    id: String,
    name: ValidForumThreadName,
    content: ValidForumThreadMessageContent,
    user_id: String,
    forum_id: String,
) -> anyhow::Result<String> {
    let mut tx = db_pool.begin().await?;

    let thread_id_result = sqlx::query!(
        "INSERT INTO forum_threads (id, name, forum_id) VALUES ($1, $2, $3) RETURNING id",
        id,
        name.to_string(),
        forum_id
    )
    .fetch_one(&mut *tx)
    .await
    ?;

    let message_id = id_generator_unique_in_db::generate_unique_forum_thread_message_id(&mut tx)
        .await
        ?;
    let message_id_result = sqlx::query!("INSERT INTO forum_thread_messages (id, thread_id, user_id) VALUES ($1, $2, $3) RETURNING ID", message_id, thread_id_result.id.clone(), user_id.clone()).fetch_one(&mut *tx).await?;

    let _message_version_id_result = sqlx::query!("INSERT INTO forum_thread_message_versions (id, user_id, content, message_id) VALUES (gen_random_uuid(), $1, $2, $3) RETURNING ID", user_id.clone(), content.to_string(), message_id_result.id).fetch_one(&mut *tx).await?;

    tx.commit().await?;
    Ok(thread_id_result.id.clone())
}

async fn storage_specific_find_forum_by_id(
    db_pool: sqlx::PgPool,
    id: String,
) -> anyhow::Result<Option<Forum>> {
    let forum = match sqlx::query!(
        r#"SELECT id, name, parent_id FROM public.forums WHERE id = $1"#,
        id
    )
    .fetch_optional(&db_pool)
    .await
    {
        Ok(Some(f)) => {
            let id = f.id;
            let name = f.name;
            let parent_id = f.parent_id;
            let f = Forum {
                id,
                name,
                parent_id,
            };
            Some(f)
        }
        Ok(None) => None,
        Err(e) => {
            return Err(anyhow::anyhow!("{e:#?}"));
        }
    };
    Ok(forum)
}
