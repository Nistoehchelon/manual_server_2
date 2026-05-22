use std::collections::HashMap;

use custom_site_variables::find_custom_site_variable;
use error_structs::SomeSortOfError;
use payment_system_structs::{CallbackHeaders, NewInvoiceRequest, PaymentInfo };
use sqlx::{Postgres, Transaction};
use webhooks::X_API_TOKEN;

#[allow(clippy::too_many_arguments)]
pub async fn new_invoice(
    tx: &mut Transaction<'_, Postgres>,
    invoice_id: String,
    amount_atomic: u64,
    method: payment_system_structs::PaymentMethod,
    success_callback: String,
    status_update_callback: String,
    cloudflare_bypass_under_attack_check_header_name: String,
    cloudflare_bypass_under_attack_check_header_value: String,
) -> Result<PaymentInfo, SomeSortOfError> {
    let payment_server_url = get_payment_server_url(tx).await?;
    let path = "new_invoice".to_string();
    let token = get_payment_server_token(tx).await?;
    let request_url = format!("{payment_server_url}/{path}");
    let mut callback_headers = HashMap::new();
    callback_headers.insert(cloudflare_bypass_under_attack_check_header_name.clone(), cloudflare_bypass_under_attack_check_header_value.clone());
    callback_headers.insert("x-api-token".to_string(), webhooks::get_token(&mut *tx).await.map_err(SomeSortOfError::new)?);
    
    let payload = NewInvoiceRequest {
        invoice_id,
        method,
        amount_atomic,
        success_callback,
        status_update_callback,
        callback_headers: CallbackHeaders(callback_headers)
    };
    let client = reqwest::Client::new();

    let response = client
        .post(&request_url)
        .header(X_API_TOKEN, token)
        .header(cloudflare_bypass_under_attack_check_header_name, cloudflare_bypass_under_attack_check_header_value)
        .json(&payload)
        .send()
        .await
        .map_err(|e| SomeSortOfError::new(e.to_string()))?;
 
    let payment_info: PaymentInfo = response
        .json()
        .await
        .map_err(|e| SomeSortOfError::new(e.to_string()))?;

    Ok(payment_info)
}

pub async fn get_status_update_callback_url(
    tx: &mut Transaction<'_, Postgres>,
) -> Result<String, SomeSortOfError> {
    use custom_site_variables::CustomSiteVariableName;
    let Some(val) = custom_site_variables::find_custom_site_variable(
        tx,
        CustomSiteVariableName::WebsiteRootUrl,
    )
    .await?
    else {
        return Err(SomeSortOfError::new("..."))?;
    };
    Ok(val)
}

pub async fn get_payment_server_url(
    tx: &mut Transaction<'_, Postgres>,
) -> Result<String, SomeSortOfError> {
    let opt = find_custom_site_variable(
        tx,
        custom_site_variables::CustomSiteVariableName::PaymentServerUrl,
    )
    .await?;
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
    let opt = find_custom_site_variable(
        tx,
        custom_site_variables::CustomSiteVariableName::PaymentServerToken,
    )
    .await?;
    match opt {
        Some(u) => Ok(u),
        None => Err(SomeSortOfError::new(
            "Could not find payment_server_token in site vars",
        )),
    }
}
