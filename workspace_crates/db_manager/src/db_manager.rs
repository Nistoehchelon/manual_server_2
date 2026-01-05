use std::fmt::Debug;

use postgres::NoTls;
use r2d2::{Pool, PooledConnection};
use r2d2_postgres::PostgresConnectionManager;
use postgres::types::ToSql;

pub type DbPool = Pool<PostgresConnectionManager<NoTls>>;
pub type DbClient = PooledConnection<PostgresConnectionManager<NoTls>>;
pub type DbRow = postgres::Row;
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
    let db_manager = PostgresConnectionManager::new(db_str.parse().unwrap(), NoTls);
    match r2d2::Pool::new(db_manager) {
        Ok(p) => return Ok(p),
        Err(e) => return Err(DbError::new(e)),
    }
}

pub fn get_client(pool: DbPool) -> Result<DbClient, DbError> {
    match pool.get() {
        Ok(c) => return Ok(c),
        Err(e) => return Err(DbError::new(e)),
    }
}

pub fn query<T>(mut client: DbClient, query: &T, params: DbParams) -> Result<Vec<DbRow>, DbError>
    where
        T: ?Sized + postgres::ToStatement,
    {
        let result = client.query(query, params);
        match result {
            Ok(r) => return Ok(r),
            Err(e) => return Err(DbError::new(e)),
        }
    }

pub fn query_one<T>(
    mut client: DbClient,
    query: &T,
    params: DbParams,
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
    params: DbParams,
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
    params: DbParams,
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
