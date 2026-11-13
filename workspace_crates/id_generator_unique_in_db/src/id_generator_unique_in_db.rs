use anyhow::anyhow;
use sqlx::{Postgres, Transaction};

// I can use a non-macro sqlx::query_scalar or something, but I don't like the idea of silently finding out
// error only on runtime... Not sure if the match case below is worth it, though...
pub enum Table {
    Users,
    ForumThreads,
    ForumThreadMessages,
    Invoices,
    Forums,
}

pub async fn generate_unique_id_for(
    tx: &mut Transaction<'_, Postgres>,
    for_table: Table,
) -> anyhow::Result<String> {
    let max_iterations = 20;
    for _ in 0..max_iterations {
        let id = id_generator::generate_random_alphanumeric_id(32);

        let exists = match for_table {
            Table::Users => {
                sqlx::query_scalar!(
                    r#"SELECT EXISTS(SELECT 1 FROM users WHERE id = $1) as "exists!""#,
                    id
                )
                .fetch_one(&mut **tx)
                .await?
            }
            Table::ForumThreads => {
                sqlx::query_scalar!(
                    r#"SELECT EXISTS(SELECT 1 FROM forum_threads WHERE id = $1) as "exists!""#,
                    id
                )
                .fetch_one(&mut **tx)
                .await?
            }
            Table::ForumThreadMessages => sqlx::query_scalar!(
                r#"SELECT EXISTS(SELECT 1 FROM forum_thread_messages WHERE id = $1) as "exists!""#,
                id
            )
            .fetch_one(&mut **tx)
            .await?,
            Table::Forums => {
                sqlx::query_scalar!(
                    r#"SELECT EXISTS(SELECT 1 FROM forums WHERE id = $1) as "exists!""#,
                    id
                )
                .fetch_one(&mut **tx)
                .await?
            }
            Table::Invoices => {
                sqlx::query_scalar!(
                    r#"SELECT EXISTS(SELECT 1 FROM invoices WHERE id = $1) as "exists!""#,
                    id
                )
                .fetch_one(&mut **tx)
                .await?
            }
        };

        if !exists {
            return Ok(id);
        }
    }

    Err(anyhow!("Failed to generate a unique ID after 20 attempts",))
}
pub async fn generate_unique_user_id(tx: &mut Transaction<'_, Postgres>) -> anyhow::Result<String> {
    generate_unique_id_for(tx, Table::Users).await
}

pub async fn generate_unique_thread_id(
    tx: &mut Transaction<'_, Postgres>,
) -> anyhow::Result<String> {
    generate_unique_id_for(tx, Table::ForumThreads).await
}

pub async fn generate_unique_forum_id(
    tx: &mut Transaction<'_, Postgres>,
) -> anyhow::Result<String> {
    generate_unique_id_for(tx, Table::Forums).await
}

pub async fn generate_unique_forum_thread_message_id(
    tx: &mut Transaction<'_, Postgres>,
) -> anyhow::Result<String> {
    generate_unique_id_for(tx, Table::ForumThreadMessages).await
}
