#[derive(Clone)]
pub struct RouterData {
    pub server_config: config_reader::ServerConfig,
    pub site_config: site_config::SiteConfig,
    pub db_pool: sqlx::PgPool,
    pub session: session::Session,
    pub http_request_data: http_stuff::http_request_parsing::HttpRequestData,
    pub base_template_data: html_templating::html_template_structs::BaseTemplateData,
    pub session_notifications: Vec<session::SessionNotification>,
}
