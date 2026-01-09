use db_manager::DbError;

use crate::forum_structs::{Forum, ForumThread};

pub async fn get_forum_by_id(db_pool: &sqlx::PgPool, id: String) -> Result<Option<Forum>, DbError> {
    if id == "0" {
        return Ok(Some(Forum {
            id: "0".to_string(),
            name: "Default forum".to_string(),
            parent_id: "0".to_string(),
        }));
    }
    storage_specific_get_forum_by_id(db_pool, id.clone()).await
}

pub async fn get_forum_threads(db_pool: &sqlx::PgPool, forum_id: String) -> Result<Vec<ForumThread>, DbError> {
    let mut threads: Vec<ForumThread> = vec![]; 
    match get_forum_by_id(&db_pool, forum_id.clone()).await? {
        Some(f) => f,
        None => {
            return Ok(threads);
        }
    };

    let result_rows = match sqlx::query!(r#"SELECT name, id FROM public.forum_threads WHERE forum_id = $1"#, forum_id).fetch_all(db_pool).await {
        Err(e) => {
            return Err(DbError::new(e));
        },
        Ok(rr) => rr,
    };
    for row in result_rows {
        let id = row.id;
        let name = row.name;
        threads.push(ForumThread { id, name, forum_id: "".to_string() });
    }
    return Ok(threads);
}

pub async fn get_subforums(db_pool: &sqlx::PgPool,  forum_id: String) -> Result<Vec<Forum>, DbError> {
    let mut subforums: Vec<Forum> = vec![]; 
    match get_forum_by_id(&db_pool, forum_id.clone()).await? {
        Some(f) => f,
        None => {
            return Ok(subforums);
        }
    };

    let result_rows = match sqlx::query!(r#"SELECT name, id FROM public.forums WHERE parent_id = $1"#, forum_id).fetch_all(db_pool).await {
        Err(e) => {
            return Err(DbError::new(e));
        },
        Ok(rr) => rr,
    };
    for row in result_rows {
        let id = row.id;
        let name = row.name;
        subforums.push(Forum { id, name, parent_id: "".to_string() });
    }
    return Ok(subforums);
}


pub async fn create_thread(db_pool: &sqlx::PgPool, name: String, content: String, user_id: String, forum_id: String) -> Result<String, DbError> {
    return storage_specific_create_thread(db_pool, name, content, user_id, forum_id,).await;
    todo!();
}

async fn storage_specific_create_thread(db_pool: &sqlx::PgPool, name: String, content: String, user_id: String, forum_id: String) ->Result<String, DbError> {
    let mut tx = db_pool.begin().await.map_err(DbError::new)?;

    let thread_id_result = sqlx::query!("INSERT INTO forum_threads (id, name, forum_id) VALUES (gen_random_uuid(), $1, $2) RETURNING id", name, forum_id).fetch_one(&mut *tx).await.map_err(DbError::new)?;

    let message_id_result = sqlx::query!("INSERT INTO forum_thread_messages (id, thread_id, user_id) VALUES (gen_random_uuid(), $1, $2) RETURNING ID", thread_id_result.id.clone(), user_id.clone()).fetch_one(&mut *tx).await.map_err(DbError::new)?;


    let message_version_id_result = sqlx::query!("INSERT INTO forum_thread_message_versions (id, user_id, content, message_id) VALUES (gen_random_uuid(), $1, $2, $3) RETURNING ID", user_id.clone(), content, message_id_result.id).fetch_one(&mut *tx).await.map_err(DbError::new)?;

    tx.commit().await.map_err(DbError::new)?;
    return Ok(thread_id_result.id.clone());    
}


async fn storage_specific_get_forum_by_id(
    db_pool: &sqlx::PgPool,
    id: String,
) -> Result<Option<Forum>, DbError> {

    let forum = match sqlx::query!(r#"SELECT id, name FROM public.forums WHERE id = $1"#, id).fetch_optional(db_pool).await {
        Ok(Some(f)) => {
            let id = f.id;
            let name = f.name;
            let f = Forum { id, name, parent_id: "".to_string() };
            Some(f)
        },
        Ok(None) => None,
        Err(e) => {
            return Err(DbError::new(e));
        }
    };
    return Ok(forum);
}
