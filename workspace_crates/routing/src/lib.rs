pub mod router;
pub mod routes;
use std::collections::HashMap;
use std::net::IpAddr;

use error_structs::SomeSortOfError;
pub use router::RouterData;
pub use router::generate_response_data;
pub use router::route;

pub async fn validate_turnstile(
    body_params: HashMap<String, String>,
    cloudflare_turnstile_post_parameter_name: String,
    client_ip: IpAddr,
    cloudflare_turnstile_secret_key: String,
) -> Result<bool, SomeSortOfError> {
    let cf_turnstile_token = match body_params
        .get(&cloudflare_turnstile_post_parameter_name)
        .cloned()
    {
        Some(t) => t,
        None => {
            return Err(SomeSortOfError::new(
                "Could not retrieve cf_turnstile_token from request body",
            ));
        }
    };
    Ok(cloudflare_turnstile::validate_turnstile(
        cloudflare_turnstile_secret_key,
        cf_turnstile_token,
        client_ip,
    )
    .await)
}
