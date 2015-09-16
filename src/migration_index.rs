use std::vec::Vec;

use postgres::{self, GenericConnection};

use super::error::Error;
use super::migration::Migration;

use super::Result;


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
    pub fn run(&self, connection: &GenericConnection) -> Result<()> {
        let mut schema_version = match MigrationIndex::schema_version(connection) {
            Ok(schema_version_option) => schema_version_option,
            Err(error) => {
                return Err(Error::new(
                    "Error reading current schema version".to_owned(),
                    error
                ));
            }
        };
        for migration in self.outstanding_migrations(schema_version.clone()).iter() {
            if let Err(error) = MigrationIndex::update_schema_version(
                connection, schema_version, Some(migration.to_string())
            ) {
                return Err(Error::new(
                    "Error updating schema version".to_owned(),
                    error
                ));
            }
            if let Err(error) = migration.up(connection) {
                return Err(Error::new(
                    format!("Error applying migration {}", migration),
                    error
                ));
            }
            schema_version = Some(migration.to_string());

            println!("Ran migration {}", migration);
        };
        Ok(())
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
    pub fn rollback(&self, connection: &GenericConnection) -> Result<()> {
        let old_schema_version = match MigrationIndex::schema_version(connection) {
            Ok(schema_version_option) => schema_version_option,
            Err(error) => {
                return Err(Error::new(
                    "Failed to get current database schema version".to_owned(),
                    error
                ))
            }
        };
        let old_schema_version = match old_schema_version {
            Some(schema_version) => schema_version,
            None => {
                // if there's nothing to roll back, this function call is a no-op
                return Ok(());
            }
        };
        let old_migration_index = self.current_index(&old_schema_version).unwrap();
        let old_migration = self.migrations.get(old_migration_index).unwrap();
        match old_migration_index {
            0 => {
                if let Err(error) = MigrationIndex::update_schema_version(
                    connection, Some(old_migration.to_string()), None
                ) {
                    return Err(Error::new(
                        format!(
                            "Failed to update schema version table when rolling back migration {}",
                            old_migration,
                        ),
                        error
                    ));
                }
                if let Err(error) = old_migration.down(connection) {
                    return Err(Error::new(
                        format!(
                            "The down() method of database migration {} failed",
                            old_migration,
                        ),
                        error
                    ));
                }
                println!(
                    "Rolled back migration {}, database is now empty.",
                    old_migration
                );
                Ok(())
            },
            _ => {
                let new_migration = self.migrations.get(old_migration_index - 1).unwrap();
                if let Err(error) = MigrationIndex::update_schema_version(
                    connection, Some(old_migration.to_string()), Some(new_migration.to_string())
                ) {
                    return Err(Error::new(
                        format!(
                            "Failed to update schema version table when rolling back migration {}",
                            new_migration,
                        ),
                        error
                    ));
                }
                if let Err(error) = old_migration.down(connection) {
                    return Err(Error::new(
                        format!(
                            "The down() method of database migration {} failed",
                            old_migration,
                        ),
                        error
                    ));
                }
                println!(
                    "Rolled back migration {}, database is now at version {}",
                    old_migration,
                    new_migration
                );
                Ok(())
            }
        }
    }

    /// Takes a queryable connection object and returns the current version of the database's
    /// schema. Panics if the queries it runs against the database fail.
    pub fn schema_version(
        connection: &GenericConnection
    ) -> postgres::Result<Option<String>> {
        let prepared_stmt = try!(connection.prepare(
            "SELECT column_name FROM information_schema.columns
            WHERE table_name=$1 LIMIT 1"
        ));
        let result = try!(prepared_stmt.query(&[&"schema_version"]));
        match result.len() {
            0 => Ok(None),
            1 => {
                let version_string: String = result.get(0).get_opt(0).unwrap();
                Ok(Some(version_string))
            },
            _ => panic!(
                    "Failed to retrieve current database schema version. The query to get column name \
                    for version tracking table returned multiple rows."
            )
        }
    }

    /// Takes the current version of the database's schema and returns a slice containing all
    /// migrations not yet applied to the database, in order from first to last.
    fn outstanding_migrations(&self, current_version: Option<String>) -> &[Box<Migration>] {
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
    fn current_index(&self, current_version: &str) -> Option<usize> {
        self.migrations.iter().position(|ref migration| {
            migration.to_string() == *current_version
        })
    }

    /// Takes a queryable connection object and uses it to record a new schema version in the
    /// database's version table.
    fn update_schema_version(
        connection: &GenericConnection,
        old_version: Option<String>,
        new_version: Option<String>
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
                    &[]
                ));
            },
            (Some(_old_version), None) => {
                try!(connection.execute("DROP TABLE schema_version;", &[]));
            },
            (None, None) => {
                // technically going from no database schema to no database schema is a no-op, but
                // it probably indicates a bug so panic on this questionable input
                panic!(
                    "Can't update schema version from None to None: at least one of old_version \
                    and new_version parameters must be Some"
                );
            }
        }
        Ok(())
    }
}
