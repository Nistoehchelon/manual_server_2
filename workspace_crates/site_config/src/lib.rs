#[derive(Debug, Clone, serde::Deserialize)]
pub struct SiteConfig {
    pub ip: String,
    pub port: u64,
    pub rust_log_directory_path: String,
    pub database_url: String,
    pub password_hasher_config: password_hashing::PasswordHasherConfig,
    pub using_https: bool,
    pub min_length_user_password: usize,
    pub max_length_user_password: usize,

    pub min_length_forum_thread_name: usize,
    pub max_length_forum_thread_name: usize,

    pub min_length_forum_message: usize,
    pub max_length_forum_message: usize,

    pub cloudflare_turnstile_enabled: bool,
    pub cloudflare_turnstile_site_key: String,
    pub cloudflare_turnstile_secret_key: String,
    pub cloudflare_turnstile_post_parameter_name: String,
    pub payment_server_cloudflare_bypass_under_attack_check_header_name: String,
    pub payment_server_cloudflare_bypass_under_attack_check_header_token_value: String,

    pub webhook_and_api_secret_path_prefix: String,
}
