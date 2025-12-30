use postgres::{NoTls};
use r2d2::{Pool, PooledConnection};
use r2d2_postgres::PostgresConnectionManager;


pub type DbPool = Pool<PostgresConnectionManager<NoTls>>;
pub type DbClient = PooledConnection<PostgresConnectionManager<NoTls>>;

pub fn get_pool(db_str: String) -> DbPool {
    let db_manager = PostgresConnectionManager::new(db_str.parse().unwrap(), NoTls);
    let db_pool = r2d2::Pool::new(db_manager).expect("Could not create db_pool");
    return db_pool;
}

pub fn get_client(pool: DbPool) -> DbClient {
    let client = pool.get().expect("Could not get client");
    return client;
}

