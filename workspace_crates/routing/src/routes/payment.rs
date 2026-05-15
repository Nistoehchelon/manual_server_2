use error_structs::InternalAppError;
use http_stuff::http_response::{Charset, HttpResponseData};

use crate::RouterData;
use serde::Serialize;


#[derive(Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum PaymentStatusResponse {
    _Pending,
    Confirmations { current: u8, required: u8 },
    _Complete,
}

pub async fn get_status(_router_data: RouterData) -> Result<HttpResponseData, InternalAppError> {
    let content = serde_json::to_string(&PaymentStatusResponse::Confirmations { current: 5, required: 10 }).map_err(InternalAppError::new)?; // test random paymentstatusresponse, let's say confirmations current 5 out of 10
    let resp = http_stuff::http_response::ok(
        content.into(),
        http_stuff::http_response::ContentType::ApplicationJson,
        Some(Charset::Utf8),
    );
    Ok(resp)
} 

