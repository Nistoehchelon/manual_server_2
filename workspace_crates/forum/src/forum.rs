use db_manager::DbFailureError;

use crate::forum_structs::{Forum,ForumThread};

pub fn get_forum_by_id(id: String) -> Result<Option<Forum>, DbFailureError> {
    if id == "0" {
        return Ok(Some(Forum {
            id: "0".to_string(),
            name: "Default forum".to_string(),
            access_level: 1,
            parent_id: "0".to_string(),
        }));
    }
    storage_specific_get_forum_by_id(id.clone())
}

pub fn get_forum_threads(_forum_id: String) -> Result<Vec<ForumThread>, DbFailureError> {
    todo!()   
}

fn storage_specific_get_forum_by_id(_id: String) -> Result<Option<Forum>, DbFailureError> {
    todo!()
}

