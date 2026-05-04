use error_structs::SomeSortOfError;

pub const FORUM_ID: &str = "dynamic_stylesheets";
pub async fn get_content(
    db_pool: sqlx::PgPool,
    page: String,
    stylesheet_number: u64,
) -> Result<String, SomeSortOfError> {
    let thread =
        forum::get_thread_by_forum_id_and_title(db_pool.clone(), FORUM_ID.to_string(), page)
            .await
            .map_err(SomeSortOfError::new)?;
    let message = forum::forum_thread_message::get_message_by_position_in_thread(
        db_pool.clone(),
        thread.id,
        stylesheet_number,
    )
    .await
    .map_err(SomeSortOfError::new)?;
    let content = forum::get_latest_message_version(db_pool.clone(), message.id)
        .await
        .map_err(SomeSortOfError::new)?
        .content;
    Ok(content)
}

pub async fn get_numbers_for_page(
    db_pool: sqlx::PgPool,
    page: String,
) -> Result<Vec<u64>, SomeSortOfError> {
    let page = page.as_str();
    let page = page.strip_prefix("/").unwrap_or(page);
    let page = page.to_string();

    let thread = forum::get_thread_by_forum_id_and_title(
        db_pool.clone(),
        "dynamic_stylesheets".to_string(),
        page,
    )
    .await
    .map_err(SomeSortOfError::new)?;
    let count = forum::get_count_thread_messages(db_pool.clone(), thread.id).await?;
    Ok((0..count).collect())
}
