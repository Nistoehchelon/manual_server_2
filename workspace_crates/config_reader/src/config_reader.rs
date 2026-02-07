use std::{fs::read_to_string, path::PathBuf, str::FromStr, time::Duration};

#[derive(Debug, Clone)]
pub struct Config {
    pub ip_and_port: String,
    pub port: u64,
    pub ip: String,
    pub database_str: String,
    pub domain_name: String,
    pub server_global_requests_token_bucket_capacity: f64,
    pub server_global_requests_token_bucket_refill_rate_per_millis: f64,
    pub per_ip_token_bucket_capacity: f64,
    pub per_ip_token_bucket_refill_rate_per_millis: f64,
    pub max_requests_per_keep_alive_connection: u64,
    pub max_total_time_allowed_for_all_requests_on_keep_alive_connection_millis: u64,

    pub common_stream_timeout_millis: u64,

    pub max_total_headers_size_bytes: usize,
    pub max_headers_count: usize,
    pub max_header_name_len: usize,
    pub max_header_value_len: usize,
    pub max_total_body_size_bytes: usize,
    pub max_request_path_length: usize,
    pub max_cookies_count: usize,
    pub environment: Environment,
    pub lets_encrypt_production: bool,
    pub lets_encrypt_contact_email: String,
    pub server_time_credits_refill_rate_per_millis: Duration,
    pub server_time_credits_capacity: Duration,
    pub per_ip_time_credits_refill_rate_per_millis: Duration,
    pub per_ip_time_credits_capacity: Duration,
}
#[derive(Debug, PartialEq, Clone)]
pub enum Environment {
    Local,
    Production,
}
impl FromStr for Environment {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "local" => Ok(Environment::Local),
            "production" => Ok(Environment::Production),
            any_other_input => Err(format!("Invalid environment {any_other_input}")),
        }
    }
}
pub fn read_config(config_file_path: PathBuf) -> Config {
    let contents = read_to_string(&config_file_path)
        .expect(format!("Could not find path config {config_file_path:#?}").as_str());

    let mut ip: Option<String> = None;
    let mut port: Option<u64> = None;
    let mut db_url: Option<String> = None;

    let mut domain_name: Option<String> = None;
    let mut environment: Option<Environment> = None;
    let mut lets_encrypt_production: Option<bool> = None;
    let mut lets_encrypt_contact_email: Option<String> = None;

    let mut server_token_bucket_capacity: Option<f64> = None;
    let mut server_token_bucket_refill_rate: Option<f64> = None;
    let mut per_ip_token_bucket_capacity: Option<f64> = None;
    let mut per_ip_token_bucket_refill_rate: Option<f64> = None;

    let mut common_stream_timeout_millis: Option<u64> = None;

    let mut max_total_headers_size_bytes: Option<usize> = None;
    let mut max_headers_count: Option<usize> = None;
    let mut max_header_name_length: Option<usize> = None;
    let mut max_header_value_length: Option<usize> = None;
    let mut max_total_body_size_bytes: Option<usize> = None;
    let mut max_request_path_length: Option<usize> = None;
    let mut max_cookies_count: Option<usize> = None;
    let mut max_requests_per_keep_alive_connection: Option<u64> = None;
    let mut max_total_time_allowed_for_all_requests_on_keep_alive_connection_millis: Option<u64> =
        None;

    let mut server_time_credits_capacity_millis: Option<u64> = None;
    let mut server_time_credits_refill_rate_millis_per_millis: Option<u64> = None;

    let mut per_ip_time_credits_capacity_millis: Option<u64> = None;
    let mut per_ip_time_credits_refill_rate_millis: Option<u64> = None;

    let mut unk_keys_count = 0;

    for line in contents
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .filter(|l| !l.starts_with('#'))
    {
        let (key, value) = line
            .split_once('=')
            .expect(format!("Invalid config line {line} expected KEY=VALUE").as_str());

        let key = key.trim();
        let value = value.trim().to_string();

        match key {
            "PER_IP_TIME_CREDITS_CAPACITY_MILLIS" => {
                per_ip_time_credits_capacity_millis = Some(value.parse().unwrap())
            }
            "PER_IP_TIME_CREDITS_REFILL_RATE_MILLIS_PER_MILLIS" => {
                per_ip_time_credits_refill_rate_millis = Some(value.parse().unwrap())
            }
            "SERVER_TIME_CREDITS_CAPACITY_MILLIS" => {
                server_time_credits_capacity_millis = Some(value.parse().unwrap())
            }
            "SERVER_TIME_CREDITS_REFILL_RATE_MILLIS_PER_MILLIS" => {
                server_time_credits_refill_rate_millis_per_millis = Some(value.parse().unwrap())
            }
            "MAX_TOTAL_TIME_ALLOWED_FOR_ALL_REQUESTS_ON_KEEP_ALIVE_CONNECTION_MILLIS" => {
                max_total_time_allowed_for_all_requests_on_keep_alive_connection_millis =
                    Some(value.parse().unwrap())
            }
            "MAX_REQUESTS_PER_KEEP_ALIVE_CONNECTION" => {
                max_requests_per_keep_alive_connection = Some(value.parse().unwrap())
            }
            "IP" => ip = Some(value),
            "PORT" => port = Some(value.parse().unwrap()),
            "DATABASE_URL" => db_url = Some(value),
            "DOMAIN_NAME" => domain_name = Some(value),
            "ENVIRONMENT" => environment = Some(value.parse().unwrap()),
            "LETS_ENCRYPT_PRODUCTION" => lets_encrypt_production = Some(value.parse().unwrap()),
            "LETS_ENCRYPT_CONTACT_EMAIL" => {
                lets_encrypt_contact_email = Some(value.parse().unwrap())
            }
            "SERVER_TOKEN_BUCKET_CAPACITY" => {
                let value = value
                    .parse::<f64>()
                    .expect(&format!("Expected f64 parsing of {value}  to work out"));
                server_token_bucket_capacity = Some(value)
            }
            "SERVER_TOKEN_BUCKET_REFILL_RATE_PER_MILLIS" => {
                let value = value
                    .parse::<f64>()
                    .expect(&format!("Expected f64 parsing of {value}  to work out"));
                server_token_bucket_refill_rate = Some(value)
            }
            "PER_IP_TOKEN_BUCKET_CAPACITY" => {
                let value = value
                    .parse::<f64>()
                    .expect(&format!("Expected f64 parsing of {value}  to work out"));
                per_ip_token_bucket_capacity = Some(value)
            }
            "PER_IP_TOKEN_BUCKET_REFILL_RATE_PER_MILLIS" => {
                let value = value
                    .parse::<f64>()
                    .expect(&format!("Expected f64 parsing of {value}  to work out"));
                per_ip_token_bucket_refill_rate = Some(value)
            }
            "COMMON_STREAM_TIMEOUT_MILLIS" => {
                let value = value
                    .parse::<u64>()
                    .expect(&format!("Expected u64 parsing of {value}  to work out"));
                common_stream_timeout_millis = Some(value)
            }
            // ...
            "MAX_TOTAL_HEADERS_SIZE_BYTES" => {
                let value = value
                    .parse::<usize>()
                    .expect(&format!("Expected usize parsing of {value}  to work out"));
                max_total_headers_size_bytes = Some(value)
            }
            "MAX_HEADERS_COUNT" => {
                let value = value
                    .parse::<usize>()
                    .expect(&format!("Expected usize parsing of {value}  to work out"));
                max_headers_count = Some(value)
            }

            "MAX_HEADER_NAME_LENGTH" => {
                let value = value
                    .parse::<usize>()
                    .expect(&format!("Expected usize parsing of {value}  to work out"));
                max_header_name_length = Some(value)
            }
            "MAX_HEADER_VALUE_LENGTH" => {
                let value = value
                    .parse::<usize>()
                    .expect(&format!("Expected usize parsing of {value}  to work out"));
                max_header_value_length = Some(value)
            }

            "MAX_TOTAL_BODY_SIZE_BYTES" => {
                let value = value
                    .parse::<usize>()
                    .expect(&format!("Expected usize parsing of {value}  to work out"));
                max_total_body_size_bytes = Some(value)
            }

            "MAX_REQUEST_PATH_LENGTH" => {
                let value = value
                    .parse::<usize>()
                    .expect(&format!("Expected usize parsing of {value}  to work out"));
                max_request_path_length = Some(value)
            }

            "MAX_COOKIES_COUNT" => {
                let value = value
                    .parse::<usize>()
                    .expect(&format!("Expected usize parsing of {value}  to work out"));
                max_cookies_count = Some(value)
            }

            // ...
            unk_key => {
                unk_keys_count += 1;
                println!("Unknown key {unk_key}")
            }
        }
    }

    if unk_keys_count > 0 {
        panic!("Panicking due to {unk_keys_count} unknown keys.");
    }

    let server_time_credits_capacity = Duration::from_millis(
        server_time_credits_capacity_millis
            .expect("Missing SERVER_TIME_CREDITS_CAPACITY_MILLIS in env file"),
    );
    let server_time_credits_refill_rate_per_millis = Duration::from_millis(
        server_time_credits_refill_rate_millis_per_millis
            .expect("Missing SERVER_TIME_CREDITS_REFILL_RATE_MILLIS_PER_MILLIS in env file"),
    );

    let per_ip_time_credits_capacity = Duration::from_millis(
        per_ip_time_credits_capacity_millis
            .expect("Missing PER_IP_TIME_CREDITS_CAPACITY_MILLIS in env file"),
    );
    let per_ip_time_credits_refill_rate_per_millis = Duration::from_millis(
        per_ip_time_credits_refill_rate_millis
            .expect("Missing PER_IP_TIME_CREDITS_REFILL_RATE_MILLIS_PER_MILLIS in env file"),
    );

    let max_requests_per_keep_alive_connection = max_requests_per_keep_alive_connection
        .expect("Missing MAX_REQUESTS_PER_KEEP_ALIVE_CONNECTION in file");
    let max_total_time_allowed_for_all_requests_on_keep_alive_connection_millis = max_total_time_allowed_for_all_requests_on_keep_alive_connection_millis.expect("Missing MAX_TOTAL_TIME_ALLOWED_FOR_ALL_REQUESTS_ON_KEEP_ALIVE_CONNECTION_MILLIS in file");
    let ip = ip.expect("Missing IP in env file");
    let port = port.expect("Missing PORT in env file");
    let db_url = db_url.expect("Missing DATABASE_URL in env file");
    let domain_name = domain_name.expect("Missing DOMAIN_NAME in env file");
    let server_token_bucket_capacity =
        server_token_bucket_capacity.expect("Missing SERVER_TOKEN_BUCKET_CAPACITY in config");
    let server_token_bucket_refill_rate = server_token_bucket_refill_rate
        .expect("Missing SERVER_TOKEN_BUCKET_REFILL_RATE_PER_MILLIS in env file");
    let per_ip_token_bucket_capacity =
        per_ip_token_bucket_capacity.expect("Missing PER_IP_TOKEN_BUCKET_CAPACITY in config");
    let per_ip_token_bucket_refill_rate = per_ip_token_bucket_refill_rate
        .expect("Missing PER_IP_TOKEN_BUCKET_REFILL_RATE_PER_MILLIS in env file");

    let common_stream_timeout_millis =
        common_stream_timeout_millis.expect("Missing COMMON_STREAM_TIMEOUT_MILLIS in env file");

    let max_total_headers_size_bytes =
        max_total_headers_size_bytes.expect("Missing MAX_TOTAL_HEADERS_SIZE_BYTES in env file");
    let max_headers_count = max_headers_count.expect("Missing MAX_HEADERS_COUNT in env file");
    let max_header_name_length =
        max_header_name_length.expect("Missing MAX_HEADER_NAME_LENGTH in env file");
    let max_header_value_length =
        max_header_value_length.expect("Missing MAX_HEADER_VALUE_LENGTH in env file");
    let max_total_body_size_bytes =
        max_total_body_size_bytes.expect("Missing MAX_TOTAL_BODY_SIZE_BYTES in env file");
    let max_request_path_length =
        max_request_path_length.expect("Missing MAX_REQUEST_PATH_LENGTH in env file");
    let max_cookies_count = max_cookies_count.expect("Missing MAX_COOKIES_COUNT in env file");

    let environment = environment.expect("Missing ENVIRONMENT in env file");

    let lets_encrypt_production =
        lets_encrypt_production.expect("Missing LETS_ENCRYPT_PRODUCTION in env file");

    let lets_encrypt_contact_email =
        lets_encrypt_contact_email.expect("Missing LETS_ENCRYPT_CONTACT_EMAIL in env file");

    Config {
        ip: ip.clone(),
        port,
        ip_and_port: format!("{ip}:{port}"),
        database_str: db_url.to_string(),
        domain_name,
        server_global_requests_token_bucket_capacity: server_token_bucket_capacity,
        server_global_requests_token_bucket_refill_rate_per_millis: server_token_bucket_refill_rate,
        per_ip_token_bucket_capacity: per_ip_token_bucket_capacity,
        per_ip_token_bucket_refill_rate_per_millis: per_ip_token_bucket_refill_rate,
        common_stream_timeout_millis: common_stream_timeout_millis,
        max_cookies_count,
        max_header_name_len: max_header_name_length,
        max_total_body_size_bytes,
        max_header_value_len: max_header_value_length,
        max_headers_count,
        max_request_path_length,
        max_total_headers_size_bytes,
        environment,
        lets_encrypt_production,
        lets_encrypt_contact_email,
        max_requests_per_keep_alive_connection,
        max_total_time_allowed_for_all_requests_on_keep_alive_connection_millis,
        server_time_credits_capacity,
        server_time_credits_refill_rate_per_millis,
        per_ip_time_credits_capacity,
        per_ip_time_credits_refill_rate_per_millis,
    }
}
