use std::fmt::{self, Display};
use postgres::{self, GenericConnection};
use trek::migration::Migration;

// this migration is expected to fail when run
#[derive(Debug)]
pub struct BadMigration1 {
    name: String
}
impl BadMigration1 {
    pub fn new() -> Self {
        BadMigration1 {
            name: "BadMigration1".to_owned(),
        }
    }
}
impl Migration for BadMigration1 {
    fn up(&self, transaction: &GenericConnection) -> postgres::Result<()> {
        try!(transaction.execute("rargle blargle", &[]));
        Ok(())
    }
    fn down(&self, transaction: &GenericConnection) -> postgres::Result<()> {
        try!(transaction.execute("rargle blargle", &[]));
        Ok(())
    }
}
impl Display for BadMigration1 {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.name)
    }
}
