use std::fmt;
#[derive(Debug)]
pub struct Dx12Error {
    message: String,
}

impl Dx12Error {
    pub fn new(message: &str) -> Dx12Error {
        Dx12Error {
            message: message.to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn get_message(&self) -> &str {
        return &self.message;
    }
}

impl fmt::Display for Dx12Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for Dx12Error {}
