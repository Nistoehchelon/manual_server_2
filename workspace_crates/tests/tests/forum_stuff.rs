


    use config_reader::read_config;
use session;
use forum;

#[tokio::test(flavor="multi_thread", worker_threads = 2)]
async fn test_read_write() {
    let env = read_config();
    let pool = sqlx::PgPool::connect(&env.database_str).await.expect("No pool :(");
 	// let limit_per_page = 15;
	// let thread_message_ids = forum::get_thread_message_ids(&db_pool, thread_id.clone(), thread_page.clone(), limit_per_page.clone()).await.map_err(InternalAppError::new)?;
    todo!();
}
