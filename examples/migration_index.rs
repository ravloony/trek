use postgres::{self, GenericConnection};
use trek::migration_index::MigrationIndex as TrekMigrationIndex;
use trek::migration::Migration as TrekMigration;
use trek::Result;
use migrations::migration_20150826001350_create_users_table::CreateUsersTable;
use migrations::migration_20151008562095_create_companies_table::CreateCompaniesTable;

pub struct MigrationIndex {
    migrations: TrekMigrationIndex,
}

impl MigrationIndex {
    #[allow(dead_code)]
    pub fn new(migrations: Vec<Box<TrekMigration>>) -> Self {
        MigrationIndex {
            migrations: TrekMigrationIndex::new(migrations)
        }
    }

    #[allow(dead_code)]
    pub fn run(&self, connection: &GenericConnection) -> Result<()> {
        self.migrations.run(connection)
    }

    #[allow(dead_code)]
    pub fn rollback(&self, connection: &GenericConnection) -> Result<()> {
        self.migrations.rollback(connection)
    }

    #[allow(dead_code)]
    pub fn schema_version(
        connection: &GenericConnection
    ) -> postgres::Result<Option<String>> {
        TrekMigrationIndex::schema_version(connection)
    }
}

impl Default for MigrationIndex {
    fn default() -> MigrationIndex {
        MigrationIndex {
            migrations: TrekMigrationIndex::new(vec![
                // record your migrations here
                Box::new(CreateUsersTable::new()),
                Box::new(CreateCompaniesTable::new()),
            ])
        }
    }
}
