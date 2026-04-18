use db_manager::DbError;
use error_structs::SomeSortOfError;
use forum::ForumThread;

pub type SitePage = ForumThread;
pub async fn list_of_site_pages(
    db_pool: sqlx::PgPool,
    _pagination_page: i64,
    _pagination_records_per_page: i64,
) -> Result<Vec<SitePage>, SomeSortOfError> {
    let site_pages = forum::get_forum_threads(db_pool.clone(), "site_pages".to_string())
        .await
        .map_err(SomeSortOfError::new)?;
    Ok(site_pages)
}

// Gemini generated this. Supposedly, a simple parser to replace [[varname]] with db-defined vars or none if not found. To be used in site pages. So, I can update something dynamically, like contact number and stuff
pub async fn replace_variables_with_contents(
    db_pool: sqlx::PgPool,
    input: String,
) -> Result<String, DbError> {
    let db_pool = &db_pool;
    let input = input.as_str();
    let mut output = input.to_string();
    let mut depth = 0;

    while depth < 5 {
        let mut names = Vec::new();
        let mut cursor = 0;

        // 1. Manual extraction scan
        while let Some(start_idx) = output[cursor..].find("[[") {
            let start = cursor + start_idx + 2;
            if let Some(end_idx) = output[start..].find("]]") {
                let end = start + end_idx;
                names.push(&output[start..end]);
                cursor = end + 2;
            } else {
                break; // Unclosed tag
            }
        }

        if names.is_empty() {
            break;
        }
        names.sort_unstable();
        names.dedup();

        // 1. Convert the &str names into owned Strings
        let names_owned: Vec<String> = names.into_iter().map(String::from).collect();
        // 2. Database Fetch (Same as before)
        let rows = sqlx::query!(
            r#"SELECT name, value FROM custom_site_variables WHERE  name = ANY($1)"#,
            &names_owned[..]
        )
        .fetch_all(db_pool)
        .await
        .map_err(DbError::new)?;

        let cache: std::collections::HashMap<_, _> =
            rows.into_iter().map(|r| (r.name, r.value)).collect();

        // 3. Manual Build/Replace
        let mut new_text = String::with_capacity(output.len());
        let mut last_end = 0;
        cursor = 0;

        while let Some(start_idx) = output[cursor..].find("[[") {
            let start = cursor + start_idx;
            let content_start = start + 2;
            if let Some(end_idx) = output[content_start..].find("]]") {
                let content_end = content_start + end_idx;
                let end = content_end + 2;

                // Push text before the tag
                new_text.push_str(&output[last_end..start]);

                // Push replacement
                let var_name = &output[content_start..content_end];
                new_text.push_str(cache.get(var_name).unwrap_or(&"none".to_string()));

                cursor = end;
                last_end = end;
            } else {
                break;
            }
        }
        new_text.push_str(&output[last_end..]);

        if output == new_text {
            break;
        }
        output = new_text;
        depth += 1;
    }

    Ok(output)
}
