use cryptocurrencies_stuff::currencies::{
    CurrencyAtomicUnit, LTCAtomicUnit, XMRAtomicUnit, XNOAtomicUnit,
};
use id_generator_unique_in_db::Table;
use payment_system_structs::PaymentId;
use serde::Serialize;
use sqlx::{Postgres, Transaction};

#[derive(Clone)]
pub struct PaymentData {
    pub amount: String,
    pub address: String,
    pub method: String,
}
pub async fn get_payment_data(
    tx: &mut Transaction<'_, Postgres>,
    invoice_id: InvoiceId,
) -> anyhow::Result<PaymentData> {
    let amount = get_invoice_data(tx, invoice_id.clone(), "amount_from_payment_server").await?;
    let address = get_invoice_data(tx, invoice_id.clone(), "address_from_payment_server").await?;
    let method = get_invoice_data(tx, invoice_id, "method_from_payment_server").await?;
    let amount = match method.as_str() {
        "monero" => {
            XMRAtomicUnit::from_atomic(CurrencyAtomicUnit(amount.parse()?)).to_string_trimmed()
        }
        "litecoin" => {
            LTCAtomicUnit::from_atomic(CurrencyAtomicUnit(amount.parse()?)).to_string_trimmed()
        }
        "nano" => {
            XNOAtomicUnit::from_atomic(CurrencyAtomicUnit(amount.parse()?)).to_string_trimmed()
        }
        _ => "unk".to_string(),
    };
    Ok(PaymentData {
        amount,
        address,
        method,
    })
}

#[derive(Clone, Debug)]
pub enum ServiceData {
    Subscription { months: u64, expires_at: u64 },
    UsernameColor { color_code: String, expires_at: u64 },
}

#[derive(Clone, Debug)]
pub enum PaymentMethod {
    XMR { amount: XMRAtomicUnit },
    LTC { amount: LTCAtomicUnit },
    XNO { amount: XNOAtomicUnit },
}

impl std::fmt::Display for PaymentMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            PaymentMethod::XMR { .. } => "monero",
            PaymentMethod::LTC { .. } => "litecoin",
            PaymentMethod::XNO { .. } => "nano",
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
    pub fn try_from(s: &str) -> anyhow::Result<Self> {
        match s {
            "subscription" => Ok(Self::Subscription),
            _ => Err(anyhow::anyhow!("Parse error")),
        }
    }
}

pub async fn success(db_pool: sqlx::PgPool, payment_id: PaymentId) -> anyhow::Result<()> {
    let mut tx = db_pool.begin().await?;
    let invoice_id = sqlx::query!(
        "SELECT invoice_id FROM invoice_data WHERE name = $1 AND value = $2",
        "payment_id_from_payment_server",
        payment_id.to_string()
    )
    .fetch_one(&db_pool)
    .await?
    .invoice_id;
    let invoice_id = InvoiceId(invoice_id);
    let invoice = sqlx::query!("SELECT id FROM invoices WHERE id = $1", invoice_id.0)
        .fetch_optional(&mut *tx)
        .await?;
    match invoice {
        Some(_) => {}
        None => {
            return Err(anyhow::anyhow!("Invoice not found"));
        }
    }
    let service = get_invoice_data(&mut tx, invoice_id.clone(), "service").await?;
    let service = ValidService::try_from(&service)?;
    match service {
        ValidService::Subscription => {
            let subscription_expires_at =
                get_invoice_data(&mut tx, invoice_id.clone(), "subscription_expires_at")
                    .await?
                    .parse::<u64>()?;
            let user_id = get_invoice_data(&mut tx, invoice_id.clone(), "user_id").await?;
            user::subscription::set_expiry_time(&mut tx, &user_id, subscription_expires_at).await?;
        }
    }

    set_invoice_data(&mut tx, invoice_id, "status", "complete").await?;
    tx.commit().await?;
    let receipt_id = "32mem2em2mdmq";
    let seller_name = "Mberg";
    let seller_user_id = "dmsodqp";
    let buyer = "bob321";
    let buyer_user_id = "m12emq";
    let seller_pubkey_url = "website.com/my_pubkey.pem";
    let base64_qr = ""; // stores link to website.com/receipt?receipt_id=
    let datetime = "4th Jan 2199 12:32 UTC-0";
    let items = r###"
    1. Subscription for 6 months, effective from approximately the datetime of this receipt, Cost: 0.0001 XNO (6 USD)<br>
    2. Colored username (blue) for 6 months, effective from approximately the datetime of this receipt: 0.0002 XNO (12 USD)<br>
    "###;
    // generate invoice
    let currency = "XNO";
    let seller_website = "website.com";
    let on_chain_transaction_id = "nano_tx_id";
    let currency_total_amount = "0.0003";
    let reference_currency = "USD";
    let reference_currency_total_amount = "18.0";
    let reference_currency_per_currency = format!("60000 {reference_currency}");
    let _invoice_html = format!(
        r###"
    <div>
        Receipt id: {receipt_id}<br>
        Link to this receipt (accessible only if you're logged in): <img src="data:image/png;base64,{base64_qr}"<br>
        Date and time: {datetime}<br>
        Buyer: name {buyer}, buyer_user_id: {buyer_user_id}<br>
        Seller: name: {seller_name}, seller_user_id: {seller_user_id}, at {seller_website}<br>
        Seller pubkey location url: {seller_pubkey_url}<br>
        Seller receipt cryptographic signature: [the signature, as well as a guide on how to verify the signature, are available inside the receipt zip file in files receipt.signature and SIGNATURE_VERIFICATION_GUIDE.md, respectively]<br>
       
        Currency name: {currency}<br>
        Currency total: {currency_total_amount} {currency}<br>
        Reference currency name: {reference_currency}<br>
        Reference currency total amount: {reference_currency_total_amount} {reference_currency}<br>
        Reference currency per currency rate ({reference_currency} / {currency}) rate (approximate, according to the typical exchange rates available at datetime of this receipt) : {reference_currency_per_currency},
        Currency ({currency}) on chain transaction id: {on_chain_transaction_id}<br>
        Items purchased:<br>
        {items}<br>

    </div>
    "###,
    );

    Ok(())
}

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum InvoicePaymentStatus {
    Pending,
    Confirmations { current: u8, required: u8 },
    Complete,
}

use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub struct ParseInvoicePaymentStatusError(String);

impl std::fmt::Display for ParseInvoicePaymentStatusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid invoice payment status: {:?}", self.0)
    }
}

impl std::error::Error for ParseInvoicePaymentStatusError {}

impl FromStr for InvoicePaymentStatus {
    type Err = ParseInvoicePaymentStatusError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "pending" => Ok(Self::Pending),
            "complete" => Ok(Self::Complete),
            s => {
                // expects "confirmations x out of y"
                let nums: Option<(u8, u8)> = s
                    .strip_prefix("confirmations ")
                    .and_then(|rest| rest.split_once(" out of "))
                    .and_then(|(current, required)| {
                        Some((current.parse().ok()?, required.parse().ok()?))
                    });

                match nums {
                    Some((current, required)) => Ok(Self::Confirmations { current, required }),
                    None => Err(ParseInvoicePaymentStatusError(s.to_string())),
                }
            }
        }
    }
}

pub async fn invoice_status(
    tx: &mut Transaction<'_, Postgres>,
    invoice_id: InvoiceId,
) -> anyhow::Result<InvoicePaymentStatus> {
    let raw = get_invoice_data(tx, invoice_id, "status").await?;
    let status = raw.parse::<InvoicePaymentStatus>()?;
    Ok(status)
}

#[allow(clippy::too_many_arguments)]
pub async fn create_invoice(
    tx: &mut Transaction<'_, Postgres>,
    user_id: String,
    service: ServiceData,
    payment_method: PaymentMethod,
    payment_success_webhook_url: String,
    payment_status_update_webhook_url: String,
    cloudflare_bypass_under_attack_check_header_name: String,
    cloudflare_bypass_under_attack_check_header_value: String,
) -> anyhow::Result<(InvoiceId, PaymentMethod, ServiceData)> {
    let invoice_id_string =
        id_generator_unique_in_db::generate_unique_id_for(tx, Table::Invoices).await?;
    let invoice_id = InvoiceId(invoice_id_string.clone());
    sqlx::query!("INSERT INTO invoices (id) VALUES ($1)", invoice_id_string)
        .execute(&mut **tx)
        .await?;

    set_invoice_data(tx, invoice_id.clone(), "status", "pending").await?;
    set_invoice_data(tx, invoice_id.clone(), "user_id", &user_id).await?;
    set_invoice_data(
        tx,
        invoice_id.clone(),
        "payment_method",
        &payment_method.to_string(),
    )
    .await?;

    let payment_amount_atomic_units = match payment_method.clone() {
        PaymentMethod::XMR { amount } => amount.as_atomic(),
        PaymentMethod::LTC { amount } => amount.as_atomic(),
        PaymentMethod::XNO { amount } => amount.as_atomic(),
    };
    set_invoice_data(
        tx,
        invoice_id.clone(),
        "payment_amount_atomic_units",
        &payment_amount_atomic_units.to_string(),
    )
    .await?;

    const FIVE_MIN: u64 = 5 * 60;
    let invoice_expires_at = utc_now()? + FIVE_MIN;
    let invoice_expires_at_string: &String = &invoice_expires_at.to_string();
    set_invoice_data(
        tx,
        invoice_id.clone(),
        "invoice_expires_at",
        invoice_expires_at_string,
    )
    .await?;
    let curr_ts = &utc_now()?.to_string();
    set_invoice_data(tx, invoice_id.clone(), "invoice_created_at", curr_ts).await?;

    if let ServiceData::Subscription { months, expires_at } = service {
        let sub_expires_at = expires_at.to_string();
        set_invoice_data(tx, invoice_id.clone(), "service", "subscription").await?;
        set_invoice_data(
            tx,
            invoice_id.clone(),
            "subscription_months",
            &months.to_string(),
        )
        .await?;
        set_invoice_data(
            tx,
            invoice_id.clone(),
            "subscription_expires_at",
            &sub_expires_at,
        )
        .await?;
    }
    let invoice = (invoice_id.clone(), payment_method.clone(), service);

    let payment_info = payment_system_client::new_invoice(
        tx,
        invoice_id_string,
        payment_amount_atomic_units,
        payment_method.to_string().parse()?,
        payment_success_webhook_url,
        payment_status_update_webhook_url,
        cloudflare_bypass_under_attack_check_header_name,
        cloudflare_bypass_under_attack_check_header_value,
    )
    .await?;

    set_invoice_data(
        tx,
        invoice_id.clone(),
        "payment_id_from_payment_server",
        &payment_info.payment_id.to_string(),
    )
    .await?;
    set_invoice_data(
        tx,
        invoice_id.clone(),
        "address_from_payment_server",
        &payment_info.address.to_string(),
    )
    .await?;
    set_invoice_data(
        tx,
        invoice_id.clone(),
        "amount_from_payment_server",
        &payment_info.amount.to_string(),
    )
    .await?;
    set_invoice_data(
        tx,
        invoice_id.clone(),
        "method_from_payment_server",
        &payment_info.method.to_string(),
    )
    .await?;
    Ok(invoice)
}

pub async fn get_invoice_data(
    tx: &mut Transaction<'_, Postgres>,
    invoice_id: InvoiceId,
    data_name: &str,
) -> anyhow::Result<String> {
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
    .await?;

    Ok(row.value)
}

pub async fn set_invoice_data(
    tx: &mut Transaction<'_, Postgres>,
    invoice_id: InvoiceId,
    name: &str,
    value: &str,
) -> anyhow::Result<()> {
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
    .await?;

    Ok(())
}

pub fn utc_now() -> anyhow::Result<u64> {
    let now = chrono::Utc::now().timestamp();
    let now = u64::try_from(now)?;
    Ok(now)
}
