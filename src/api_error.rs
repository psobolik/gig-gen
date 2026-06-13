use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct ApiError {
    code: i32,
    reason: String,
}

impl ApiError {
    pub fn new(code: i32, reason: &str) -> Self {
        Self {
            code,
            reason: reason.to_owned()
        }
    }
}

impl Display for ApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "API Error: {} - {}", self.code, self.reason)
    }
}

impl std::error::Error for ApiError {
    fn description(&self) -> &str {
        self.reason.as_str()
    }
}