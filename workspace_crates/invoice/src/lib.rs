use anyhow::{ Result};
use cryptocurrencies_stuff::currencies::{
    CurrencyAtomicUnit, LTC, LTCAtomicUnit, XMR, XMRAtomicUnit, XNO, XNOAtomicUnit,
};
use financial_stuff::get_by_usd;
use id_generator_unique_in_db::Table;
use payment_system_structs::{PaymentId, PaymentMethod};
use serde::Serialize;
use shop::{ShopService, calculate_service_item_usd_price};
use sqlx::{Postgres, Transaction};
use time_funcs::{add_months_to_timestamp, utc_now};
use user::set_user_data;

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
pub enum PaymentMethodAmount {
    XMR { amount: XMRAtomicUnit },
    LTC { amount: LTCAtomicUnit },
    XNO { amount: XNOAtomicUnit },
}

impl std::fmt::Display for PaymentMethodAmount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            PaymentMethodAmount::XMR { .. } => "monero",
            PaymentMethodAmount::LTC { .. } => "litecoin",
            PaymentMethodAmount::XNO { .. } => "nano",
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone)]
pub struct InvoiceId(pub String);

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
    let invoice_items = get_invoice_items(&mut tx, invoice_id.0.clone()).await?;
    for item in invoice_items {
        let service = item.name.parse::<ShopService>()?;
        match service {
            ShopService::Subscription => {
                let subscription_months =
                    get_invoice_item_data(&mut tx, "months".to_owned(), item.id.clone())
                        .await?
                        .parse::<u64>()?;
                let user_id = get_invoice_data(&mut tx, invoice_id.clone(), "user_id").await?;
                let subscription_expires_at =
                    add_months_to_timestamp(utc_now()?, subscription_months)?;
                user::subscription::set_expiry_time(&mut tx, &user_id, subscription_expires_at)
                    .await?;
            }
            ShopService::UsernameColor => {
                let service_months =
                    get_invoice_item_data(&mut tx, "months".to_owned(), item.id.clone())
                        .await?
                        .parse::<u64>()?;
                let color_code =
                    get_invoice_item_data(&mut tx, "color_code".to_owned(), item.id.clone())
                        .await?
                        .parse::<csscolorparser::Color>()?;
                let user_id = get_invoice_data(&mut tx, invoice_id.clone(), "user_id").await?;
                let color_expires_at = add_months_to_timestamp(utc_now()?, service_months)?;
                set_user_data(
                    &mut tx,
                    &user_id,
                    "username_color_code",
                    &color_code.to_string(),
                )
                .await?;
                set_user_data(
                    &mut tx,
                    &user_id,
                    "username_color_expires_at",
                    &color_expires_at.to_string(),
                )
                .await?;
            }
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
    cart_id: String,
    payment_method: payment_system_structs::PaymentMethod,
    payment_server_url: String,
    payment_server_token: String,
    payment_success_webhook_url: String,
    payment_status_update_webhook_url: String,
    cloudflare_bypass_under_attack_check_header_name: String,
    cloudflare_bypass_under_attack_check_header_value: String,
) -> anyhow::Result<(InvoiceId, PaymentMethod)> {
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

    let items = sqlx::query!(
        "SELECT id, name FROM shopping_cart_items WHERE shopping_cart_id = $1",
        cart_id
    )
    .fetch_all(&mut **tx)
    .await?;
    let mut total_usd = 0.0;
    for item in items {
        let item_id = item.id;
        let price_usd = calculate_service_item_usd_price(tx, item_id.clone()).await?;
        total_usd += price_usd;
        let payment_method_currency_price_atomic_unit = match payment_method {
            payment_system_structs::PaymentMethod::Monero => {
                get_by_usd::<XMR>(tx, &price_usd.to_string())
                    .await?
                    .as_atomic()
            }
            payment_system_structs::PaymentMethod::Litecoin => {
                get_by_usd::<LTC>(tx, &price_usd.to_string())
                    .await?
                    .as_atomic()
            }
            payment_system_structs::PaymentMethod::Nano => {
                get_by_usd::<XNO>(tx, &price_usd.to_string())
                    .await?
                    .as_atomic()
            }
        }
        .to_string();
        sqlx::query!("INSERT INTO shopping_cart_item_data (id, name, value, shopping_cart_item_id) VALUES (uuidv7()::text, $1, $2, $3)", "payment_method_atomic_unit_price", payment_method_currency_price_atomic_unit, &item_id).execute(&mut **tx).await?;
        sqlx::query!("INSERT INTO shopping_cart_item_data (id, name, value, shopping_cart_item_id) VALUES (uuidv7()::text, $1, $2, $3)", "usd_price", price_usd.to_string(), &item_id).execute(&mut **tx).await?;
    }
    let total_payment_amount_atomic_units = match payment_method {
        payment_system_structs::PaymentMethod::Monero => {
            get_by_usd::<XMR>(tx, &total_usd.to_string())
                .await?
                .as_atomic()
        }
        payment_system_structs::PaymentMethod::Litecoin => {
            get_by_usd::<LTC>(tx, &total_usd.to_string())
                .await?
                .as_atomic()
        }
        payment_system_structs::PaymentMethod::Nano => {
            get_by_usd::<XNO>(tx, &total_usd.to_string())
                .await?
                .as_atomic()
        }
    };
    set_invoice_data(
        tx,
        invoice_id.clone(),
        "payment_amount_atomic_units",
        &total_payment_amount_atomic_units.to_string(),
    )
    .await?;

    move_shopping_cart_items_into_invoice(invoice_id.clone(), cart_id.clone(), tx).await?;

    let payment_info = payment_system_client::new_invoice(
        tx,
        invoice_id.0.clone(),
        total_payment_amount_atomic_units,
        payment_method.to_string().parse()?,
        payment_server_url,
        payment_server_token,
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
    todo!()
}

pub async fn move_shopping_cart_items_into_invoice(
    invoice_id: InvoiceId,
    cart_id: String,
    tx: &mut Transaction<'_, Postgres>,
) -> anyhow::Result<()> {
    sqlx::query!(
        r###"
        WITH source_items AS (
            SELECT 
                id AS old_item_id,
                uuidv7()::text AS new_item_id,
                name
            FROM shopping_cart_items
            WHERE shopping_cart_id = $2
        ),
        inserted_items AS (
            INSERT INTO invoice_cart_items (id, invoice_id, name)
            SELECT new_item_id, $1, name
            FROM source_items
            RETURNING id
        ),
        inserted_item_data AS (
            INSERT INTO invoice_cart_item_data (id, invoice_cart_item_id, name, value)
            SELECT 
                uuidv7()::text,
                si.new_item_id,
                scid.name,
                scid.value
            FROM shopping_cart_item_data scid
            JOIN source_items si ON scid.shopping_cart_item_id = si.old_item_id
            RETURNING id
        )
        DELETE FROM shopping_cart_items
        WHERE id IN (SELECT old_item_id FROM source_items)
    "###,
        invoice_id.0,
        cart_id
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub struct InvoiceItems {
    id: String,
    name: String,
}
pub async fn get_invoice_items(
    tx: &mut Transaction<'_, Postgres>,
    invoice_id: String,
) -> anyhow::Result<Vec<InvoiceItems>> {
    let items = sqlx::query_as!(
        InvoiceItems,
        "SELECT id, name FROM invoice_cart_items WHERE invoice_id = $1",
        invoice_id
    )
    .fetch_all(&mut **tx)
    .await?;
    Ok(items)
}

pub async fn get_invoice_item_data(
    tx: &mut Transaction<'_, Postgres>,
    item_name: String,
    invoice_item_id: String,
) -> anyhow::Result<String> {
    Ok(sqlx::query!(r###"SELECT value FROM invoice_cart_item_data WHERE name = $1 AND invoice_cart_item_id = $2"###, item_name, invoice_item_id).fetch_one(&mut **tx).await?.value)
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
