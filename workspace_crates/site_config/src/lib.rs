#[derive(Debug, Clone, serde::Deserialize)]
pub struct SiteConfig {
    // app specific
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

    pub webhook_and_api_secret_path_prefix: String,
    pub payment_server_url: String,
}
