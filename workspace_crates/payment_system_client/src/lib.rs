use custom_site_variables::find_custom_site_variable;
use error_structs::SomeSortOfError;
use payment_system_structs::{NewInvoiceRequest, NewInvoiceResponse};
use sqlx::{Postgres, Transaction};

pub struct PaymentInfo {
    pub amount: Amount,
    pub address: Address,
    pub method: Method,
}
pub struct Address {
    inner: String,
}
impl Address {
    pub fn from(s: String) -> Self {
        Self {
            inner: s.to_string(),
        }
    }
}
impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}
pub struct Amount {
    inner: String,
}
impl Amount {
    pub fn from(s: String) -> Self {
        Self {
            inner: s.to_string(),
        }
    }
}
impl std::fmt::Display for Amount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}
pub struct Method {
    inner: String,
}
impl Method {
    pub fn from(s: String) -> Self {
        Self {
            inner: s.to_string(),
        }
    }
}
impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}


pub async fn new_invoice(
    tx: &mut Transaction<'_, Postgres>,
    invoice_id: String,
    amount_atomic: u64,
    method: String,
) -> Result<PaymentInfo, SomeSortOfError> {
    let success_callback = get_success_callback_url().await;
    let status_update_callback = get_status_update_callback_url().await;
    let payment_server_url = get_payment_server_url(tx).await?;
    let path = "new_invoice".to_string();
    let token = get_payment_server_token(tx).await?;
    let request_url = format!("{payment_server_url}/{path}");
    let payload = NewInvoiceRequest {
        invoice_id,
        method,
        amount_atomic,
        token,
        success_callback,
        status_update_callback,
    };
    let client = reqwest::Client::new();

    let response = client
        .post(&request_url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| SomeSortOfError::new(e.to_string()))?;

    let invoice_data: NewInvoiceResponse = response
        .json()
        .await
        .map_err(|e| SomeSortOfError::new(e.to_string()))?;

    Ok(PaymentInfo {
        amount: Amount::from(invoice_data.amount),
        address: Address::from(invoice_data.address),
        method: Method::from(invoice_data.method),
    })
}


pub async fn get_status_update_callback_url() -> String {
    "https://website.com/secret123/webhook/payment_status_update".to_string()
}

pub async fn get_success_callback_url() -> String {
    "https://website.com/secret123/webhook/payment_success".to_string()
}
pub async fn get_payment_server_url(
    tx: &mut Transaction<'_, Postgres>,
) -> Result<String, SomeSortOfError> {
    let opt = find_custom_site_variable(tx, custom_site_variables::CustomSiteVariableName::PaymentServerUrl).await?;
    match opt {
        Some(u) => Ok(u),
        None => Err(SomeSortOfError::new(
            "Could not find payment_server_url in site vars",
        )),
    }
}
pub async fn get_payment_server_token(
    tx: &mut Transaction<'_, Postgres>,
) -> Result<String, SomeSortOfError> {
    let opt = find_custom_site_variable(tx, custom_site_variables::CustomSiteVariableName::PaymentServerToken).await?;
    match opt {
        Some(u) => Ok(u),
        None => Err(SomeSortOfError::new(
            "Could not find payment_server_token in site vars",
        )),
    }
}
