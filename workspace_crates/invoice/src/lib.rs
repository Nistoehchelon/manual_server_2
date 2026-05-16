use cryptocurrencies_stuff::atomic_unit::{LTCAtomicUnit, XMRAtomicUnit};
use error_structs::SomeSortOfError;
use id_generator_unique_in_db::Table;
use sqlx::{Postgres, Transaction};

pub enum ServiceData {
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

#[derive(Clone)]
pub struct InvoiceId(pub String);

pub enum ValidService {
    Subscription,
}
impl ValidService {
    pub fn try_from(s: &str) -> Result<Self, SomeSortOfError> {
        match s {
            "subscription" => Ok(Self::Subscription),
            _ => Err(SomeSortOfError::new("Parse error")),
        }
    }
}

pub async fn success(db_pool: sqlx::PgPool, invoice_id: InvoiceId) -> Result<(), SomeSortOfError> {
    let mut tx = db_pool.begin().await.map_err(SomeSortOfError::new)?;
    let invoice = sqlx::query!("SELECT id FROM invoices WHERE id = $1", invoice_id.0)
        .fetch_optional(&mut *tx)
        .await
        .map_err(SomeSortOfError::new)?;
    match invoice {
        Some(_) => {}
        None => {
            return Err(SomeSortOfError::new("Invoice not found"));
        }
    }
    let service = get_invoice_data(&mut tx, invoice_id.clone(), "service")
        .await
        .map_err(SomeSortOfError::new)?;
    let service = ValidService::try_from(&service).map_err(SomeSortOfError::new)?;
    match service {
        ValidService::Subscription => {
            let subscription_expires_at =
                get_invoice_data(&mut tx, invoice_id.clone(), "subscription_expires_at")
                    .await
                    .map_err(SomeSortOfError::new)?
                    .parse::<u64>()
                    .map_err(SomeSortOfError::new)?;
            let user_id = get_invoice_data(&mut tx, invoice_id.clone(), "user_id")
                .await
                .map_err(SomeSortOfError::new)?;
            user::subscription::set_expiry_time(&mut tx, &user_id, subscription_expires_at).await?;
        }
    }
    set_invoice_data(&mut tx, invoice_id, "status", "complete")
        .await
        .map_err(SomeSortOfError::new)?;
    tx.commit().await.map_err(SomeSortOfError::new)?;
    Ok(())
}

pub async fn invoice_status(
    _tx: &mut Transaction<'_, Postgres>,
    _invoice_id: InvoiceId,
) -> Result<String, SomeSortOfError> {
    todo!();
}

pub async fn create_invoice(
    tx: &mut Transaction<'_, Postgres>,
    user_id: String,
    service: ServiceData,
    payment_method: PaymentMethod,
) -> Result<InvoiceId, SomeSortOfError> {
    let invoice_id_string = id_generator_unique_in_db::generate_unique_id_for(tx, Table::Invoices)
        .await
        .map_err(SomeSortOfError::new)?;
    let invoice_id = InvoiceId(invoice_id_string.clone());
    sqlx::query!("INSERT INTO invoices (id) VALUES ($1)", invoice_id_string)
        .execute(&mut **tx)
        .await
        .map_err(SomeSortOfError::new)?;

    set_invoice_data(tx, invoice_id.clone(), "status", "pending")
        .await
        .map_err(SomeSortOfError::new)?;
    set_invoice_data(tx, invoice_id.clone(), "user_id", &user_id)
        .await
        .map_err(SomeSortOfError::new)?;
    set_invoice_data(
        tx,
        invoice_id.clone(),
        "payment_method",
        &payment_method.to_string(),
    )
    .await
    .map_err(SomeSortOfError::new)?;

    let payment_amount_atomic_units = match payment_method {
        PaymentMethod::XMR { amount } => amount.as_atomic(),
        PaymentMethod::LTC { amount } => amount.as_atomic(),
    };
    set_invoice_data(
        tx,
        invoice_id.clone(),
        "payment_amount_atomic_units",
        &payment_amount_atomic_units.to_string(),
    )
    .await
    .map_err(SomeSortOfError::new)?;

    const FIVE_MIN: u64 = 5 * 60;
    let invoice_expires_at = utc_now().map_err(SomeSortOfError::new)? + FIVE_MIN;
    let invoice_expires_at_string: &String = &invoice_expires_at.to_string();
    set_invoice_data(
        tx,
        invoice_id.clone(),
        "invoice_expires_at",
        invoice_expires_at_string,
    )
    .await
    .map_err(SomeSortOfError::new)?;
    let curr_ts = &utc_now().map_err(SomeSortOfError::new)?.to_string();
    set_invoice_data(tx, invoice_id.clone(), "invoice_created_at", curr_ts)
        .await
        .map_err(SomeSortOfError::new)?;

    match service {
        ServiceData::Subscription { expires_at } => {
            let sub_expires_at = expires_at.to_string();
            set_invoice_data(tx, invoice_id.clone(), "service", "subscription")
                .await
                .map_err(SomeSortOfError::new)?;
            set_invoice_data(
                tx,
                invoice_id.clone(),
                "subscription_expires_at",
                &sub_expires_at,
            )
            .await
            .map_err(SomeSortOfError::new)?;
        }
    }
    Ok(invoice_id)
}

pub async fn get_invoice_data(
    tx: &mut Transaction<'_, Postgres>,
    invoice_id: InvoiceId,
    data_name: &str,
) -> Result<String, SomeSortOfError> {
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
    .fetch_one(&mut **tx)
    .await
    .map_err(SomeSortOfError::new)?;

    Ok(row.value)
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

pub fn utc_now() -> Result<u64, SomeSortOfError> {
    let now = chrono::Utc::now().timestamp();
    let now = u64::try_from(now).map_err(SomeSortOfError::new)?;
    Ok(now)
}
