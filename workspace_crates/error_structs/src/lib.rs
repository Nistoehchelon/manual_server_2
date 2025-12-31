#[derive(Debug)]
pub struct PublicError {
    pub message: String,
}
impl PublicError {
    pub fn new(s: String) -> Self {
        Self { message: s }
    }
}
