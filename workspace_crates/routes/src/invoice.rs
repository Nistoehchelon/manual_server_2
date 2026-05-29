use cryptocurrencies_stuff::currencies::{LTC, XMR};
use ::invoice::{PaymentMethod, ServiceData, create_invoice, utc_now};
use error_structs::InternalAppError;
use financial_stuff::get_by_usd;
use html_templating::{html_template_structs::ViewInvoiceTemplate, render};
use http_stuff::{
    http_request_parsing::parse_body_into_string_form_input_params,
    http_response::{Charset, HttpResponseData, ok_html, redirect_to},
};
use invoice::InvoiceId;
use session::{add_session_notification_error, add_session_notification_success};

use routing_structs::RouterData;

pub enum PreliminaryProcessing {
    Forbid,
    Okay { invoice_id: String },
}
pub async fn invoice_viewing_preliminary_checks_and_processing(
    router_data: RouterData,
) -> Result<PreliminaryProcessing, InternalAppError> {
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
        .await
        .map_err(InternalAppError::new)?
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
        .await
        .map_err(InternalAppError::new)?
    {
        tracing::error!(
            "User tried to view an invoice (not necessarily an invoice that actually exists) but were denied"
        );
        return Ok(PreliminaryProcessing::Forbid);
    }
    Ok(PreliminaryProcessing::Okay { invoice_id })
}

pub async fn get_json_status(
    router_data: RouterData,
) -> Result<HttpResponseData, InternalAppError> {
    let RouterData { db_pool, .. } = router_data.clone();

    let invoice_id =
        match invoice_viewing_preliminary_checks_and_processing(router_data.clone()).await? {
            PreliminaryProcessing::Forbid => return Ok(redirect_to("/".to_string())),
            PreliminaryProcessing::Okay { invoice_id } => invoice_id,
        };
    let mut tx = db_pool.begin().await.map_err(InternalAppError::new)?;
    let payment_status = invoice::invoice_status(&mut tx, InvoiceId(invoice_id))
        .await
        .map_err(InternalAppError::new)?;
    let payment_status_json =
        serde_json::to_string(&payment_status).map_err(InternalAppError::new)?;

    let resp = http_stuff::http_response::ok(
        payment_status_json.into(),
        http_stuff::http_response::ContentType::ApplicationJson,
        Some(Charset::Utf8),
    );
    Ok(resp)
}

pub async fn get_view(router_data: RouterData) -> Result<HttpResponseData, InternalAppError> {
    let RouterData { db_pool, .. } = router_data.clone();

    let invoice_id =
        match invoice_viewing_preliminary_checks_and_processing(router_data.clone()).await? {
            PreliminaryProcessing::Forbid => return Ok(redirect_to("/".to_string())),
            PreliminaryProcessing::Okay { invoice_id } => invoice_id,
        };

    // test
    let mut tx = db_pool
        .clone()
        .begin()
        .await
        .map_err(InternalAppError::new)?;
    let invoice::PaymentData {
        method,
        address,
        amount,
    } = invoice::get_payment_data(&mut tx, InvoiceId(invoice_id.clone()))
        .await
        .map_err(InternalAppError::new)?;
    
    tx.commit().await.map_err(InternalAppError::new)?;
    Ok(ok_html(render(ViewInvoiceTemplate {
        main_info: router_data.base_template_data,
        amount,
        method,
        address,
        invoice_id,
    })))
}
pub async fn post_create_invoice(
    router_data: RouterData,
) -> Result<HttpResponseData, InternalAppError> {
    let RouterData {
        site_config,
        db_pool,
        http_request_data,
        session,
        ..
    } = router_data.clone();
    let mut tx = db_pool
        .clone()
        .begin()
        .await
        .map_err(InternalAppError::new)?;
    let Ok(user) = user::get_user_by_user_id(db_pool.clone(), session.user_id.clone()).await else {
        return Ok(redirect_to("/".to_string()));
    };
    if !abac::user_can_create_invoice(db_pool.clone(), user.id.clone())
        .await
        .map_err(InternalAppError::new)?
    {
        return Ok(redirect_to("/".to_string()));
    }

    let body_params =
        parse_body_into_string_form_input_params(http_request_data.body_bytes.clone())
            .map_err(InternalAppError::new)?;

    if site_config.cloudflare_turnstile_enabled
        && !crate::validate_turnstile(
            body_params.clone(),
            site_config.cloudflare_turnstile_post_parameter_name.clone(),
            http_request_data.x_real_ip,
            site_config.cloudflare_turnstile_secret_key.clone(),
        )
        .await
        .map_err(InternalAppError::new)?
    {
        add_session_notification_error(
            db_pool.clone(),
            session.id.clone(),
            String::from("Could not validate Turnstile Captcha"),
        )
        .await;
        return Ok(redirect_to("/signup".to_string()));
    }
    let service = body_params.get("service").cloned();
    let payment_method = body_params.get("payment_method").cloned();
    let csrf_token = body_params.get("csrf_token").cloned();

    let (service, payment_method, csrf_token) = match (service, payment_method, csrf_token) {
        (Some(service), Some(payment_method), Some(csrf_token)) => {
            (service, payment_method, csrf_token)
        }
        _ => {
            return Ok(redirect_to("/".to_string()));
        }
    };

    let csrf_token_valid =
        csrf_stuff::validate_token(db_pool.clone(), session.id.clone(), csrf_token.clone())
            .await
            .map_err(InternalAppError::new)?;
    if !csrf_token_valid {
        add_session_notification_error(
            db_pool.clone(),
            session.id.clone(),
            "csrf_token validation error".to_string(),
        )
        .await;
        return Ok(redirect_to("/signup".to_string()));
    }

    let service = match service.as_str() {
        "subscription" => {
            let Some(months) = body_params.get("months").to_owned() else {
                tracing::error!("Invalid invoice Subscription months field");
                return Ok(redirect_to("/".to_string()));
            };
            let Ok(months) = months.parse::<u64>() else {
                tracing::error!("Invalid invoice Subscription months field");
                return Ok(redirect_to("/".to_string()));
            };
            ServiceData::Subscription {
                months,
                expires_at: {
                    let now = utc_now().map_err(InternalAppError::new)?;
                    now + (months * 30 * 24 * 60)
                },
            }
        }
        _ => {
            tracing::error!("Invalid invoice service");
            return Ok(redirect_to("/".to_string()));
        }
    };

    let method = match service {
        ServiceData::Subscription { months, .. } => {
            let Some(sub_cost) = custom_site_variables::find_custom_site_variable(
                &mut tx,
                custom_site_variables::CustomSiteVariableName::SubscriptionCostUsd,
            )
            .await
            .map_err(InternalAppError::new)?
            else {
                return Err(InternalAppError::new("No usd cost custom site var set"));
            };
            let sub_cost: f64 = sub_cost.parse().map_err(InternalAppError::new)?;

            
            match payment_method.as_str() {
                "litecoin" => PaymentMethod::LTC {
                    amount: get_by_usd::<LTC>(&mut tx, sub_cost * (months as f64))
                        .await
                        .map_err(InternalAppError::new)?,
                },
                "monero" => PaymentMethod::XMR {
                    amount: get_by_usd::<XMR>(&mut tx, sub_cost * (months as f64))
                        .await
                        .map_err(InternalAppError::new)?,
                },
                _ => {
                    tracing::error!("Invalid invoice payment method");
                    return Ok(redirect_to("/".to_string()));
                }
            }
        }
    };
    let payment_success_url = webhooks::get_endpoint_url(
        &mut tx,
        site_config.webhook_and_api_secret_path_prefix.clone(),
        webhooks::WebhookEndpoint::PaymentSuccess,
    )
    .await
    .map_err(InternalAppError::new)?;
    let payment_status_update_url = webhooks::get_endpoint_url(
        &mut tx,
        site_config.webhook_and_api_secret_path_prefix.clone(),
        webhooks::WebhookEndpoint::PaymentStatusUpdate,
    )
    .await
    .map_err(InternalAppError::new)?;
    let invoice_data = create_invoice(
        &mut tx,
        user.id,
        service,
        method,
        payment_success_url,
        payment_status_update_url,
        router_data.server_config.cloudflare_bypass_under_attack_check_header_name.clone(),
        router_data.server_config.cloudflare_bypass_under_attack_check_header_token_value.clone(),
        
    )
    .await
    .map_err(InternalAppError::new)?;
    tx.commit().await.map_err(InternalAppError::new)?;
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
