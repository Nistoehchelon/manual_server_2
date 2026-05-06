use count_string_length::count_string_length::{count_string_length, valid_string_length};
use error_structs::PublicError;

#[derive(Clone, Debug, Default)]
pub struct User {
    pub primary_key: i64,
    pub id: String,
    pub username: String,
    pub password_hash: String,
}

pub const MIN_LENGTH_USERNAME: usize = 1;
pub const MAX_LENGTH_USERNAME: usize = 50;
#[derive(Clone, Debug, PartialEq, Eq)]

pub struct ValidUsername {
    inner: String,
}
impl ValidUsername {
    pub fn parse(s: String) -> Result<Self, PublicError> {
        let s = s.trim().to_string();
        let len = count_string_length(s.clone());
        if !valid_string_length(s.clone(), MIN_LENGTH_USERNAME, MAX_LENGTH_USERNAME) {
            return Err(PublicError::new(format!(
                "Username length must be between {} to {} characters but {} given.",
                MIN_LENGTH_USERNAME, MAX_LENGTH_USERNAME, len
            )));
        }
        if s.chars().any(|c| c.is_control() || c == '\n') {
            return Err(PublicError::new(
                "The username must not contain newlines or control characters.",
            ));
        }



        Ok(Self { inner: s })
    }
}
impl std::fmt::Display for ValidUsername {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}


#[derive(Clone, Debug, PartialEq, Eq)]

pub struct ValidPassword {
    inner: String,
}
impl ValidPassword {
    pub fn parse(s: String, min_length_user_password: usize, max_length_user_password: usize) -> Result<Self, PublicError> {
        let len = count_string_length(s.clone());
        if !valid_string_length(s.clone(), min_length_user_password, max_length_user_password) {
            return Err(PublicError::new(format!(
                "Password must be {} to {} characters but {} given.",
                min_length_user_password, max_length_user_password, len
            )));
        }
        Ok(Self { inner: s })
    }
}
impl std::fmt::Display for ValidPassword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}
