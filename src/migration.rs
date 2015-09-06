use std::fmt::Display;

use postgres;

use super::migration_version::MigrationVersion;


pub trait Migration : Display {
    /// Applies this migration.
    fn up(&self, transaction: &postgres::GenericConnection) -> postgres::Result<()>;
    /// Undoes this migration.
    fn down(&self, transaction: &postgres::GenericConnection) -> postgres::Result<()>;
    /// Returns the database schema version corresponding to this migration.
    fn version(&self) -> MigrationVersion;
}
