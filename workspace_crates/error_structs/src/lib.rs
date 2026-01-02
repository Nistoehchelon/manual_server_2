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
    pub fn new(s: String) -> Self {
        eprintln!("{s}");
        Self { message: s }
    }
}
