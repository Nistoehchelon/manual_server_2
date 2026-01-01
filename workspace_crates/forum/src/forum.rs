use db_manager::DbError;

use crate::forum_structs::{Forum, ForumThread};

pub fn get_forum_by_id(db_pool: &db_manager::DbPool, id: String) -> Result<Option<Forum>, DbError> {
    if id == "0" {
        return Ok(Some(Forum {
            id: "0".to_string(),
            name: "Default forum".to_string(),
            access_level: 1,
            parent_id: "0".to_string(),
        }));
    }
    storage_specific_get_forum_by_id(db_pool, id.clone())
}

pub fn get_forum_threads(_forum_id: String) -> Result<Vec<ForumThread>, DbError> {
    todo!()
}

pub fn get_subforums(forum_id: String) -> Result<Vec<Forum>, DbError> {
    todo!()
}

fn storage_specific_get_forum_by_id(
    db_pool: &db_manager::DbPool,
    id: String,
) -> Result<Option<Forum>, DbError> {
    let mut client = db_manager::get_client(db_pool.clone())?;
    todo!()
}
