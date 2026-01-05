use std::fs::read_to_string;
pub struct Env {
    pub ip_and_port: String,
    pub database_str: String,
    pub server_global_requests_token_bucket_capacity: f64,
    pub server_global_requests_token_bucket_refill_rate_per_millis: f64,
    pub per_ip_token_bucket_capacity: f64,
    pub per_ip_token_bucket_refill_rate_per_millis: f64,
}

pub fn get_env() -> Env {
    let contents = read_to_string("env").expect("Could't file or read env file");

    let mut ip: Option<String> = None;
    let mut port: Option<String> = None;
    let mut db_url: Option<String> = None;

    let mut server_token_bucket_capacity: Option<f64> = None;
    let mut server_token_bucket_refill_rate: Option<f64> = None;
    let mut per_ip_token_bucket_capacity: Option<f64> = None;
    let mut per_ip_token_bucket_refill_rate: Option<f64> = None;

    for line in contents
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .filter(|l| !l.starts_with('#'))
    {
        let (key, value) = line
            .split_once('=')
            .expect(format!("Invalid env line {line} expected KEY=VALUE").as_str());

        let value = value.trim().to_string();

        match key {
            "IP" => ip = Some(value),
            "PORT" => port = Some(value),
            "DATABASE_URL" => db_url = Some(value),
            "SERVER_TOKEN_BUCKET_CAPACITY" =>  {
            let value = value.parse::<f64>().expect(&format!("Expected f64 parsing of {value}  to work out"));
             server_token_bucket_capacity = Some(value)
            },
            "SERVER_TOKEN_BUCKET_REFILL_RATE_PER_MILLIS" =>  {
            let value = value.parse::<f64>().expect(&format!("Expected f64 parsing of {value}  to work out"));
             server_token_bucket_refill_rate = Some(value)
            },
            "PER_IP_TOKEN_BUCKET_CAPACITY" =>  {
            let value = value.parse::<f64>().expect(&format!("Expected f64 parsing of {value}  to work out"));
             per_ip_token_bucket_capacity = Some(value)
            },
            "PER_IP_TOKEN_BUCKET_REFILL_RATE_PER_MILLIS" =>  {
            let value = value.parse::<f64>().expect(&format!("Expected f64 parsing of {value}  to work out"));
             per_ip_token_bucket_refill_rate = Some(value)
            },
            unk_key => panic!("Unknown key {unk_key}"),
        }
    }

    let ip = ip.expect("Missing IP in env file");
    let port = port.expect("Missing PORT in env file");
    let db_url = db_url.expect("Missing DATABASE_URL in env file");
    let server_token_bucket_capacity = server_token_bucket_capacity.expect("Missing SERVER_TOKEN_BUCKET_CAPACITY in env");
    let server_token_bucket_refill_rate = server_token_bucket_refill_rate.expect("Missing SERVER_TOKEN_BUCKET_REFILL_RATE_PER_MILLIS in env file");
    let per_ip_token_bucket_capacity = per_ip_token_bucket_capacity.expect("Missing PER_IP_TOKEN_BUCKET_CAPACITY in env");
    let per_ip_token_bucket_refill_rate = per_ip_token_bucket_refill_rate.expect("Missing PER_IP_TOKEN_BUCKET_REFILL_RATE_PER_MILLIS in env file");

    Env {
        ip_and_port: format!("{ip}:{port}"),
        database_str: db_url.to_string(),
        server_global_requests_token_bucket_capacity: server_token_bucket_capacity,
        server_global_requests_token_bucket_refill_rate_per_millis: server_token_bucket_refill_rate,
        per_ip_token_bucket_capacity: per_ip_token_bucket_capacity,
        per_ip_token_bucket_refill_rate_per_millis: per_ip_token_bucket_refill_rate,

    }
}
