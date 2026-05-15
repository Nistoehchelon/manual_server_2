use cryptocurrencies_stuff::atomic_unit::{LTCAtomicUnit, XMRAtomicUnit};
use error_structs::SomeSortOfError;
use id_generator_unique_in_db::Table;
use sqlx::{Postgres, Transaction};

pub enum PaymentService {
    Subscription { expires_at: u64 },
}

pub enum PaymentMethod {
    XMR { amount: XMRAtomicUnit },
    LTC { amount: LTCAtomicUnit },
}

impl std::fmt::Display for PaymentMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            PaymentMethod::XMR { .. } => "ltc",
            PaymentMethod::LTC { .. } => "xmr",
        };
        write!(f, "{}", s)
    }
}

pub struct InvoiceId(pub String);

pub async fn create_invoice(
    tx: &mut Transaction<'_, Postgres>,
    user_id: String,
    service: PaymentService,
    method: PaymentMethod,
) -> Result<InvoiceId, SomeSortOfError> {
    let invoice_id = id_generator_unique_in_db::generate_unique_id_for(tx, Table::Invoices)
        .await
        .map_err(SomeSortOfError::new)?;
    sqlx::query!("INSERT INTO invoices (id) VALUES ($1)", invoice_id)
        .execute(&mut **tx)
        .await
        .map_err(SomeSortOfError::new)?;

    sqlx::query!(
        "INSERT INTO invoice_data (invoice_id, name, value) VALUES ($1, $2, $3)",
        invoice_id,
        "user_id",
        user_id.clone()
    )
    .execute(&mut **tx)
    .await
    .map_err(SomeSortOfError::new)?;

    sqlx::query!(
        "INSERT INTO invoice_data (invoice_id, name, value) VALUES ($1,$2,$3)",
        invoice_id,
        "payment_method",
        method.to_string()
    )
    .execute(&mut **tx)
    .await
    .map_err(SomeSortOfError::new)?;

    match service {
        PaymentService::Subscription { expires_at } => {
            let expires_at = expires_at.to_string();
            sqlx::query!(
                r#"INSERT INTO invoice_data (invoice_id, name, value) VALUES ($1, $2, $3)"#,
                invoice_id,
                "subscription_will_be_valid_until",
                expires_at
            )
            .execute(&mut **tx)
            .await
            .map_err(SomeSortOfError::new)?;
        }
    }
    Ok(InvoiceId(invoice_id))
}



pub async fn get_invoice_data(
    tx: &mut Transaction<'_, Postgres>,
    invoice_id: InvoiceId,
    data_name: &str,
) -> Result<Option<String>, SomeSortOfError> {
    let row = sqlx::query!(
        r#"
        SELECT value
        FROM invoice_data
        WHERE invoice_id = $1
          AND name = $2
        "#,
        invoice_id.0,
        data_name
    )
    .fetch_optional(&mut **tx)
    .await
    .map_err(SomeSortOfError::new)?;

    Ok(row.map(|r| r.value))
}

pub async fn set_invoice_data(
    tx: &mut Transaction<'_, Postgres>,
    invoice_id: InvoiceId,
    name: &str,
    value: &str,
) -> Result<(), SomeSortOfError> {
    sqlx::query!(
        r#"
        INSERT INTO invoice_data (
            invoice_id,
            name,
            value
        )
        VALUES (
            $1,
            $2,
            $3
        )
        ON CONFLICT (invoice_id, name)
        DO UPDATE SET
            value = EXCLUDED.value
        "#,
        invoice_id.0,
        name,
        value
    )
    .execute(&mut **tx)
    .await
    .map_err(SomeSortOfError::new)?;

    Ok(())
}
