extern crate docopt;
extern crate postgres;
extern crate rustc_serialize;
extern crate trek;

mod migration_index;
mod migrations;

use std::env;
use std::path::Path;
use docopt::Docopt;
use postgres::{Connection, TlsMode};
use postgres::error::ConnectError;
use self::migration_index::MigrationIndex;

const USAGE: &'static str = "
example - an example program showing off the trek library's features.

Usage:
  example [-h]
  example trek migrate [-h]
  example trek rollback [-h]
  example trek g migration <name> [-h]
  example trek generate migration <name> [-h]

Options:
  -h --help        Show help text.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    cmd_trek: bool,
    cmd_migrate: bool,
    arg_name: String,
    cmd_rollback: bool,
    cmd_g: bool,
    cmd_generate: bool,
    cmd_migration: bool,
}


fn should_run_migrations(args: &Args) -> bool {
    args.cmd_trek && args.cmd_migrate
}

fn should_rollback_migrations(args: &Args) -> bool {
    args.cmd_trek && args.cmd_rollback
}

fn should_generate_migrations(args: &Args) -> bool {
    args.cmd_trek && (args.cmd_g || args.cmd_generate) && args.cmd_migration
}

/// Creates and returns a new database connection, or an error if a connection could not be
/// established.
pub fn new_connection() -> Result<Connection, ConnectError> {
    // use the same database as the tests, for convenience
    let db_params = env::var("TREK_TEST_DB_PARAMS")
        .ok().expect(
            "TREK_TEST_DB_PARAMS environment variable is unset. This environment variable should \
            contain a database connection string for a PostgreSQL database to use when testing. \
            This string should take the form:\n\
            postgresql://user[:password]@host[:port][/database][?param1=val1[[&param2=val2]...]]\n\
            See the rust-postgres documentation for more details:\n\
            https://sfackler.github.io/rust-postgres/doc/postgres/struct.Connection.html#method.connect\n"
        );
    Connection::connect(&*db_params, TlsMode::None)
}

fn main() {
    let args: Args =
        Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());
    if should_run_migrations(&args) {
        let migrations: MigrationIndex = Default::default();
        match new_connection() {
            Err(error) => {
                panic!("Failed to get a connection from the pool: {}", error);
            }
            Ok(ref connection) => {
                match connection.transaction() {
                    Err(error) => {
                        panic!("Failed to start database transaction: {}", error);
                    }
                    Ok(transaction) => {
                        match migrations.run(&transaction) {
                            Err(error) => {
                                panic!("Error running database migrations: {}", error);
                            }
                            Ok(()) => {
                                match transaction.commit() {
                                    Err(error)=> {
                                        panic!("Failed to commit database transaction: {}", error);
                                    }
                                    Ok(_) => {
                                        println!(
                                            "All outstanding database migrations have been applied."
                                        );
                                        return;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    } else if should_rollback_migrations(&args) {
        let migrations: MigrationIndex = Default::default();
        match new_connection() {
            Err(error) => {
                panic!("Failed to get a connection from the pool: {}", error);
            }
            Ok(ref connection) => {
                match connection.transaction() {
                    Err(error) => {
                        panic!("Failed to start database transaction: {}", error);
                    }
                    Ok(transaction) => {
                        match migrations.rollback(&transaction) {
                            Err(error) => {
                                panic!("Error running database migrations: {}", error);
                            }
                            Ok(()) => {
                                match transaction.commit() {
                                    Err(error)=> {
                                        panic!("Failed to commit database transaction: {}", error);
                                    }
                                    Ok(_) => {
                                        println!(
                                            "All outstanding database migrations have been applied."
                                        );
                                        return;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    } else if should_generate_migrations(&args) {
        // generate a new empty migration
        let migration_dir = Path::new("examples/migrations/");
        match trek::create_migration(&args.arg_name, &migration_dir) {
            Ok(name) => {
                println!("Created migration {}", name);
                std::process::exit(0)
            },
            Err(error) => {
                println!("Error generating new database migration: {}", error);
                std::process::exit(1)
            }
        }
    } else {
        // your program logic here
        println!("Run main program");
    }
}
