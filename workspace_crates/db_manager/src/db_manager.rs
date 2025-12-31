use std::fmt::Debug;

use postgres::NoTls;
use r2d2::{Pool, PooledConnection};
use r2d2_postgres::PostgresConnectionManager;

pub type DbPool = Pool<PostgresConnectionManager<NoTls>>;
pub type DbClient = PooledConnection<PostgresConnectionManager<NoTls>>;
pub type DbRow = postgres::Row;

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
pub fn get_pool(db_str: String) -> DbPool {
    let db_manager = PostgresConnectionManager::new(db_str.parse().unwrap(), NoTls);
    let db_pool = r2d2::Pool::new(db_manager).expect("Could not create db_pool");
    return db_pool;
}

pub fn get_client(pool: DbPool) -> DbClient {
    let client = pool.get().expect("Could not get client");
    return client;
}

use postgres::types::ToSql;

pub fn query_one<T>(
    mut client: DbClient,
    query: &T,
    params: &[&(dyn ToSql + Sync)],
) -> Result<DbRow, DbError>
where
    T: ?Sized + postgres::ToStatement,
{
    let result = client.query_one(query, params);
    match result {
        Ok(r) => return Ok(r),
        Err(e) => return Err(DbError::new(e)),
    }
}

pub fn execute<T>(
    mut client: DbClient,
    query: &T,
    params: &[&(dyn ToSql + Sync)],
) -> Result<u64, DbError>
where
    T: ?Sized + postgres::ToStatement,
{
    let result = client.execute(query, params);
    match result {
        Err(e) => return Err(DbError::new(e)),
        Ok(rows_modified) => return Ok(rows_modified),
    }
}

pub fn query_opt<T>(
    mut client: DbClient,
    query: &T,
    params: &[&(dyn ToSql + Sync)],
) -> Result<Option<DbRow>, DbError>
where
    T: ?Sized + postgres::ToStatement,
{
    let result = client.query_opt(query, params);
    match result {
        Err(e) => return Err(DbError::new(e)),
        Ok(opt) => return Ok(opt),
    }
}
