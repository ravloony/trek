use std::fmt::Display;

use postgres;

use super::migration_version::MigrationVersion;


pub trait Migration : Display {
    /// Applies this migration.
    fn up(&self, transaction: &postgres::GenericConnection);
    /// Undoes this migration.
    fn down(&self, transaction: &postgres::GenericConnection);
    /// Returns the database schema version corresponding to this migration.
    fn version(&self) -> MigrationVersion;
}
