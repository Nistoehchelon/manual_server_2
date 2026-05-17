use std::{collections::HashMap, net::IpAddr};

use error_structs::SomeSortOfError;

pub mod create_forum;
pub mod create_thread;
pub mod dashboard;
pub mod dump_request;
pub mod dynamic_stylesheet;
pub mod edit_forum_thread_message;
pub mod email_otp;
pub mod forum;
pub mod forum_thread;
pub mod forum_thread_message_versions;
pub mod index;
pub mod invoice;
pub mod login;
pub mod logout;
pub mod manage_custom_site_pages;
pub mod not_found;
pub mod signup;
pub mod subscription;
pub mod view_custom_site_page;
pub mod webhook;

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
