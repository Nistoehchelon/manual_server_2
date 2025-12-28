use crate::{field_count_trait::FieldCount, storage::TextStorable};

#[derive(Clone, Debug, Default)]
pub struct Forum {
    pub id: String,
    pub name: String,
    pub parent_id: String,
    pub access_level: usize,
}
#[derive(Clone, Debug, Default)]
pub struct ForumThread {
    pub id: String,
    pub name: String,
    pub forum_id: String,
}
impl FieldCount for ForumThread {
    fn field_count() -> usize {
        3
    }
}

impl TextStorable for ForumThread {
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

impl FieldCount for Forum {
    fn field_count() -> usize {
        4
    }
}

impl TextStorable for Forum {
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