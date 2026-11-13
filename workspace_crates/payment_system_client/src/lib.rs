use std::collections::HashMap;

use anyhow::anyhow;
use custom_site_variables::find_custom_site_variable;
use payment_system_structs::{CallbackHeaders, NewInvoiceRequest, PaymentInfo};
use sqlx::{Postgres, Transaction};
use webhooks::X_API_TOKEN;

#[allow(clippy::too_many_arguments)]
pub async fn new_invoice(
    tx: &mut Transaction<'_, Postgres>,
    _invoice_id: String,
    amount_atomic: u64,
    method: payment_system_structs::PaymentMethod,
    success_callback_url: String,
    status_update_callback_url: String,
    cloudflare_bypass_under_attack_check_header_name: String,
    cloudflare_bypass_under_attack_check_header_value: String,
) -> anyhow::Result<PaymentInfo> {
    let payment_server_url = get_payment_server_url(tx).await?;
    let path = "new_invoice".to_string();
    let token = get_payment_server_token(tx).await?;
    let request_url = format!("{payment_server_url}/{path}");
    let mut callback_headers = HashMap::new();
    callback_headers.insert(
        cloudflare_bypass_under_attack_check_header_name.clone(),
        cloudflare_bypass_under_attack_check_header_value.clone(),
    );
    callback_headers.insert(
        "x-api-token".to_string(),
        webhooks::get_token(&mut *tx)
            .await?,
    );

    let payload = NewInvoiceRequest {
        method,
        amount_atomic,
        callback_headers: CallbackHeaders(callback_headers),
        success_callback_url,
        status_update_callback_url,
    };
    let client = reqwest::Client::new();

    let response = client
        .post(&request_url)
        .header(X_API_TOKEN, token)
        .header(
            cloudflare_bypass_under_attack_check_header_name,
            cloudflare_bypass_under_attack_check_header_value,
        )
        .json(&payload)
        .send()
        .await?;

    let payment_info: PaymentInfo = response
        .json()
        .await?;

    Ok(payment_info)
}

pub async fn get_status_update_callback_url(
    tx: &mut Transaction<'_, Postgres>,
) -> anyhow::Result<String> {
    use custom_site_variables::CustomSiteVariableName;
    let Some(val) = custom_site_variables::find_custom_site_variable(
        tx,
        CustomSiteVariableName::WebsiteRootUrl,
    )
    .await?
    else {
        return Err(anyhow!("..."))?;
    };
    Ok(val)
}

pub async fn get_payment_server_url(
    tx: &mut Transaction<'_, Postgres>,
) -> anyhow::Result<String> {
    let opt = find_custom_site_variable(
        tx,
        custom_site_variables::CustomSiteVariableName::PaymentServerUrl,
    )
    .await?;
    match opt {
        Some(u) => Ok(u),
        None => Err(anyhow!(
            "Could not find payment_server_url in site vars",
        )),
    }
}
pub async fn get_payment_server_token(
    tx: &mut Transaction<'_, Postgres>,
) -> anyhow::Result<String> {
    let opt = find_custom_site_variable(
        tx,
        custom_site_variables::CustomSiteVariableName::PaymentServerToken,
    )
    .await?;
    match opt {
        Some(u) => Ok(u),
        None => Err(anyhow::anyhow!(
            "Could not find payment_server_token in site vars",
        )),
    }
}
