use sqlx::PgConnection;

pub async fn find_receipt_id_by_invoice_id(
    conn: &mut PgConnection,
    invoice_id: String,
) -> anyhow::Result<String> {
    Ok(
        sqlx::query!("SELECT id FROM receipts WHERE invoice_id = $1", invoice_id)
            .fetch_one(conn)
            .await?
            .id,
    )
}
