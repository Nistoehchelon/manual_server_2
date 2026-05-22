use std::collections::HashMap;

use cryptocurrencies_stuff::atomic_unit::XMRAtomicUnit;
use error_structs::InternalAppError;
use http_stuff::{
    http_request_parsing::parse_body_into_string_form_input_params,
    http_response::{Charset, HttpResponseData, ok},
};

use payment_system_structs::PaymentId;
use routing_structs::RouterData;

pub async fn post(rt: RouterData) -> Result<HttpResponseData, InternalAppError> {
    println!("Parsing body...");
    let body_params =
        parse_body_into_string_form_input_params(rt.http_request_data.body_bytes.clone())
            .map_err(InternalAppError::new)?;
    let token = body_params
        .get("token")
        .map(|s| s.as_str())
        .unwrap_or("check_headers")
        .to_string();
    let token = if token.as_str() == "check_headers" {
        let tin = rt
            .http_request_data
            .headers
            .get("x-api-token")
            .unwrap_or("nope");
        tin.to_string()
    } else {
        token
    };
    let validity_of_the_token = webhooks::validate_token(rt.db_pool.clone(), token.clone())
        .await
        .map_err(InternalAppError::new)?;
    if !validity_of_the_token {
        let resp_msg = format!("Invalid token {token} provided");
        tracing::warn!("{resp_msg}");
        let mut resp = ok(
            resp_msg.into_bytes(),
            http_stuff::http_response::ContentType::TextPlain,
            Some(Charset::Utf8),
        );
        resp.status_code = http_stuff::http_response::StatusCode::Code200Ok;
        return Ok(resp);
    }

    let parts: Vec<&str> = rt
        .http_request_data
        .path
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();

    match parts.as_slice() {
        [_secret, _call_type, action_part] => {
            let action = action_part.to_string();
            call_action(rt.db_pool, action, body_params).await
        }
        _ => Err(InternalAppError::new("Invalid webhook/api parts")),
    }
}

pub async fn call_action(
    db_pool: sqlx::PgPool,
    action: String,
    body_params: HashMap<String, String>,
) -> Result<HttpResponseData, InternalAppError> {
    match action.as_str() {
        "payment_success" => {
            tracing::info!("Entering payment_success webhook");
            let Some(payment_id) = body_params.get("payment_id") else {
                return Err(InternalAppError::new(format!(
                    "Did not get payment_id, {body_params:#?}"
                )))?;
            };
            let payment_id = PaymentId(payment_id.to_string());
            invoice::success(db_pool.clone(), payment_id)
                .await
                .map_err(InternalAppError::new)?;
            println!("payment_success webhook success");
            let resp = ok(
                "Payment successful\n".to_string().into_bytes(),
                http_stuff::http_response::ContentType::TextPlain,
                Some(Charset::Utf8),
            );
            Ok(resp)
        }
        "set_website_fund_xmr" => {
            let amount = body_params
                .get("amount")
                .map(|s| s.as_str())
                .unwrap_or("0")
                .to_string();

            let amount = amount.parse::<u64>().map_err(InternalAppError::new)?;
            let amount = XMRAtomicUnit::from_atomic(amount);
            webhooks::set_website_fund_xmr(db_pool.clone(), amount)
                .await
                .map_err(InternalAppError::new)?;
            println!("website_fund_xmr webhook success");
            let resp = ok(
                "Set success\n".to_string().into_bytes(),
                http_stuff::http_response::ContentType::TextPlain,
                Some(Charset::Utf8),
            );
            Ok(resp)
        }
        "add_admin_permissions_to_user" => {
            let user_id = body_params
                .get("user_id")
                .map(|s| s.as_str())
                .unwrap_or("0")
                .to_string();
            let user = user::get_user_by_user_id(db_pool.clone(), user_id.clone()).await;
            if user.is_err() {
                let mut resp = ok(
                    "User not found\n".to_string().into_bytes(),
                    http_stuff::http_response::ContentType::TextPlain,
                    Some(Charset::Utf8),
                );
                resp.status_code = http_stuff::http_response::StatusCode::Code404NotFound;
                return Ok(resp);
            }
            abac::add_admin_permissions_to_user(db_pool.clone(), user_id.clone()).await;

            let content = format!("Permissions added for user_id \"{user_id}\"\n");
            let resp = ok(
                content.into_bytes(),
                http_stuff::http_response::ContentType::TextPlain,
                Some(Charset::Utf8),
            );
            Ok(resp)
        }
        _ => {
            println!("Invalid action");
            let mut resp = ok(
                "Invalid action\n".to_string().into_bytes(),
                http_stuff::http_response::ContentType::TextPlain,
                Some(Charset::Utf8),
            );
            resp.status_code = http_stuff::http_response::StatusCode::Code404NotFound;
            Ok(resp)
        }
    }
}
