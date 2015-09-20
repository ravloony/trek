use std::fmt::{Display, Formatter, Result};
use std;

use postgres;

/// An Error type for wrapping database errors in a higher-level message. For example, a database
/// error may indicate a query failed but it would be more meaningful to provide a higher-level
/// error message explaining what the query was trying to do.
#[derive(Debug)]
pub struct Error {
    message: String,
    cause: postgres::error::Error,
}

impl Error {

    /// Wrap a new database error with a message.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # extern crate postgres;
    /// # extern crate trek;
    /// # fn main() {
    /// # use postgres;
    /// # use postgres::rows::Rows;
    /// # use trek::error::Error;
    /// fn count_inventory(connection: &postgres::Connection) -> trek::Result<u64> {
    ///     let inventory_query = connection.prepare("query SQL").unwrap();
    ///     match inventory_query.execute(&[]) {
    ///         Ok(result) => Ok(result),
    ///         Err(db_error) => {
    ///             Err(Error::new(
    ///                 "Failed to fetch inventory data".to_owned(),
    ///                 db_error
    ///             ))
    ///         }
    ///     }
    /// }
    /// # }
    /// ```
    pub fn new(message: String, cause: postgres::error::Error) -> Self {
        Error {
            message: message,
            cause: cause
        }
    }

    /// Get the original error.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # extern crate postgres;
    /// # extern crate trek;
    /// # fn main() {
    /// # use postgres::{self, Connection, SslMode};
    /// # use postgres::rows::Rows;
    /// # use trek::error::Error;
    /// # fn f() {
    /// # let connection = Connection::connect("server url", &SslMode::None).unwrap();
    /// # let inventory_query = connection.prepare("query SQL").unwrap();
    /// # match inventory_query.execute(&[]) {
    /// #     Ok(result) => println!("no op"),
    /// #     Err(db_error) => {
    /// let error = Error::new("Failed to fetch inventory data".to_owned(), db_error);
    /// println!("Problem communicating with the DB, the low-level error is: {}", error.cause());
    /// # }
    /// # }
    /// # }
    /// # }
    /// ```
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
