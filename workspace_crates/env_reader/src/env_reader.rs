use std::fs::read_to_string;
pub struct Env {
    pub ip_and_port: String,
    pub database_str: String,
}

pub fn get_env() -> Env {
    let contents = read_to_string("env").expect("Could't file or read env file");

    let mut ip: Option<&str> = None;
    let mut port: Option<&str> = None;
    let mut db_url: Option<&str> = None;

    for line in contents
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .filter(|l| !l.starts_with('#'))
    {
        let (key, value) = line
            .split_once('=')
            .expect(format!("Invalid env line {line} expected KEY=VALUE").as_str());

        let value = value.trim();

        match key {
            "IP" => ip = Some(value),
            "PORT" => port = Some(value),
            "DATABASE_URL" => db_url = Some(value),
            unk_key => println!("Unknown key {unk_key}"),
        }
    }

    let ip = ip.expect("Missing IP in env file");
    let port = port.expect("Missing PORT in env file");
    let db_url = db_url.expect("Missing DATABASE_URL in env file");

    Env {
        ip_and_port: format!("{ip}:{port}"),
        database_str: db_url.to_string(),
    }
}
