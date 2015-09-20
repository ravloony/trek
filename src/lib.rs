#![doc(html_root_url = "https://starim.github.io/trek/")]

extern crate chrono;
extern crate postgres;

use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

use chrono::UTC;

pub mod error;
pub mod migration;
pub mod migration_index;


/// A type alias for the result type used by most of the methods in this crate's API.
pub type Result<T> = std::result::Result<T, self::error::Error>;

/// A convenience method that automates creating a new, empty database migration from a name and a
/// directory where the new migration file should be created.
///
/// # Examples:
///
/// ```no_run
/// # use std::path::Path;
/// # use trek::create_migration;
/// let migrations_dir = Path::new("src/db/migrations/");
/// match create_migration("create_users_table", migrations_dir) {
///     Ok(name) => println!("Created new migration named {}", name),
///     Err(error) => println!("Error creating new database migration: {}", error)
/// }
/// ```
pub fn create_migration(name: &str, migrations_dir: &Path) -> io::Result<String> {
    let file_name = format!("migration_{}_{}.rs", time_prefix(), name);
    let mut final_path = migrations_dir.to_path_buf();
    final_path.push(file_name.clone());
    let final_path = final_path.as_path();
    {
        let mut file = try!(File::create(final_path));
        try!(file.write_all(migration_template(name, &*file_name).as_bytes()));
    }
    Ok(file_name)
}

fn time_prefix() -> String {
    UTC::now().format("%Y%m%d%H%M%S").to_string()
}

/// Takes a name (e.g. "create_users_table"), a file name (e.g.
/// "20150822094521_create_users_table.rs"), and the schema version for a new migration and returns
/// a string that can be written into the new migration file to fill in all the boilerplate code a
/// migration requires
fn migration_template(name: &str, file_name: &str) -> String {
    // turns "my_migration" into "MyMigration"
    let capitalized_name = name.to_owned().split('_').flat_map(|word|
        word.chars().enumerate().flat_map(|input| {
            let index = input.0;
            let character = input.1;
            if index == 0 {
                // some exotic Unicode characters have an uppercase form composed of multiple
                // characters
                character.to_uppercase().collect()
            } else {
                vec!(character)
            }
        }).collect::<Vec<char>>()
    ).collect::<String>();

    format!("\
use std::fmt::{{self, Display}};
use postgres;
use trek::migration::Migration;

#[derive(Debug)]
pub struct {capitalized_name} {{
    name: String,
}}
impl {capitalized_name} {{
    pub fn new() -> Self {{
        {capitalized_name} {{
            name: \"{file_name}\".to_owned()
        }}
    }}
}}
impl Migration for {capitalized_name} {{
    fn up(&self, connection: &postgres::GenericConnection) -> postgres::Result<()> {{
        try!(connection.execute(\"Your SQL here.\", &[]));
        Ok(())
    }}

    fn down(&self, connection: &postgres::GenericConnection) -> postgres::Result<()> {{
        try!(connection.execute(\"Your SQL here.\", &[]));
        Ok(())
    }}
}}
impl Display for {capitalized_name} {{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {{
        write!(formatter, \"{{}}\", self.name)
    }}
}}
",
        file_name=file_name,
        capitalized_name=capitalized_name
    )
}
