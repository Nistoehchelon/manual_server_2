use std::fmt::Debug;

#[derive(Debug)]
pub struct PublicError {
    pub message: String,
}
impl PublicError {
    pub fn new(s: String) -> Self {
        Self { message: s }
    }
}

#[derive(Debug)]
pub struct InternalAppError {
    pub message: String,
}
impl InternalAppError {
    pub fn new<E: Debug>(s: E) -> Self {
        let message = format!("{s:?}");
        println!("InternalAppError: {}", message.clone());
        Self { message }
    }
}
#[derive(Debug)]
pub struct ExpectedToGetButFailedError {
    pub message: String,
}
impl ExpectedToGetButFailedError {
    pub fn new<E: Debug>(s: E) -> Self {
        let message = format!("{s:?}");
        println!("Expected the thingie to exist but apparently it does not: {}", message.clone());
        Self { message }
    }
}
