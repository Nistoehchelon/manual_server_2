use db_manager::DbError;

use crate::forum_structs::{Forum, ForumThread};

pub async fn get_forum_by_id(db_pool: &sqlx::PgPool, id: String) -> Result<Option<Forum>, DbError> {
    if id == "0" {
        return Ok(Some(Forum {
            public_id: "0".to_string(),
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

    let result_rows = match sqlx::query!(r#"SELECT name, public_id FROM public.forum_threads WHERE forum_id = $1"#, forum_id).fetch_all(db_pool).await {
        Err(e) => {
            return Err(DbError::new(e));
        },
        Ok(rr) => rr,
    };
    for row in result_rows {
        let public_id = row.public_id;
        let name = row.name;
        threads.push(ForumThread { public_id, name, forum_id: "".to_string() });
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

    let result_rows = match sqlx::query!(r#"SELECT name, public_id FROM public.forums WHERE parent_id = $1"#, forum_id).fetch_all(db_pool).await {
        Err(e) => {
            return Err(DbError::new(e));
        },
        Ok(rr) => rr,
    };
    for row in result_rows {
        let public_id = row.public_id;
        let name = row.name;
        subforums.push(Forum { public_id, name, parent_id: "".to_string() });
    }
    return Ok(subforums);
}

async fn storage_specific_get_forum_by_id(
    db_pool: &sqlx::PgPool,
    id: String,
) -> Result<Option<Forum>, DbError> {

    let forum = match sqlx::query!(r#"SELECT public_id, name FROM public.forums WHERE public_id = $1"#, id).fetch_optional(db_pool).await {
        Ok(Some(f)) => {
            let public_id = f.public_id;
            let name = f.name;
            let f = Forum { public_id, name, parent_id: "".to_string() };
            Some(f)
        },
        Ok(None) => None,
        Err(e) => {
            return Err(DbError::new(e));
        }
    };
    return Ok(forum);
}
