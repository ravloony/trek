Trek ![Travis build status](https://travis-ci.org/starim/trek.svg?branch=master)
==

Database migration management for Rust programs.

All code in the `src` folder is licensed under LGPL v3+.
All code in the `examples` folder is available under the MIT license.


To use Trek from crates.io in your project, add it as a dependency in your
`Cargo.toml`:

```
[dependencies]
trek = "0.2.0"
```

Usage
--

The heart of Trek is the `MigrationIndex` class, which keeps track of all
migrations and manages applying and rolling them back. It's recommended that
you add an implementation of the `Default` trait in your program for
`MigrationIndex`. This allows creating a single source of truth for the list of
database migrations that exist for your program. Updating this default
implementation will cause any of your code that uses the `MigrationIndex`
default value to automatically become aware of new database migrations.
Unfortunately Rust doesn't allow implementing traits on structs from other
crates, so you'll have to create a `MigrationIndex` wrapper in your own program
and implemennt the `Default` trait on that. Fortunately the boilerplate code
for this is simple and can be copied from the example program's implementation
at `examples/migration_index.rs`. Remeber after copying to edit the `Default`
trait in that file and set it to an empty migration list.  When you add new
migrations, you'll add them to the `Default` trait in this file.

You'll also need a folder in your source tree for the migrations, and a
`mod.rs` file that exports them. Each migration is its own struct in its own
file in this folder.

For ease of use, check out the example program at `examples/example.rs` to see
how to hook Trek into your own program so you can use Trek's migration
management through your own program's CLI interface.


Creating Migrations
--

Note: Trek expects migration names to be snake-cased like `my_new_migration`.

If you integrated Trek into your program's CLI interface like the example
program does, then adding a new migration is as easy as `cargo run my_program
-- trek g migration new_migration_name` (if you want to try it out with the
example program, `cargo run --example example -- trek g migration
new_migration_name`). Otherwise you'll have to call `Trek::create_migration()`
programmatically, passing in the path to your migrations folder and the
migration's snake-cased name. Either way, you'll get a new migration skeleton
in your migrations folder. With the skeleton generated, there are a couple
manual steps to turning into a fully-ready migration:

1. Fill out the new migration skeleton with your SQL. The `up` method provides
   the SQL to apply the migration, and the `down` method provides the SQL to
   undo it.
2. Add a `pub mod <migration file name>` line to the `mod.rs` file in your migrations folder.
3. Update your MigrationIndex's `Default` impl to include the new migration.
   For an example, see the bottom of `examples/migration_index.rs`.


Running Migrations
--

The example program provides sample code at `example/example.rs` for
integrating Trek's facilities for applying and rolllig back migrations. It's
recommended that you copy this code into your own program so that you can apply
or roll back migrations from your own program's CLI interface.


Test Setup
--

Trek expects an empty PostgreSQL database that it can test against. To set it up for testing:

1. Create a new, empty database named `trek_test`: connect to your postgres database and run `create database trek_test;`
2. Set environment variables giving connection information for the new database: there's a skeleton shell script at `tests/env_vars_example.sh`. Copy this file to `tests/env_vars.sh` and edit `env_vars.sh` to have your database name, user name, and other information filled in. The connection parameters are expected to be in the [format used by the rust-postgres crate](https://sfackler.github.io/rust-postgres/doc/v0.10.0/postgres/struct.Connection.html#method.connect).
3. Load environment variables by running `source tests/env_vars.sh`
4. Run tests with `cargo test`
