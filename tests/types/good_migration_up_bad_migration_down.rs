use std::fmt::{self, Display};
use postgres::{self, GenericConnection};
use trek::migration::Migration;

// this migration has a valid up() but its down() will fail
#[derive(Debug)]
pub struct GoodMigrationUpBadMigrationDown {
    name: String
}
impl GoodMigrationUpBadMigrationDown {
    pub fn new() -> Self {
        GoodMigrationUpBadMigrationDown {
            name: "GoodMigrationUpBadMigrationDown".to_owned(),
        }
    }
}
impl Migration for GoodMigrationUpBadMigrationDown {
    fn up(&self, transaction: &GenericConnection) -> postgres::Result<()> {
        try!(transaction.execute(
            "CREATE TABLE independent_data (
                good_up_bad_down_migration_ran boolean NOT NULL DEFAULT FALSE
            );",
            &[]
        ));
        try!(transaction.execute(
            "INSERT INTO independent_data (good_up_bad_down_migration_ran) values (true)",
            &[]
        ));
        Ok(())
    }
    fn down(&self, transaction: &GenericConnection) -> postgres::Result<()> {
        try!(transaction.execute("rargle blargle", &[]));
        Ok(())
    }
}
impl Display for GoodMigrationUpBadMigrationDown {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.name)
    }
}
