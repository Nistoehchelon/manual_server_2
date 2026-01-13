use http_stuff::http_request_parsing::{Query, encode_string};

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

impl ToString for Url {
    fn to_string(&self) -> String {
        let mut result = self.path.clone();

        if !self.query.is_empty() {
            result.push('?');

            // iterate and append key=value pairs efficiently
            let mut first = true;
            for (k, v) in &self.query {
                if !first {
                    result.push('&');
                }
                first = false;

                let encoded_key = encode_string(k.to_string());
                let encoded_value = encode_string(v.to_string());

                result.push_str(&encoded_key);
                result.push('=');
                result.push_str(&encoded_value);
            }
        }

        result
    }
}
