extern crate chrono;
extern crate postgres;
extern crate trek;

use std::fmt::{self, Display};
use std::env;

use chrono::DateTime;
use postgres::{Connection, GenericConnection, SslMode, Transaction};

use trek::migration::Migration;
use trek::migration_version::MigrationVersion;
use trek::migration_index::MigrationIndex;


struct GoodMigration1 {
    version: MigrationVersion
}
impl GoodMigration1 {
    pub fn new() -> Self {
        GoodMigration1 {
            version: MigrationVersion::from_rfc3339_string("1992-06-02T15:30:00-08:00").unwrap()
        }
    }
}
impl Migration for GoodMigration1 {
    fn up(&self, transaction: &GenericConnection) -> postgres::Result<()> {
        try!(transaction.execute("CREATE TABLE data {
            good_migration_1_ran boolean NOT NULL DEFAULT false
        );", &[]));
        try!(transaction.execute("INSERT INTO data (good_migration_1_ran) values (true)", &[]));
        Ok(())
    }
    fn down(&self, transaction: &GenericConnection) -> postgres::Result<()> {
        try!(transaction.execute("DROP TABLE data;", &[]));
        Ok(())
    }
    fn version(&self) -> MigrationVersion {
        self.version
    }
}
impl Display for GoodMigration1 {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.version)
    }
}

// this migration depends on GoodMigration1 having been run
struct GoodMigration2 {
    version: MigrationVersion
}
impl GoodMigration2 {
    pub fn new() -> Self {
        GoodMigration2 {
            version: MigrationVersion::from_rfc3339_string("1993-06-02T15:30:00-08:00").unwrap()
        }
    }
}
impl Migration for GoodMigration2 {
    fn up(&self, transaction: &GenericConnection) -> postgres::Result<()> {
        try!(transaction.execute("ALTER TABLE data {
            ADD COLUMN good_migration_2_ran boolean NOT NULL DEFAULT false
        );", &[]));
        try!(transaction.execute("UPDATE data SET good_migration_2_ran = true;", &[]));
        Ok(())
    }
    fn down(&self, transaction: &GenericConnection) -> postgres::Result<()> {
        try!(transaction.execute("ALTER TABLE data {
            DROP COLUMN good_migration_2
        };", &[]));
        Ok(())
    }
    fn version(&self) -> MigrationVersion {
        self.version
    }
}
impl Display for GoodMigration2 {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.version)
    }
}

// this migration has a valid up() but its down() will fail
struct GoodMigrationUpBadMigrationDown {
    version: MigrationVersion
}
impl GoodMigrationUpBadMigrationDown {
    pub fn new() -> Self {
        GoodMigrationUpBadMigrationDown {
            version: MigrationVersion::from_rfc3339_string("1994-06-02T15:30:00-08:00").unwrap()
        }
    }
}
impl Migration for GoodMigrationUpBadMigrationDown {
    fn up(&self, transaction: &GenericConnection) -> postgres::Result<()> {
        try!(transaction.execute("CREATE TABLE independent_data {
            good_up_bad_down_migration_ran boolean NOT NULL DEFAULT FALSE
        };", &[]));
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
    fn version(&self) -> MigrationVersion {
        self.version
    }
}
impl Display for GoodMigrationUpBadMigrationDown {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.version)
    }
}

// this migration is expected to fail when run
struct BadMigration1 {
    version: MigrationVersion
}
impl BadMigration1 {
    pub fn new() -> Self {
        BadMigration1 {
            version: MigrationVersion::from_rfc3339_string("1995-06-02T15:30:00-08:00").unwrap()
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
    fn version(&self) -> MigrationVersion {
        self.version
    }
}
impl Display for BadMigration1 {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.version)
    }
}

/// Connects to the database specified by the TREK_TEST_DB_PARAMS environment variable
/// and returns a transaction on it.
fn new_test_connection() -> Connection {
    let db_params = env::var("TREK_TEST_DB_PARAMS")
        .ok().expect(
            "TREK_TEST_DB_PARAMS environment variable is unset. This environment variable should \
            contain a database connection string for a PostgreSQL database to use when testing. \
            This string should take the form:\n\
            postgresql://user[:password]@host[:port][/database][?param1=val1[[&param2=val2]...]]\n\
            See the rust-postgres documentation for more details:\n\
            https://sfackler.github.io/rust-postgres/doc/postgres/struct.Connection.html#method.connect\n"
        );
    Connection::connect(&*db_params, &SslMode::None).unwrap()
}

#[test]
fn can_run_migration() {
    let connection = new_test_connection();
    let transaction = connection.transaction().unwrap();
    let migration_index = MigrationIndex::new(
        vec![Box::new(GoodMigration1::new())]
    );
    assert!(migration_index.run(&transaction).is_ok());

    // check that the changes were applied
    let prepared_statement = transaction.prepare("SELECT good_migration_1_ran FROM data;")
        .unwrap();
    let result = prepared_statement.query(&[]).unwrap();
    assert_eq!(result.len(), 1);
    let migration_ran: bool = result.get(0).get(0);
    assert!(migration_ran);

    // check schema version is correct
    let schema_version = MigrationIndex::schema_version(&transaction).unwrap();
    assert!(schema_version.is_some());
    assert_eq!(
        schema_version.unwrap().to_string(),
        "1992-06-02T15:30:00-08:00"
    );
}

#[test]
fn can_rollback_migration() {
    let connection = new_test_connection();
    let transaction = connection.transaction().unwrap();
    let migration_index = MigrationIndex::new(
        vec![Box::new(GoodMigration1::new())]
    );
    assert!(migration_index.run(&transaction).is_ok());
    assert!(migration_index.rollback(&transaction).is_ok());

    // check that the changes were applied
    let prepared_statement = transaction.prepare(
            "SELECT table_name FROM information_schema.tables WHERE table_schema='public'; "
        ).unwrap();
    let result = prepared_statement.query(&[]).unwrap();
    assert_eq!(result.len(), 0);

    // check schema version is correct
    assert!(MigrationIndex::schema_version(&transaction).unwrap().is_none());
}

#[test]
fn can_apply_migrations_sequentially() {
    let connection = new_test_connection();
    let transaction = connection.transaction().unwrap();
    let migration_index = MigrationIndex::new(
        vec![
            Box::new(GoodMigration1::new()),
            Box::new(GoodMigration2::new()),
        ]
    );
    assert!(migration_index.run(&transaction).is_ok());

    // check that the changes were applied
    let prepared_statement = transaction.prepare("SELECT good_migration_2_ran FROM data;")
        .unwrap();
    let result = prepared_statement.query(&[]).unwrap();
    assert_eq!(result.len(), 1);
    let migration_ran: bool = result.get(0).get(0);
    assert!(migration_ran);

    // check schema version is correct
    let schema_version = MigrationIndex::schema_version(&transaction).unwrap();
    assert!(schema_version.is_some());
    assert_eq!(
        schema_version.unwrap().to_string(),
        "1993-06-02T15:30:00-08:00"
    );
}

#[test]
fn can_rollback_migrations_sequentially() {
    let connection = new_test_connection();
    let transaction = connection.transaction().unwrap();
    let migration_index = MigrationIndex::new(
        vec![
            Box::new(GoodMigration1::new()),
            Box::new(GoodMigration2::new()),
        ]
    );
    assert!(migration_index.run(&transaction).is_ok());
    assert!(migration_index.rollback(&transaction).is_ok());

    // check that only the last migration was rolled back
    let prepared_statement = transaction.prepare(
            "SELECT * FROM information_schema.columns
            WHERE table_schema = 'public' AND table_name = 'data';"
        ).unwrap();
    let result = prepared_statement.query(&[]).unwrap();
    assert_eq!(result.len(), 1);

    let migration_ran: String = result.get(0).get(0);
    assert_eq!(migration_ran, "good_migration_1_ran");
    let schema_version = MigrationIndex::schema_version(&transaction).unwrap();
    assert!(schema_version.is_some());
    assert_eq!(
        schema_version.unwrap().to_string(),
        "1992-06-02T15:30:00-08:00"
    );

    // now all migrations should be rolled back
    assert!(migration_index.rollback(&transaction).is_ok());
    let prepared_statement = transaction.prepare(
            "SELECT table_name FROM information_schema.tables WHERE table_schema='public'; "
        )
        .unwrap();
    let result = prepared_statement.query(&[]).unwrap();
    assert_eq!(result.len(), 0);
    assert!(MigrationIndex::schema_version(&transaction).unwrap().is_none());
}

#[test]
fn fails_gracefully_on_migration_run_error() {
    let connection = new_test_connection();
    let transaction = connection.transaction().unwrap();
    let migration_index = MigrationIndex::new(
        vec![Box::new(BadMigration1::new())]
    );
    assert!(migration_index.run(&transaction).is_err());
}

#[test]
fn fails_gracefully_on_migration_rollback_error() {
    let connection = new_test_connection();
    let transaction = connection.transaction().unwrap();
    let migration_index = MigrationIndex::new(
        vec![
            Box::new(GoodMigration1::new()),
            Box::new(GoodMigrationUpBadMigrationDown::new()),
        ]
    );
    assert!(migration_index.run(&transaction).is_ok());
    assert!(migration_index.rollback(&transaction).is_err());
}
