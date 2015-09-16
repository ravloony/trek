use std::fmt::{self, Display};
use postgres::{self, GenericConnection};
use trek::migration::Migration;

// this migration depends on GoodMigration1 having been run
#[derive(Debug)]
pub struct GoodMigration2 {
    name: String
}
impl GoodMigration2 {
    pub fn new() -> Self {
        GoodMigration2 {
            name: "GoodMigration2".to_owned(),
        }
    }
}
impl Migration for GoodMigration2 {
    fn up(&self, transaction: &GenericConnection) -> postgres::Result<()> {
        try!(transaction.execute(
            "ALTER TABLE data ADD COLUMN good_migration_2_ran boolean NOT NULL DEFAULT false;",
            &[]
        ));
        try!(transaction.execute("UPDATE data SET good_migration_2_ran = true;", &[]));
        Ok(())
    }
    fn down(&self, transaction: &GenericConnection) -> postgres::Result<()> {
        try!(transaction.execute(
            "ALTER TABLE data DROP COLUMN good_migration_2_ran;",
            &[]
        ));
        Ok(())
    }
}
impl Display for GoodMigration2 {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.name)
    }
}
