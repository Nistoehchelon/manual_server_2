use std::fs::read_to_string;
use std::io::ErrorKind;
pub struct Env {
    pub ip_and_port: String,
    pub database_str: String,
}

pub fn get_env() -> Env {
    let env_file_contents = match read_to_string("env") {
        Ok(contents) => contents,
        Err(e) => match e.kind() {
            ErrorKind::NotFound => panic!(
                "Couldn't find the env file. \n Help: Create file called `env` in the same path you are calling the executable from, with example contents: IP=127.0.0.1>>>PORT=8080 (Note: >>> is just a delimeter used like a newline"
            ),
            _ => panic!("Error reading file: {}", e),
        },
    };

    let delimeter = ">>>";
    let env_parts: Vec<&str> = env_file_contents.split(delimeter).collect();
    assert!(
        env_parts.len() == 3,
        "Expected the env file to have 3 entries, got {}",
        env_parts.len()
    );
    let mut found_ip = false;
    let mut found_port = false;
    let mut found_db_url = false;
    let mut ip = "";
    let mut port = "";
    let mut db_url = "";
    for part in env_parts {
        let split: Vec<&str> = part.split("=").collect();
        assert!(
            split.len() == 2,
            "Invalid env file format. Expected key=value (split by = length 2) but got different length"
        );
        let key = split[0];
        let value = split[1];
        if key == "IP" {
            found_ip = true;
            ip = value.trim();
        }
        if key == "PORT" {
            found_port = true;
            port = value.trim();
        }
        if key == "DATABASE_URL" {
            found_db_url = true;
            db_url = value.trim();
        }
    }
    assert!(
        found_ip && found_port && found_db_url,
        "env file must specify port and ip and db url. Example: IP=127.0.0.1>>>PORT=8080>>>DATABASE_URL=postgres://user123:password123@localhost:5432/database123"
    );
    Env {
        ip_and_port: format!("{ip}:{port}"),
        database_str: format!("{db_url}"),
    }
}
