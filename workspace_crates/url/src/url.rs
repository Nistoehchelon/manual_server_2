use encoding::encode_request_string;
use indexmap::IndexMap;
use std::fmt;

pub type Query = IndexMap<String, String>;

#[derive(Clone, Debug)]
pub struct Url {
    pub path: String,
    pub query: Query,
}

impl Url {
    pub fn new(path: String, query: Query) -> Self {
        Self { path, query }
    }
}


impl fmt::Display for Url {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path)?;

        if self.query.is_empty() {
            return Ok(());
        }
        write!(f, "?")?;

        let mut first = true;
        for (k, v) in &self.query {
            if !first {
                write!(f, "&")?;
            }
            first = false;

            let encoded_key = encode_request_string(k.to_string());
            let encoded_value = encode_request_string(v.to_string());

            write!(f, "{}={}", encoded_key, encoded_value)?;
        }

        Ok(())
    }
}
