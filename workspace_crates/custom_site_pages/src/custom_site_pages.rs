use db_manager::DbError;
use error_structs::SomeSortOfError;
use forum::ForumThread;

pub const FORUM_ID: &str = "custom_site_pages";

pub type SitePage = ForumThread;
pub async fn list_of_site_pages(
    db_pool: sqlx::PgPool,
    _pagination_page: i64,
    _pagination_records_per_page: i64,
) -> Result<Vec<SitePage>, SomeSortOfError> {
    let site_pages = forum::get_forum_threads(db_pool.clone(), FORUM_ID.to_string())
        .await
        .map_err(SomeSortOfError::new)?;
    Ok(site_pages)
}

use regex::Regex;
use std::collections::{HashSet};

pub async fn replace_variables_with_contents(
    db_pool: sqlx::PgPool,
    input: String,
) -> Result<String, DbError> {
    let input = input.as_str();
 // match [[variable_name]]
    let re = Regex::new(r"\[\[(.*?)\]\]").unwrap();

    let mut output = input.to_string();

    // 1. Collect unique variable names
    let mut names = HashSet::new();
    for cap in re.captures_iter(input) {
        let name = cap[1].trim().to_lowercase();
        names.insert(name);
    }

    // 2. Fetch each variable individually
    let mut cache = std::collections::HashMap::new();

    for name in names {
        let row = sqlx::query!(
            r#"SELECT value FROM custom_site_variables WHERE name = $1"#,
            name
        )
        .fetch_optional(&db_pool)
        .await
        .map_err(DbError::new)?;

        if let Some(r) = row {
            cache.insert(name, r.value);
        }
    }

    // 3. Replace using regex
    output = re
        .replace_all(&output, |caps: &regex::Captures| {
            let key = caps[1].trim().to_lowercase();
            cache
                .get(&key)
                .map(|s| s.as_str())
                .unwrap_or("none")
                .to_string()
        })
        .to_string();

    Ok(output)
}