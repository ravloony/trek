extern crate chrono;
extern crate postgres;
extern crate trek;

use std::env;

use postgres::{Connection, SslMode};

use trek::migration_index::MigrationIndex;

use self::types::good_migration_1::GoodMigration1;
use self::types::good_migration_2::GoodMigration2;
use self::types::good_migration_up_bad_migration_down::GoodMigrationUpBadMigrationDown;
use self::types::bad_migration_1::BadMigration1;

mod types;

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
    Connection::connect(&*db_params, SslMode::None).unwrap()
}

#[test]
fn can_run_migration() {
    let connection = new_test_connection();
    let transaction = connection.transaction().unwrap();
    let migration_index = MigrationIndex::new(
        vec![Box::new(GoodMigration1::new())]
    );
    migration_index.run(&transaction).unwrap();

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
        schema_version.unwrap(),
        "GoodMigration1"
    );
}

#[test]
fn can_rollback_migration() {
    let connection = new_test_connection();
    let transaction = connection.transaction().unwrap();
    let migration_index = MigrationIndex::new(
        vec![Box::new(GoodMigration1::new())]
    );
    migration_index.run(&transaction).unwrap();
    migration_index.rollback(&transaction).unwrap();

    let schema_name_prepared_stmt = transaction.prepare("SELECT current_schema;").unwrap();
    let schema_name: String = schema_name_prepared_stmt.query(&[]).unwrap().get(0).get(0);

    // check that the changes were applied
    let prepared_statement = transaction.prepare(
            "SELECT table_name FROM information_schema.tables WHERE table_schema=$1; "
        ).unwrap();
    let result = prepared_statement.query(&[&schema_name]).unwrap();
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
    migration_index.run(&transaction).unwrap();

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
        schema_version.unwrap(),
        "GoodMigration2"
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
    migration_index.run(&transaction).unwrap();
    migration_index.rollback(&transaction).unwrap();

    let schema_name_prepared_stmt = transaction.prepare("SELECT current_schema;").unwrap();
    let schema_name: String = schema_name_prepared_stmt.query(&[]).unwrap().get(0).get(0);

    // check that only the last migration was rolled back
    let prepared_statement = transaction.prepare(
            "SELECT column_name FROM information_schema.columns
            WHERE table_name='data' AND table_schema=$1;"
        ).unwrap();
    let result = prepared_statement.query(&[&schema_name]).unwrap();
    assert_eq!(result.len(), 1);

    let migration_ran: String = result.get(0).get(0);
    assert_eq!(migration_ran, "good_migration_1_ran");
    let schema_version = MigrationIndex::schema_version(&transaction).unwrap();
    assert!(schema_version.is_some());
    assert_eq!(
        schema_version.unwrap(),
        "GoodMigration1"
    );

    // now all migrations should be rolled back
    migration_index.rollback(&transaction).unwrap();
    let prepared_statement = transaction.prepare(
            "SELECT table_name FROM information_schema.tables WHERE table_schema=$1;"
        )
        .unwrap();
    let result = prepared_statement.query(&[&schema_name]).unwrap();
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
    migration_index.run(&transaction).unwrap();
    assert!(migration_index.rollback(&transaction).is_err());
}
