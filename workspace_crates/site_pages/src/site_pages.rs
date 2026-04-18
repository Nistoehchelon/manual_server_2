use error_structs::SomeSortOfError;

pub struct SitePage {
    pub primary_key: i64,
    pub id: String,
    pub name: String,
}
pub struct SitePageHistoryOfEditsEntry {
    pub content: String,
    pub user_id: String,
    pub timestamp: String,
}
   pub async fn list_of_site_pages(db_pool: sqlx::PgPool, pagination_page: i64, pagination_records_per_page: i64) -> Result<Vec<SitePage>, SomeSortOfError> {
    let mut tx = db_pool.begin().await.map_err(SomeSortOfError::new)?;
 
     let limit = pagination_records_per_page;
     let page = pagination_page;
     let offset = (page - 1) * limit;
      let site_pages = sqlx::query_as!(SitePage, "SELECT primary_key, id, name FROM site_pages ORDER BY primary_key DESC LIMIT $1 OFFSET $2", limit, offset).fetch_all(&mut *tx).await.map_err(SomeSortOfError::new)?;
      
      tx.commit().await.map_err(SomeSortOfError::new)?;
      Ok(site_pages)
   }


//    pub async fn get_thread_message_versions(
//     db_pool: sqlx::PgPool,
//     message_id: String,
//     page: u64,
//     records_per_page: u64,
// ) -> Result<Vec<ForumThreadMessageVersion>, SomeSortOfError> {
//     let limit = i64::try_from(records_per_page).map_err(SomeSortOfError::new)?;
//     let page = i64::try_from(page).map_err(SomeSortOfError::new)?;
//     let offset = (page - 1) * limit;
//     let msg_vers = sqlx::query_as!(ForumThreadMessageVersion, "SELECT primary_key, id, user_id, message_id, content, created_at FROM forum_thread_message_versions WHERE message_id = $1 ORDER BY primary_key DESC LIMIT $2 OFFSET $3", message_id, limit, offset).fetch_all(&db_pool).await.map_err(SomeSortOfError::new)?;

//     Ok(msg_vers)
// }