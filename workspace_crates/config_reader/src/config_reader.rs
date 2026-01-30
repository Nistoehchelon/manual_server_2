use std::{fs::read_to_string, path::PathBuf, str::FromStr};
pub struct Config {
    pub ip_and_port: String,
    pub database_str: String,
    pub domain_name: String,
    pub server_global_requests_token_bucket_capacity: f64,
    pub server_global_requests_token_bucket_refill_rate_per_millis: f64,
    pub per_ip_token_bucket_capacity: f64,
    pub per_ip_token_bucket_refill_rate_per_millis: f64,

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
}
#[derive(Debug, PartialEq)]
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
    let mut port: Option<String> = None;
    let mut db_url: Option<String> = None;

    let mut domain_name: Option<String> = None;
    let mut environment: Option<Environment> = None;
    let mut lets_encrypt_production: Option<bool> = None;

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
            "IP" => ip = Some(value),
            "PORT" => port = Some(value),
            "DATABASE_URL" => db_url = Some(value),
            "DOMAIN_NAME" => domain_name = Some(value),
            "ENVIRONMENT" => environment = Some(value.parse().unwrap()),
            "LETS_ENCRYPT_PRODUCTION" => lets_encrypt_production = Some(value.parse().unwrap()),
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
            unk_key => panic!("Unknown key {unk_key}"),
        }
    }

    let ip = ip.expect("Missing IP in config file");
    let port = port.expect("Missing PORT in config file");
    let db_url = db_url.expect("Missing DATABASE_URL in config file");
    let domain_name = domain_name.expect("Missing DOMAIN_NAME in config file");
    let server_token_bucket_capacity =
        server_token_bucket_capacity.expect("Missing SERVER_TOKEN_BUCKET_CAPACITY in config");
    let server_token_bucket_refill_rate = server_token_bucket_refill_rate
        .expect("Missing SERVER_TOKEN_BUCKET_REFILL_RATE_PER_MILLIS in config file");
    let per_ip_token_bucket_capacity =
        per_ip_token_bucket_capacity.expect("Missing PER_IP_TOKEN_BUCKET_CAPACITY in config");
    let per_ip_token_bucket_refill_rate = per_ip_token_bucket_refill_rate
        .expect("Missing PER_IP_TOKEN_BUCKET_REFILL_RATE_PER_MILLIS in config file");

    let common_stream_timeout_millis =
        common_stream_timeout_millis.expect("Missing COMMON_STREAM_TIMEOUT_MILLIS in config file");

    let max_total_headers_size_bytes =
        max_total_headers_size_bytes.expect("Missing MAX_TOTAL_HEADERS_SIZE_BYTES in config file");
    let max_headers_count = max_headers_count.expect("Missing MAX_HEADERS_COUNT in config file");
    let max_header_name_length =
        max_header_name_length.expect("Missing MAX_HEADER_NAME_LENGTH in config file");
    let max_header_value_length =
        max_header_value_length.expect("Missing MAX_HEADER_VALUE_LENGTH in config file");
    let max_total_body_size_bytes =
        max_total_body_size_bytes.expect("Missing MAX_TOTAL_BODY_SIZE_BYTES in config file");
    let max_request_path_length =
        max_request_path_length.expect("Missing MAX_REQUEST_PATH_LENGTH in config file");
    let max_cookies_count = max_cookies_count.expect("Missing MAX_COOKIES_COUNT in config file");
    let use_local_ssl = environment.expect("Missing ENVIRONMENT in config file");
    let lets_encrypt_production = lets_encrypt_production.expect("Missing LETS_ENCRYPT_PRODUCTION in config file");

    Config {
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
        environment: use_local_ssl,
        lets_encrypt_production,

    }
}
