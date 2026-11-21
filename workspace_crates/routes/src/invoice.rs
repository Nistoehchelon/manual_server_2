use ::invoice::create_invoice;
use custom_site_variables::get_custom_site_variable;
use html_templating::{html_template_structs::ViewInvoiceTemplate, render};
use http_stuff::{
    http_request_parsing::parse_body_into_string_form_input_params,
    http_response::{Charset, HttpResponseData, ok_html, redirect_to},
};
use invoice::InvoiceId;
use session::{add_session_notification_error, add_session_notification_success};

use routing_structs::RouterData;

use crate::shop::possibly_create_and_get_user_cart_id;

pub enum PreliminaryProcessing {
    Forbid,
    Okay { invoice_id: String },
}
pub async fn invoice_viewing_preliminary_checks_and_processing(
    router_data: RouterData,
) -> anyhow::Result<PreliminaryProcessing> {
    let RouterData {
        site_config: _,
        db_pool,
        http_request_data,
        session,
        ..
    } = router_data.clone();
    let Some(invoice_id) = http_request_data.query.get("id").cloned() else {
        return Ok(PreliminaryProcessing::Forbid);
    };
    if !abac::user_can_view_invoice(db_pool.clone(), session.user_id.clone(), invoice_id.clone())
        .await?
    {
        tracing::error!(
            "User tried to view an invoice (not necessarily an invoice that actually exists) but were denied"
        );
        return Ok(PreliminaryProcessing::Forbid);
    }
    let Some(invoice_id) = http_request_data.query.get("id").cloned() else {
        return Ok(PreliminaryProcessing::Forbid);
    };
    if !abac::user_can_view_invoice(db_pool.clone(), session.user_id.clone(), invoice_id.clone())
        .await?
    {
        tracing::error!(
            "User tried to view an invoice (not necessarily an invoice that actually exists) but were denied"
        );
        return Ok(PreliminaryProcessing::Forbid);
    }
    Ok(PreliminaryProcessing::Okay { invoice_id })
}

pub async fn get_json_status(router_data: RouterData) -> anyhow::Result<HttpResponseData> {
    let RouterData { db_pool, .. } = router_data.clone();

    let invoice_id =
        match invoice_viewing_preliminary_checks_and_processing(router_data.clone()).await? {
            PreliminaryProcessing::Forbid => return Ok(redirect_to("/".to_string())),
            PreliminaryProcessing::Okay { invoice_id } => invoice_id,
        };
    let mut tx = db_pool.begin().await?;
    let payment_status = invoice::invoice_status(&mut tx, InvoiceId(invoice_id)).await?;
    let payment_status_json = serde_json::to_string(&payment_status)?;

    let resp = http_stuff::http_response::ok(
        payment_status_json.into(),
        http_stuff::http_response::ContentType::ApplicationJson,
        Some(Charset::Utf8),
    );
    Ok(resp)
}

pub async fn get_view(router_data: RouterData) -> anyhow::Result<HttpResponseData> {
    let RouterData { db_pool, .. } = router_data.clone();

    let invoice_id =
        match invoice_viewing_preliminary_checks_and_processing(router_data.clone()).await? {
            PreliminaryProcessing::Forbid => return Ok(redirect_to("/".to_string())),
            PreliminaryProcessing::Okay { invoice_id } => invoice_id,
        };

    // test
    let mut tx = db_pool.clone().begin().await?;
    let invoice::PaymentData {
        method,
        address,
        amount,
    } = invoice::get_payment_data(&mut tx, InvoiceId(invoice_id.clone())).await?;

    tx.commit().await?;
    Ok(ok_html(render(ViewInvoiceTemplate {
        main_info: router_data.base_template_data,
        amount,
        method,
        address,
        invoice_id,
    })))
}
#[tracing::instrument(skip_all)]
pub async fn invoice_creation_request(router_data: RouterData) -> anyhow::Result<HttpResponseData> {
    let RouterData {
        site_config,
        db_pool,
        http_request_data,
        session,
        ..
    } = router_data.clone();
    let mut tx = db_pool.clone().begin().await?;
    let Ok(user) = user::get_user_by_user_id(db_pool.clone(), session.user_id.clone()).await else {
        tracing::error!("Could not get user");
        return Ok(redirect_to("/".to_string()));
    };
    if !abac::user_can_create_invoice(db_pool.clone(), user.id.clone()).await? {
        tracing::error!("User can't create invoice");
        return Ok(redirect_to("/".to_string()));
    }

    let user_cart_id =
        possibly_create_and_get_user_cart_id(db_pool.clone(), user.id.clone()).await?;
    let body_params =
        parse_body_into_string_form_input_params(http_request_data.body_bytes.clone())
            .map_err(|e| anyhow::anyhow!("{e:#?}"))?;

    let payment_currency = body_params.get("payment_currency").cloned();
    let csrf_token = body_params.get("csrf_token").cloned();

    let (payment_currency, csrf_token) = match (payment_currency, csrf_token) {
        (Some(payment_method), Some(csrf_token)) => (payment_method, csrf_token),
        _ => {
            tracing::error!("Invalid body params {body_params:#?}");
            return Ok(redirect_to("/".to_string()));
        }
    };

    let csrf_token_valid =
        csrf_stuff::validate_token(db_pool.clone(), session.id.clone(), csrf_token.clone()).await?;
    if !csrf_token_valid {
        add_session_notification_error(
            db_pool.clone(),
            session.id.clone(),
            "csrf_token validation error".to_string(),
        )
        .await;
        return Ok(redirect_to("/signup".to_string()));
    }

    let payment_success_url = webhooks::get_endpoint_url(
        &mut tx,
        site_config.webhook_and_api_secret_path_prefix.clone(),
        webhooks::WebhookEndpoint::PaymentSuccess,
    )
    .await?;
    let payment_status_update_url = webhooks::get_endpoint_url(
        &mut tx,
        site_config.webhook_and_api_secret_path_prefix.clone(),
        webhooks::WebhookEndpoint::PaymentStatusUpdate,
    )
    .await?;
    let payment_server_url = get_custom_site_variable(
        &mut tx,
        custom_site_variables::CustomSiteVariableName::PaymentServerUrl,
    )
    .await?;
    let payment_server_token = get_custom_site_variable(
        &mut tx,
        custom_site_variables::CustomSiteVariableName::PaymentServerToken,
    )
    .await?;
    let invoice_data = create_invoice(
        &mut tx,
        user.id,
        user_cart_id,
        payment_currency.parse()?,
        payment_server_url,
        payment_server_token,
        payment_success_url,
        payment_status_update_url,
        router_data
            .site_config
            .payment_server_cloudflare_bypass_under_attack_check_header_name
            .clone(),
        router_data
            .site_config
            .payment_server_cloudflare_bypass_under_attack_check_header_token_value
            .clone(),
    )
    .await?;
    tx.commit().await?;

    add_session_notification_success(
        db_pool.clone(),
        session.id,
        "Invoice creation successful".to_string(),
    )
    .await;

    Ok(redirect_to(format!(
        "/invoice/view?id={}",
        invoice_data.0.0
    )))
}
