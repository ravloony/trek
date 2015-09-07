use std::vec::Vec;

use postgres;

use super::migration::Migration;
use super::migration_version::MigrationVersion;


/// Tracks and manages database migrations for this system.
pub struct MigrationIndex {
    /// all database migrations, in order from first to last
    migrations: Vec<Box<Migration>>
}
impl MigrationIndex {
    /// Wrap the given Migrations list into a new MigrationIndex.
    #[allow(dead_code)]
    pub fn new(mut migrations: Vec<Box<Migration>>) -> Self {
        migrations.shrink_to_fit();
        MigrationIndex {
            migrations: migrations
        }
    }

    /// Runs all database migrations that haven't yet been applied to the database. Panics if any
    /// database migration failed or the current schema version can't be determined.
    ///
    /// # Examples
    ///
    /// ```
    /// use postgres::{Connection, Transaction};
    ///
    /// let connection = try!(Connection::connect("server url", &SslMode::None));
    /// let transaction = connection.transaction().unwrap();
    ///
    /// let migrations = MigrationIndex::new(migration_list);
    /// match migrations.run(&transaction) {
    ///     Ok(_) => {
    ///         try!(transaction.commit());
    ///         println!("All outstanding database migrations have been applied.");
    ///     },
    ///     Err(error) => {
    ///         // note: no need to manually roll back the transaction, it'll automatically roll
    ///         // back when the transaction variable goes out of scope
    ///
    ///         println!("Error updating database structure: {}", error);
    ///     }
    /// }
    ///
    /// ```
    pub fn run<T: postgres::GenericConnection>(&self, connection: &T) {
        let mut schema_version = MigrationIndex::schema_version(connection);
        for migration in self.outstanding_migrations(schema_version).iter() {
            let result = MigrationIndex::update_schema_version(
                connection, schema_version, Some(migration.version())
            );
            if result.is_err() {
                panic!(
                    "Failed to update schema version table: {}",
                    result.err().unwrap().to_string()
                );
            }
            migration.up(connection);
            schema_version = Some(migration.version());

            println!("Ran migration {}", migration.to_string());
        };
    }

    /// Rolls back the last database migration that was successfully applied to the database.
    /// Panics if the migration failed when being rolled back or if the current schema version
    /// can't be determined.
    ///
    /// # Examples
    ///
    /// ```
    /// use postgres::{Connection, Transaction};
    ///
    /// let connection = try!(Connection::connect("server url", &SslMode::None));
    /// let transaction = connection.transaction().unwrap();
    ///
    /// let migrations = MigrationIndex::new(migration_list);
    /// match migrations.rollback(&transaction) {
    ///     Ok(_) => {
    ///         try!(transaction.commit());
    ///         println!("Rollback of latest migration complete.");
    ///     },
    ///     Err(error) => {
    ///         // note: no need to manually roll back the transaction, it'll automatically roll
    ///         // back when the transaction variable goes out of scope
    ///
    ///         println!("Error error rolling back last applied migration: {}", error);
    ///     }
    /// }
    ///
    /// ```
    pub fn rollback<T: postgres::GenericConnection>(&self, connection: &T) {
        let old_schema_version = MigrationIndex::schema_version(connection);
        if old_schema_version.is_none() {
            println!("No database migrations applied, no rollback necessary.");
            return;
        }
        let old_schema_version = old_schema_version.unwrap();
        let old_migration_index = self.current_index(&old_schema_version).unwrap();
        let old_migration = self.migrations.get(old_migration_index).unwrap();
        // new_migration will be None if old_migration is the very first migration
        match self.migrations.get(old_migration_index - 1) {
            Some(new_migration) => {
                let result = MigrationIndex::update_schema_version(
                    connection, Some(old_migration.version()), Some(new_migration.version())
                );
                if result.is_err() {
                    panic!(
                        "Failed to update schema version table: {}",
                        result.err().unwrap().to_string()
                    );
                }
                old_migration.down(connection);
                println!(
                    "Rolled back migration {}, database is now at version {}",
                    old_migration.to_string(),
                    new_migration.to_string()
                );
            }
            None => {
                let result = MigrationIndex::update_schema_version(
                    connection, Some(old_migration.version()), None
                );
                if result.is_err() {
                    panic!(
                        "Failed to update schema version table: {}",
                        result.err().unwrap().to_string()
                    );
                }
                old_migration.down(connection);
                println!(
                    "Rolled back migration {}, database is now empty.",
                    old_migration.to_string()
                );
            }
        }
    }

    /// Takes a queryable connection object and returns the current version of the database's
    /// schema. Panics if the queries it runs against the database fail.
    pub fn schema_version<T: postgres::GenericConnection>(connection: &T) -> Option<MigrationVersion> {
        let prepared_stmt = connection.prepare(
            "SELECT column_name FROM information_schema.columns
            WHERE table_name=$1 LIMIT 1"
        ).unwrap();
        let result = prepared_stmt.query(&[&"schema_migrations"]).unwrap();
        match result.len() {
            0 => None,
            1 => {
                let version_string: String = result.get(0).get_opt(0).unwrap();
                Some(MigrationVersion::from_rfc3339_string(&version_string).unwrap())
            },
            _ => panic!(
                    "Failed to retrieve current database schema version. The query to get column name \
                    for version tracking table returned multiple rows."
            )
        }
    }

    /// Takes the current version of the database's schema and returns a slice containing all
    /// migrations not yet applied to the database, in order from first to last.
    fn outstanding_migrations(&self, current_version: Option<MigrationVersion>) -> &[Box<Migration>] {
        match current_version {
            Some(current_version) => {
                 match self.current_index(&current_version) {
                    Some(current_index) => {
                        &self.migrations[(current_index + 1)..]
                    }
                    None => {
                        &*self.migrations
                    }
                }
            }
            None => &*self.migrations
        }
    }

    /// Takes the current version of the database's schema and returns the index of the migrations
    /// field corresponding to the last applied database migration. Returns None if no migrations
    /// have been applied to the database yet.
    fn current_index(&self, current_version: &MigrationVersion) -> Option<usize> {
        self.migrations.iter().position(|ref migration| {
            migration.version() == *current_version
        })
    }

    /// Takes a queryable connection object and uses it to record a new schema version in the
    /// database's version table.
    fn update_schema_version<T: postgres::GenericConnection>(
        connection: &T,
        old_version: Option<MigrationVersion>,
        new_version: Option<MigrationVersion>
    ) -> postgres::Result<()> {
        match (old_version, new_version) {
            (Some(old_version), Some(new_version)) => {
                try!(connection.execute(
                    &format!(
                        "ALTER TABLE schema_version RENAME COLUMN \"{}\" TO \"{}\";",
                        &old_version, &new_version
                    ),
                    &[]
                ));
            },
            (None, Some(new_version)) => {
                try!(connection.execute(
                    &format!(
                        "CREATE TABLE schema_version (
                             \"{}\" INT NOT NULL
                        );",
                        &new_version
                    ),
                    &[])
                );
            },
            (Some(_old_version), None) => {
                try!(connection.execute("DROP TABLE schema_version;", &[]));
            },
            (None, None) => {
                panic!(
                    "Can't update schema version: at least one of old_version and new_version \
                    parameters must be Some(MigrationVersion)"
                );
            }
        }
        Ok(())
    }
}
