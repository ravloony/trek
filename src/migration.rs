use std::fmt::Display;

use postgres;

use postgres::Result;


pub trait Migration : Display {
    /// Applies this migration.
    fn up(&self, transaction: &postgres::GenericConnection) -> Result<()>;
    /// Undoes this migration.
    fn down(&self, transaction: &postgres::GenericConnection) -> Result<()>;
}
