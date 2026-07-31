use custom_site_variables::find_custom_site_variable;
use db_manager::DbError;
use error_structs::SomeSortOfError;
use forum::ForumThread;

pub const FORUM_ID: &str = "custom_site_pages";

pub type SitePage = ForumThread;

pub struct CustomSitePage {
    pub name: String,
    pub content: String,
}
pub async fn find_custom_site_page(
    db_pool: sqlx::PgPool,
    page_path: String,
) -> Result<Option<CustomSitePage>, SomeSortOfError> {
    let thread = match forum::find_thread(db_pool.clone(), page_path.clone()).await {
        Ok(Some(t)) => t,
        Ok(None) => return Ok(None),
        Err(e) => {
            return Err(SomeSortOfError::new(e));
        }
    };

    let content: String = {
        let thread_page = 1;
        let limit = 1;
        let msg_id =
            forum::get_thread_message_ids(db_pool.clone(), page_path.clone(), thread_page, limit)
                .await
                .map_err(SomeSortOfError::new)?;
        let msg_id = msg_id[0].to_string();
        let msg = forum::get_latest_message_version(db_pool.clone(), msg_id.clone())
            .await
            .map_err(SomeSortOfError::new)?;
        let msg_content = msg.content;
        replace_variables_with_contents(db_pool.clone(), msg_content.to_string())
            .await
            .map_err(SomeSortOfError::new)?
    };
    Ok(Some(CustomSitePage {
        name: thread.name,
        content,
    }))
}
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
use std::collections::HashSet;

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
        let mut tx = db_pool.clone().begin().await.map_err(DbError::new)?;
        let Ok(var_name_parsed) = name.parse() else {
            cache.insert(name.clone(), format!("var {name} not found"));
            continue;
        };
        let var_val = find_custom_site_variable(&mut tx, var_name_parsed)
            .await
            .map_err(DbError::new)?;
        tx.commit().await.map_err(DbError::new)?;
        if let Some(var_val) = var_val {
            cache.insert(name, var_val);
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
