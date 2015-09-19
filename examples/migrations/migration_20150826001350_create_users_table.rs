use std::fmt::{self, Display};

use postgres;

use trek::migration::Migration;

#[derive(Debug)]
pub struct CreateUsersTable {
    name: String,
}
impl CreateUsersTable {
    pub fn new() -> Self {
        CreateUsersTable {
            name: "20150826001350_create_users_table".to_owned()
        }
    }
}
impl Migration for CreateUsersTable {
    fn up(&self, connection: &postgres::GenericConnection) -> postgres::Result<()> {
        try!(connection.execute("CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    admin BOOLEAN NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);", &[]));

        Ok(())
    }

    fn down(&self, connection: &postgres::GenericConnection) -> postgres::Result<()> {
        try!(connection.execute("DROP TABLE users;", &[]));
        Ok(())
    }
}
impl Display for CreateUsersTable {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.name)
    }
}
