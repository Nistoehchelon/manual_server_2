use db_manager::DbError;

use crate::forum_structs::{Forum, ForumThread};

pub fn get_forum_by_id(db_pool: &db_manager::DbPool, id: String) -> Result<Option<Forum>, DbError> {
    if id == "0" {
        return Ok(Some(Forum {
            public_id: "0".to_string(),
            name: "Default forum".to_string(),
            parent_id: "0".to_string(),
        }));
    }
    storage_specific_get_forum_by_id(db_pool, id.clone())
}

pub fn get_forum_threads(db_pool: &db_manager::DbPool, forum_id: String) -> Result<Vec<ForumThread>, DbError> {
    let mut threads: Vec<ForumThread> = vec![]; 
    match get_forum_by_id(&db_pool, forum_id.clone())? {
        Some(f) => f,
        None => {
            return Ok(threads);
        }
    };
    let client = db_manager::get_client(db_pool.clone())?;
    let query = "SELECT name, public_id FROM public.forum_threads WHERE forum_id = $1";
    let params: db_manager::DbParams = &[&forum_id];
    let result_rows = db_manager::query(client, query, params)?;
    for row in result_rows {
        let public_id = row.get("public_id");
        let name = row.get("name");
        threads.push(ForumThread { public_id, name, forum_id: "".to_string() });
    }
    return Ok(threads);
}

pub fn get_subforums(db_pool: &db_manager::DbPool,  forum_id: String) -> Result<Vec<Forum>, DbError> {
    let mut subforums: Vec<Forum> = vec![]; 
    match get_forum_by_id(&db_pool, forum_id.clone())? {
        Some(f) => f,
        None => {
            return Ok(subforums);
        }
    };
    let client = db_manager::get_client(db_pool.clone())?;
    let query = "SELECT name, public_id FROM public.forums WHERE parent_id = $1";
    let params: db_manager::DbParams = &[&forum_id];
    let result_rows = db_manager::query(client, query, params)?;
    for row in result_rows {
        let public_id = row.get("public_id");
        let name = row.get("name");
        subforums.push(Forum { public_id, name, parent_id: "".to_string() });
    }
    return Ok(subforums);
}

fn storage_specific_get_forum_by_id(
    db_pool: &db_manager::DbPool,
    id: String,
) -> Result<Option<Forum>, DbError> {
    let client = db_manager::get_client(db_pool.clone())?;
    let query= "SELECT public_id, name FROM public.forums WHERE public_id = $1";
    let params: db_manager::DbParams = &[&id];
    println!("{}", query);
    let forum: Option<Forum> = match db_manager::query_opt(client, query, params)? {
        Some(f) => {
            let public_id = f.get("public_id");
            let name = f.get("name");
            let f = Forum { public_id, name, parent_id: "".to_string() };
            Some(f)
        },
        None => None,
    };
    return Ok(forum);
}
