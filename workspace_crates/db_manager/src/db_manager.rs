use std::fmt::Debug;

pub type DbPool = sqlx::pool::Pool<sqlx::Postgres>;

#[derive(Debug)]
pub struct DbError {
    pub message: String,
}

impl DbError {
    pub fn new<E: Debug>(s: E) -> Self {
        let message = format!("{s:?}");
        println!("DbError: {}", message.clone());
        Self { message }
    }
}

pub struct DbSuccess {
    pub message: String,
}
impl DbSuccess {
    pub fn new(s: &str) -> Self {
        Self {
            message: format!("{}", s),
        }
    }
}
