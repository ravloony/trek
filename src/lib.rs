extern crate chrono;
extern crate postgres;

pub mod migration;
pub mod migration_index;

use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

use chrono::UTC;

use error::Error;

mod error;


// A type alias for the result type used by most of the methods in this crate's API.
pub type Result<T> = std::result::Result<T, Error>;

/// A convenience method that automates creating a new, empty database migration from a name and a
/// directory where the new migration file should be created.
///
/// # Examples:
///
/// ```
/// let migrations_dir = Path::new("src/db/migrations/");
/// match create_migration("create_users_table", migrations_dir) {
///     Ok(name) => println!("Created new migration named {}", name)
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
        try!(file.write_all(migration_template(name, &file_name).as_bytes()));
    }
    try!(update_migration_index(&file_name, name));
    Ok(file_name)
}

fn time_prefix() -> String {
    UTC::now().format("%Y%m%d%H%M%S").to_string()
}

/// Takes a name (e.g. "create_users_table"), a file name (e.g. "20150822_create_users_table.rs"),
/// and the schema version for a new migration and returns a string that can be written into the
/// new migration file to fill in all the boilerplate code a migration requires
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

use db::migrations::migration::Migration;

#[derive(Debug)]
pub struct {capitalized_name} {{
    file_name: String,
}}
impl {capitalized_name} {{
    pub fn new() -> Self {{
        {capitalized_name} {{
            file_name: \"{file_name}\".to_owned()
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
        write!(formatter, \"{{}}\", self.file_name)
    }}
}}
",
        file_name=file_name,
        capitalized_name=capitalized_name
    )
}

/// Automates adding a new database migration into the MigrationIndex's implementation of the
/// Default trait, so that this migration will be present in any MigrationIndex generated from the
/// Default constructor. Note that this function directly modifies a source code file, so be wary
/// of using it if you haven't checked migration_index.rs into source control.
fn update_migration_index(
    migration_file_name: &str, migration_class_name: &str
) -> io::Result<()> {
    let index_file_path = Path::new("src/db/migrations/migration_index.rs");
    let mut index_file_contents;
    {
        let mut index_file = try!(File::open(index_file_path));
        // try to be efficient and allocate roughly enough space for the file buffer immediately
        // (assumes each character is one byte, so it won't be perfect)
        index_file_contents = match index_file.metadata() {
            Ok(metadata) => {
                let bytes = metadata.len();
                let bytes: usize = if bytes > usize::max_value() as u64 {
                    usize::max_value()
                } else {
                    bytes as usize
                };
                String::with_capacity(bytes)
            },
            Err(_) => String::new()
        };
        try!(index_file.read_to_string(&mut index_file_contents));
    }

    index_file_contents.replace("

", &format!("
use super::{}::{}

", migration_file_name, migration_class_name
    ));

    index_file_contents.replace("
            ])
", &format!("
                Box::new({}::new()),
            ]))
", migration_class_name));

    {
        let mut index_file = try!(File::create(index_file_path));
        try!(index_file.write_all(index_file_contents.as_bytes()));
    }
    Ok(())
}
