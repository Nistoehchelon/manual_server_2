use crate::{
    ForumThreadMessage,
    forum_structs::{Forum, ForumThread, ForumThreadMessageVersion},
};
use db_manager::DbError;
use error_structs::{ExpectedToGetButFailed, SomeSortOfError};


pub async fn generate_unique_thread_id(db_pool: sqlx::PgPool) -> Result<String, DbError> {
    let max_iterations = 20;
    for _ in 0..max_iterations {
        let id = id_generator::generate_unique_id();
        
        let exists = sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM forum_threads WHERE id = $1) as "exists!""#,
            id
        )
        .fetch_one(&db_pool)
        .await
        .map_err(DbError::new)?;

        if !exists {
            return Ok(id);
        }
    }

    Err(DbError::new("Failed to generate a unique thread ID after 20 attempts"))

}
pub async fn find_forum_by_id(db_pool: sqlx::PgPool, id: String) -> Result<Option<Forum>, DbError> {
    if id == "0" {
        return Ok(Some(Forum {
            id: "0".to_string(),
            name: "Default forum".to_string(),
            parent_id: "0".to_string(),
        }));
    }
    storage_specific_get_forum_by_id(db_pool, id.clone()).await
}

pub async fn get_count_thread_pages(
    db_pool: sqlx::PgPool,
    thread_id: String,
    messages_per_page: u64,
) -> Result<u64, SomeSortOfError> {
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
) -> Result<Vec<ForumThreadMessageVersion>, SomeSortOfError> {
    let limit = i64::try_from(records_per_page).map_err(SomeSortOfError::new)?;
    let page = i64::try_from(page).map_err(SomeSortOfError::new)?;
    let offset = (page - 1) * limit;
    let msg_vers = sqlx::query_as!(ForumThreadMessageVersion, "SELECT primary_key, id, user_id, message_id, content, created_at FROM forum_thread_message_versions WHERE message_id = $1 ORDER BY primary_key DESC LIMIT $2 OFFSET $3", message_id, limit, offset).fetch_all(&db_pool).await.map_err(SomeSortOfError::new)?;

    Ok(msg_vers)
}

pub async fn get_count_thread_message_versions(
    db_pool: sqlx::PgPool,
    message_id: String,
) -> Result<u64, SomeSortOfError> {
    let count = sqlx::query_scalar!(
        "SELECT COUNT(primary_key) as count FROM forum_thread_message_versions WHERE message_id = $1",
        message_id
    )
    .fetch_one(&db_pool)
    .await
    .map_err(SomeSortOfError::new)?;
    if let Some(c) = count {
        let as_u64 = u64::try_from(c).map_err(SomeSortOfError::new)?;
        return Ok(as_u64);
    }
    Err(SomeSortOfError::new("Error getting message versions count"))
}

pub async fn find_thread_message(
    db_pool: sqlx::PgPool,
    message_id: String,
) -> Result<Option<ForumThreadMessage>, SomeSortOfError> {
    let result = sqlx::query_as!(
        ForumThreadMessage,
        "SELECT primary_key, id, thread_id FROM forum_thread_messages WHERE id = $1",
        message_id
    )
    .fetch_optional(&db_pool)
    .await
    .map_err(SomeSortOfError::new)?;
    Ok(result)
}

pub async fn get_count_thread_messages(
    db_pool: sqlx::PgPool,
    thread_id: String,
) -> Result<u64, SomeSortOfError> {
    let count = sqlx::query_scalar!(
        "SELECT COUNT(primary_key) as count FROM forum_thread_messages WHERE thread_id = $1",
        thread_id
    )
    .fetch_one(&db_pool)
    .await
    .map_err(SomeSortOfError::new)?;
    if let Some(c) = count {
        let as_u64 = u64::try_from(c).map_err(SomeSortOfError::new)?;
        return Ok(as_u64);
    }
    Err(SomeSortOfError::new("Error getting thread messages count"))
}
pub async fn get_forum_threads(
    db_pool: sqlx::PgPool,
    forum_id: String,
) -> Result<Vec<ForumThread>, DbError> {
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
            return Err(DbError::new(e));
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

pub async fn get_subforums(db_pool: sqlx::PgPool, forum_id: String) -> Result<Vec<Forum>, DbError> {
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
            return Err(DbError::new(e));
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
) -> Result<Option<ForumThread>, DbError> {
    let result = sqlx::query_as!(
        ForumThread,
        "SELECT id, name, forum_id FROM forum_threads WHERE id = $1",
        id.clone()
    )
    .fetch_optional(&db_pool)
    .await
    .map_err(DbError::new)?;
    Ok(result)
}

pub async fn get_thread(
    db_pool: sqlx::PgPool,
    id: String,
) -> Result<ForumThread, ExpectedToGetButFailed> {
    let thread = match find_thread(db_pool.clone(), id)
        .await
        .map_err(ExpectedToGetButFailed::new)?
    {
        Some(t) => t,
        None => {
            return Err(ExpectedToGetButFailed::new("..."));
        }
    };
    Ok(thread)
}

pub async fn get_latest_message_version(
    db_pool: sqlx::PgPool,
    msg_id: String,
) -> Result<ForumThreadMessageVersion, SomeSortOfError> {
    let result = sqlx::query_as!(ForumThreadMessageVersion, "SELECT primary_key, id, user_id, content, created_at, message_id FROM forum_thread_message_versions WHERE message_id = $1 ORDER BY created_at DESC LIMIT 1", msg_id).fetch_one(&db_pool).await.map_err(SomeSortOfError::new)?;
    Ok(result)
}
pub async fn get_initial_message_version(
    db_pool: sqlx::PgPool,
    msg_id: String,
) -> Result<ForumThreadMessageVersion, SomeSortOfError> {
    let result = sqlx::query_as!(ForumThreadMessageVersion, "SELECT primary_key, id, user_id, content, created_at, message_id FROM forum_thread_message_versions WHERE message_id = $1 ORDER BY created_at ASC LIMIT 1", msg_id).fetch_one(&db_pool).await.map_err(SomeSortOfError::new)?;
    Ok(result)
}

pub async fn add_message(
    db_pool: sqlx::PgPool,
    message: String,
    user_id: String,
    thread_id: String,
) -> Result<(), DbError> {
    let mut tx = db_pool.begin().await.map_err(DbError::new)?;

    let message_id_result = sqlx::query!("INSERT INTO forum_thread_messages (id, thread_id, user_id) VALUES (gen_random_uuid(), $1, $2) RETURNING ID", thread_id.clone(), user_id.clone()).fetch_one(&mut *tx).await.map_err(DbError::new)?;

    let _message_version_id_result = sqlx::query!("INSERT INTO forum_thread_message_versions (id, user_id, content, message_id) VALUES (gen_random_uuid(), $1, $2, $3) RETURNING ID", user_id.clone(), message, message_id_result.id).fetch_one(&mut *tx).await.map_err(DbError::new)?;

    tx.commit().await.map_err(DbError::new)?;
    Ok(())
}

pub async fn get_modifications_count(
    db_pool: sqlx::PgPool,
    msg_id: String,
) -> Result<i64, SomeSortOfError> {
    let result = sqlx::query_scalar!(
        "SELECT COUNT(*) as count FROM forum_thread_message_versions WHERE message_id = $1",
        msg_id
    )
    .fetch_one(&db_pool)
    .await
    .map_err(SomeSortOfError::new)?;
    if let Some(c) = result {
        Ok(c)
    } else {
        Err(SomeSortOfError::new("Could not get modifications count"))
    }
}
pub async fn get_thread_message_ids(
    db_pool: sqlx::PgPool,
    thread_id: String,
    thread_page: u64,
    limit: u64,
) -> Result<Vec<String>, SomeSortOfError> {
    let limit = i64::try_from(limit).map_err(SomeSortOfError::new)?;
    let thread_page = i64::try_from(thread_page).map_err(SomeSortOfError::new)?;
    let offset = (thread_page - 1) * limit;
    let result = sqlx::query!(
        "SELECT id FROM forum_thread_messages WHERE thread_id = $1 LIMIT $2 OFFSET $3",
        thread_id,
        limit,
        offset
    )
    .fetch_all(&db_pool)
    .await
    .map_err(SomeSortOfError::new)?;
    let mut vec_string: Vec<String> = vec![];
    for r in result {
        vec_string.push(r.id.clone());
    }
    Ok(vec_string)
}
pub async fn create_thread(
    db_pool: sqlx::PgPool,
    id: String,
    name: String,
    content: String,
    user_id: String,
    forum_id: String,
) -> Result<String, DbError> {
    storage_specific_create_thread( db_pool,id, name, content, user_id, forum_id).await
}

async fn storage_specific_create_thread(
    db_pool: sqlx::PgPool,
    id: String,
    name: String,
    content: String,
    user_id: String,
    forum_id: String,
) -> Result<String, DbError> {
    let mut tx = db_pool.begin().await.map_err(DbError::new)?;



    let thread_id_result = sqlx::query!("INSERT INTO forum_threads (id, name, forum_id) VALUES ($1, $2, $3) RETURNING id", id, name, forum_id).fetch_one(&mut *tx).await.map_err(DbError::new)?;

    let message_id = id_generator::generate_unique_id();
    let message_id_result = sqlx::query!("INSERT INTO forum_thread_messages (id, thread_id, user_id) VALUES ($1, $2, $3) RETURNING ID", message_id, thread_id_result.id.clone(), user_id.clone()).fetch_one(&mut *tx).await.map_err(DbError::new)?;

    let _message_version_id_result = sqlx::query!("INSERT INTO forum_thread_message_versions (id, user_id, content, message_id) VALUES (gen_random_uuid(), $1, $2, $3) RETURNING ID", user_id.clone(), content, message_id_result.id).fetch_one(&mut *tx).await.map_err(DbError::new)?;

    tx.commit().await.map_err(DbError::new)?;
    Ok(thread_id_result.id.clone())
}

async fn storage_specific_get_forum_by_id(
    db_pool: sqlx::PgPool,
    id: String,
) -> Result<Option<Forum>, DbError> {
    let forum = match sqlx::query!(r#"SELECT id, name FROM public.forums WHERE id = $1"#, id)
        .fetch_optional(&db_pool)
        .await
    {
        Ok(Some(f)) => {
            let id = f.id;
            let name = f.name;
            let f = Forum {
                id,
                name,
                parent_id: "".to_string(),
            };
            Some(f)
        }
        Ok(None) => None,
        Err(e) => {
            return Err(DbError::new(e));
        }
    };
    Ok(forum)
}
