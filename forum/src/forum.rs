use crate::forum_structs::{Forum,ForumThread};

pub fn get_forum_by_id(id: String) -> Option<Forum> {
    if id == "0" {
        return Some(Forum {
            id: "0".to_string(),
            name: "Default forum".to_string(),
            access_level: 1,
            parent_id: "0".to_string(),
        });
    }
    storage_specific_get_forum_by_id(id.clone())
}

pub fn get_forum_threads(forum_id: String) -> Vec<ForumThread> {
    let all_threads = storage::read_contents::<ForumThread>();
    let mut forum_threads: Vec<ForumThread> = vec![];
    for t in all_threads {
        if !(t.forum_id == forum_id) {
            continue;
        }
        forum_threads.push(t.clone());
    }
    forum_threads
}

fn storage_specific_get_forum_by_id(id: String) -> Option<Forum> {
    let forums = storage::read_contents::<Forum>();
    let mut exists = false;
    let mut forum = Forum::default();
    for f in forums {
        if f.id == id {
            exists = true;
            forum = f.clone();
        }
    }
    if !exists {
        return None;
    }
    Some(forum)
}
