use error_structs::SomeSortOfError;
use sqlx::{Postgres, Transaction};

pub async fn set_custom_site_variable(
    tx: &mut Transaction<'_, Postgres>,
    var_name: String,
    var_value: String,
) -> Result<(), SomeSortOfError> {
    sqlx::query!(
        r#"
        INSERT INTO custom_site_variables (name, value)
        VALUES ($1, $2)
        ON CONFLICT (name)
        DO UPDATE SET value = EXCLUDED.value
        "#,
        var_name,
        var_value,
    )
    .execute(&mut **tx)
    .await
    .map_err(SomeSortOfError::new)?;
    Ok(())
}
pub async fn find_custom_site_variable(
    tx: &mut Transaction<'_, Postgres>,
    var_name: String,
) -> Result<Option<String>, SomeSortOfError> {
    let row = sqlx::query!(
        r#"SELECT value FROM custom_site_variables WHERE name = $1"#,
        var_name
    )
    .fetch_optional(&mut **tx)
    .await
    .map_err(SomeSortOfError::new)?;
    if let Some(row) = row {
        let val = row.value;
        return Ok(Some(val));
    }
    Err(SomeSortOfError::new(format!(
        "Could not find custom_site_variable {var_name}"
    )))
}
