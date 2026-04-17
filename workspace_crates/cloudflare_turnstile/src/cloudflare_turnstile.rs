use std::net::IpAddr;
pub async fn validate_turnstile(cf_secret_key: String, cf_token: String, client_ip: IpAddr) -> bool {
    use reqwest::Client;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Debug)]
    struct TurnstileRequest<'a> {
        secret: &'a str,
        response: &'a str,
        remoteip: &'a str,
    }

    #[derive(Deserialize, Debug)]
    pub struct TurnstileResponse {
        pub success: bool,
        #[serde(rename = "error-codes")]
        pub error_codes: Option<Vec<String>>,
    }

    let cf_url = String::from("https://challenges.cloudflare.com/turnstile/v0/siteverify");

    let body = TurnstileRequest {
        secret: &cf_secret_key,
        response: &cf_token,
        remoteip: &client_ip.to_string(),
    };
    let client = Client::new();
    let response = match client.post(cf_url).json(&body).send().await {
        Ok(response) => match response.json::<TurnstileResponse>().await {
            Ok(r) => r,
            Err(err) => {
                eprintln!("Turnstile JSON parse error: {:?}", err);
                TurnstileResponse {
                    success: false,
                    error_codes: Some(vec!["myapperror parse".to_string()]),
                }
            }
        },
        Err(err) => {
             eprintln!("Turnstile validation error: {:?}", err);
            TurnstileResponse {
                success: false,
                error_codes: Some(vec!["internal-error".to_string()]),
            }
        }
    };

    
    println!("Turnstile validation result: {} {:#?}",response.success, response.error_codes, );
    response.success
}
