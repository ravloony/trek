use std::fmt::{self, Display};

use postgres;

use trek::migration::Migration;

#[derive(Debug)]
pub struct CreateCompaniesTable {
    name: String,
}
impl CreateCompaniesTable {
    pub fn new() -> Self {
        CreateCompaniesTable {
            name: "20151008562095_create_companies_table".to_owned()
        }
    }
}
impl Migration for CreateCompaniesTable {
    fn up(&self, connection: &postgres::GenericConnection) -> postgres::Result<()> {
        try!(connection.execute("CREATE TABLE companies (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    address TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);", &[]));

        try!(connection.execute("ALTER TABLE users
    ADD COLUMN company_id INTEGER
    ADD CONSTRAINT fk_users_company_id FOREIGN KEY (company_id) REFERENCES companies (id)
;", &[]));

        Ok(())
    }

    fn down(&self, connection: &postgres::GenericConnection) -> postgres::Result<()> {
        try!(connection.execute("ALTER TABLE users DROP COLUMN company_id;", &[]));
        try!(connection.execute("DROP TABLE companies;", &[]));
        Ok(())
    }
}
impl Display for CreateCompaniesTable {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.name)
    }
}
