use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum Error {
    RegEx(regex::Error),
    Standard(Box<dyn std::error::Error>),
    MinReq(minreq::Error),
    Api(crate::ApiError),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (error_type, message) = match &self {
            Error::RegEx(error) => ("RegEx Error", error.to_string()),
            Error::Standard(error) => ("Standard", error.to_string()),
            Error::MinReq(error) => ("MinReq", error.to_string()),
            Error::Api(error) => ("API", error.to_string()),
        };
        write!(f, "Error: type: {error_type}; message: {message}")
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(match self {
            Error::RegEx(error) => error,
            Error::Standard(error) => error.as_ref(),
            Error::MinReq(error) => error,
            Error::Api(error) => error,
        })
    }
}

impl From<regex::Error> for Error {
    fn from(error: regex::Error) -> Self {
        Error::RegEx(error)
    }
}

impl From<minreq::Error> for Error {
    fn from(error: minreq::Error) -> Self {
        Error::MinReq(error)
    }
}

impl From<Box<dyn std::error::Error>> for Error {
    fn from(error: Box<dyn std::error::Error>) -> Self {
        Error::Standard(error)
    }
}

impl From<crate::ApiError> for Error {
    fn from(error: crate::ApiError) -> Self {
        Error::Api(error)
    }
}
