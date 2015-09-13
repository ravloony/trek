use std::fmt::{Display, Formatter, Result};
use std;

use postgres;

#[derive(Debug)]
pub struct Error {
    message: String,
    cause: postgres::error::Error,
}

impl Error {

    #[allow(dead_code)]
    pub fn new(message: String, cause: postgres::error::Error) -> Self {
        Error {
            message: message,
            cause: cause
        }
    }

    #[allow(dead_code)]
    pub fn cause(&self) -> &postgres::error::Error {
        &self.cause
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        &*self.message
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> Result {
        write!(formatter, "{}. The specific error is: {}", self.message, self.cause)
    }
}
