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
    let action = body_params
        .get("action")
        .map(|s| s.as_str())
        .unwrap_or("")
        .to_string();

    match action.as_str() {
        "website_fund_xmr_add" => {

            let amount = body_params
                .get("amount")
                .map(|s| s.as_str())
                .unwrap_or("")
                .to_string();
            let amount = amount.parse::<u64>().map_err(InternalAppError::new)?;
            let amount = webhooks::XMRAmount::from_atomic(amount);
            webhooks::add_website_fund_xmr(rt.db_pool.clone(), amount)
                .await
                .map_err(InternalAppError::new)?;
            println!("website_fund_xmr webhook success");
            let resp = ok(
                "Success".to_string().into_bytes(),
                http_stuff::http_response::ContentType::TextPlain,
                Some(Charset::Utf8),
            );
            Ok(resp)
        }
        "website_fund_xmr_subtract" => {

            let amount = body_params
                .get("amount")
                .map(|s| s.as_str())
                .unwrap_or("")
                .to_string();
            let amount = amount.parse::<u64>().map_err(InternalAppError::new)?;
            let amount = webhooks::XMRAmount::from_atomic(amount);
            webhooks::subtract_website_fund_xmr(rt.db_pool.clone(), amount)
                .await
                .map_err(InternalAppError::new)?;
            println!("website_fund_xmr webhook success");
            let resp = ok(
                "Success".to_string().into_bytes(),
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
