use std::collections::HashMap;

use error_structs::InternalAppError;
use http_stuff::{
    http_request_parsing::parse_body_into_string_form_input_params,
    http_response::{Charset, HttpResponseData, ok},
};

use crate::RouterData;

pub async fn post(rt: RouterData) -> Result<HttpResponseData, InternalAppError> {
    let body_params =
        parse_body_into_string_form_input_params(rt.http_request_data.body_bytes.clone())
            .map_err(InternalAppError::new)?;
    let token = body_params
        .get("token")
        .map(|s| s.as_str())
        .unwrap_or("")
        .to_string();
    let validity_of_the_token = webhooks::validate_token(rt.db_pool.clone(), token.clone())
        .await
        .map_err(InternalAppError::new)?;
    if !validity_of_the_token {
        println!("Invalid token provided.");
         let mut resp = ok(
                "Invalid token provided".to_string().into_bytes(),
                http_stuff::http_response::ContentType::TextPlain,
                Some(Charset::Utf8),
            );
            resp.status_code = http_stuff::http_response::StatusCode::Code404NotFound;
            return Ok(resp);
    }
  call_webhook(rt.db_pool, body_params).await
}

pub async fn call_webhook(db_pool: sqlx::PgPool, body_params: HashMap<String, String>) -> Result<HttpResponseData, InternalAppError> {
      
    let action = body_params
        .get("action")
        .map(|s| s.as_str())
        .unwrap_or("")
        .to_string();

    match action.as_str() {
        "set_website_fund_xmr" => {
            let amount = body_params
                .get("amount")
                .map(|s| s.as_str())
                .unwrap_or("0")
                .to_string();
            let amount = amount.parse::<u64>().map_err(InternalAppError::new)?;
            let amount = webhooks::XMRAtomicUnit::from_atomic(amount);
            webhooks::set_website_fund_xmr(db_pool.clone(), amount)
                .await
                .map_err(InternalAppError::new)?;
            println!("website_fund_xmr webhook success");
            let resp = ok(
                "Add success".to_string().into_bytes(),
                http_stuff::http_response::ContentType::TextPlain,
                Some(Charset::Utf8),
            );
            Ok(resp)
        }
        _ => {
            println!("Invalid action");
            let mut resp = ok(
                "Invalid action".to_string().into_bytes(),
                http_stuff::http_response::ContentType::TextPlain,
                Some(Charset::Utf8),
            );
            resp.status_code = http_stuff::http_response::StatusCode::Code404NotFound;
            Ok(resp)
        }
    }
}