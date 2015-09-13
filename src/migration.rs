use std::fmt::Display;

use postgres;

use super::migration_version::MigrationVersion;
use postgres::Result;


pub trait Migration : Display {
    /// Applies this migration.
    fn up(&self, transaction: &postgres::GenericConnection) -> Result<()>;
    /// Undoes this migration.
    fn down(&self, transaction: &postgres::GenericConnection) -> Result<()>;
    /// Returns the database schema version corresponding to this migration.
    fn version(&self) -> MigrationVersion;
}
