use std::fmt::Debug;

use deadpool_postgres::{Client, Config, Pool, Runtime};
use tokio_postgres::types::ToSql;
use tokio_postgres::{NoTls, Row, ToStatement};

pub type DbPool = Pool;
pub type DbClient = Client;
pub type DbRow = Row;
pub type DbParam<'a> = &'a (dyn ToSql + Sync);
pub type DbParams<'a> = &'a [DbParam<'a>];

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
pub fn get_pool(db_str: String) -> Result<DbPool, DbError> {
    let mut cfg = Config::new();
    cfg.url = Some(db_str);

    cfg.create_pool(Some(Runtime::Tokio1), NoTls)
        .map_err(DbError::new)
}

pub async fn get_client(pool: &DbPool) -> Result<DbClient, DbError> {
    pool.get().await.map_err(DbError::new)
}

pub async fn query<T>(
    client: &DbClient,
    query: &T,
    params: &[&(dyn ToSql + Sync)],
) -> Result<Vec<DbRow>, DbError>
where
    T: ?Sized + ToStatement,
{
    client.query(query, params).await.map_err(DbError::new)
}

pub async fn query_one<T>(
    client: &DbClient,
    statement: &T,
    params: &[&(dyn ToSql + Sync)],
) -> Result<DbRow, DbError>
where
    T: ?Sized + ToStatement,
{
    let result = client
        .query_one(statement, params)
        .await
        .map_err(DbError::new);
    result
}

pub async fn execute<T>(
    client: &DbClient,
    statement: &T,
    params: &[&(dyn ToSql + Sync)],
) -> Result<u64, DbError>
where
    T: ?Sized + ToStatement,
{
    let result = client
        .execute(statement, params)
        .await
        .map_err(DbError::new);
    result
}

pub async fn query_opt<T>(
    client: &DbClient,
    statement: &T,
    params: &[&(dyn ToSql + Sync)],
) -> Result<Option<Row>, DbError>
where
    T: ?Sized + ToStatement,
{
    let result = client
        .query_opt(statement, params)
        .await
        .map_err(DbError::new);
    return result;
}
