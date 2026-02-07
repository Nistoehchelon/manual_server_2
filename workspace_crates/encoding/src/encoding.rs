use urlencoding;

pub fn decode_request_string(s: String) -> Result<String, String> {
    let s = s.replace("+", " ");
    let s = match urlencoding::decode(&s) {
        Err(e) => {
            return Err(format!("Could not decode string into string, {e:#?}"));
        }
        Ok(s) => s,
    };
    let s = s.to_string();
    Ok(s)
}

pub fn encode_request_string(s: String) -> String {
    let s = s.replace(" ", "+");
    let s = urlencoding::encode(&s);
    s.to_string()
}
