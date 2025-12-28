use storage::TextStorable;


#[derive(Clone, Debug, Default)]
pub struct ForumThread {
    pub id: String,
    pub name: String,
    pub forum_id: String,
}

impl TextStorable for ForumThread {
    fn field_count() -> usize {
        3
    }
    fn to_fields(&self) -> Vec<String> {
        vec![self.id.clone(), self.name.clone(), self.forum_id.clone()]
    }
    fn from_fields(fields: Vec<String>) -> Option<Self> {
        if fields.len() != Self::field_count() {
            panic!(
                "Mismatch of expected struct fields. Expected: {}, got: {}",
                Self::field_count(),
                fields.len()
            );
        }
        Some(Self {
            id: fields[0].clone(),
            name: fields[1].clone(),
            forum_id: fields[2].clone(),
        })
    }
    fn file_name() -> &'static str {
        "forum_threads"
    }
}

#[derive(Clone, Debug, Default)]
pub struct Forum {
    pub id: String,
    pub name: String,
    pub parent_id: String,
    pub access_level: usize,
}

impl TextStorable for Forum {
    fn field_count() -> usize {
        4
    }
    fn to_fields(&self) -> Vec<String> {
        vec![
            self.id.clone(),
            self.name.clone(),
            self.parent_id.clone(),
            self.access_level.clone().to_string(),
        ]
    }
    fn from_fields(fields: Vec<String>) -> Option<Self> {
        if fields.len() != Self::field_count() {
            panic!(
                "Mismatch of expected struct fields. Expected: {}, got: {}",
                Self::field_count(),
                fields.len()
            );
        }
        let access_level_str = fields[3].clone();
        let access_level = access_level_str
            .parse::<usize>()
            .expect(format!("Could not parse {} into usize", access_level_str).as_str());
        Some(Self {
            id: fields[0].clone(),
            name: fields[1].clone(),
            parent_id: fields[2].clone(),
            access_level,
        })
    }
    fn file_name() -> &'static str {
        "forums"
    }
}
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
