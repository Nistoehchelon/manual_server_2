use ::invoice::{PaymentMethod, ServiceData, create_invoice, utc_now};
use cryptocurrencies_stuff::atomic_unit::{AtomicUnit, CryptocurrencyConfig, LTC, XMR};
use error_structs::{InternalAppError, SomeSortOfError};
use http_stuff::{
    http_request_parsing::parse_body_into_string_form_input_params,
    http_response::{Charset, HttpResponseData, redirect_to},
};
use session::{add_session_notification_error, add_session_notification_success};

use crate::{RouterData, validate_turnstile};
use serde::Serialize;

#[derive(Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum PaymentStatusResponse {
    _Pending,
    Confirmations { current: u8, required: u8 },
    _Complete,
}

pub async fn get_status(_router_data: RouterData) -> Result<HttpResponseData, InternalAppError> {
    let content = serde_json::to_string(&PaymentStatusResponse::Confirmations {
        current: 5,
        required: 10,
    })
    .map_err(InternalAppError::new)?; // test random paymentstatusresponse, let's say confirmations current 5 out of 10
    let resp = http_stuff::http_response::ok(
        content.into(),
        http_stuff::http_response::ContentType::ApplicationJson,
        Some(Charset::Utf8),
    );
    Ok(resp)
}

pub async fn post_create_invoice(
    router_data: RouterData,
) -> Result<HttpResponseData, InternalAppError> {
    let RouterData {
        server_config,
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

    if server_config.cloudflare_turnstile_enabled
        && !validate_turnstile(
            body_params.clone(),
            server_config
                .cloudflare_turnstile_post_parameter_name
                .clone(),
            http_request_data.x_real_ip,
            server_config.cloudflare_turnstile_secret_key.clone(),
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

    let Some(months) = body_params.get("months").to_owned() else {
        tracing::error!("Invalid invoice Subscription months field");
        return Ok(redirect_to("/".to_string()));
    };
    let Ok(months) = months.parse::<u64>() else {
        tracing::error!("Invalid invoice Subscription months field");
        return Ok(redirect_to("/".to_string()));
    };

    let service = match service.as_str() {
        "subscription" => ServiceData::Subscription {
            expires_at: {
                let now = utc_now().map_err(InternalAppError::new)?;
                now + (months * 30 * 24 * 60)
            },
        },
        _ => {
            tracing::error!("Invalid invoice service");
            return Ok(redirect_to("/".to_string()));
        }
    };
    let method = match service {
        ServiceData::Subscription { .. } => match payment_method.as_str() {
            "ltc" => PaymentMethod::LTC {
                amount: get_by_usd::<LTC>(1.0 * (months as f64))
                    .await
                    .map_err(InternalAppError::new)?,
            },
            "xmr" => PaymentMethod::XMR {
                amount: get_by_usd::<XMR>(1.0 * (months as f64))
                    .await
                    .map_err(InternalAppError::new)?,
            },
            _ => {
                tracing::error!("Invalid invoice payment method");
                return Ok(redirect_to("/".to_string()));
            }
        },
    };

    let id = create_invoice(&mut tx, user.id, service, method)
        .await
        .map_err(InternalAppError::new)?;
    tx.commit().await.map_err(InternalAppError::new)?;
    add_session_notification_success(db_pool.clone(), session.id, "Invoice creation successful".to_string()).await;
    Ok(redirect_to(format!("/invoice/view?id={}", id.0)))
}

pub async fn get_by_usd<C: CryptocurrencyConfig>(
    usd: f64,
) -> Result<AtomicUnit<C>, SomeSortOfError> {
    let coin_price_usd = match C::SYMBOL {
        "LTC" => 55.0,
        "XMR" => 350.0,
        _ => return Err(SomeSortOfError::new("Unsupported crypto")),
    };

    let atomic = ((usd / coin_price_usd) * C::ATOMIC_PER_UNIT as f64).round() as u64;

    Ok(AtomicUnit::<C>::from_atomic(atomic))
}
