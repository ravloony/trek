use std::fmt::{self, Display};
use postgres::{self, GenericConnection};
use trek::migration::Migration;

#[derive(Debug)]
pub struct GoodMigration1 {
    name: String
}
impl GoodMigration1 {
    pub fn new() -> Self {
        GoodMigration1 {
            name: "GoodMigration1".to_owned(),
        }
    }
}
impl Migration for GoodMigration1 {
    fn up(&self, transaction: &GenericConnection) -> postgres::Result<()> {
        try!(transaction.execute(
            "CREATE TABLE data (
                good_migration_1_ran boolean NOT NULL DEFAULT false
            );",
            &[]
        ));
        try!(transaction.execute("INSERT INTO data (good_migration_1_ran) values (true);", &[]));
        Ok(())
    }
    fn down(&self, transaction: &GenericConnection) -> postgres::Result<()> {
        try!(transaction.execute("DROP TABLE data;", &[]));
        Ok(())
    }
}
impl Display for GoodMigration1 {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.name)
    }
}
