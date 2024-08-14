mod m20240813_000001_create_widget_table;

pub use sea_orm_migration::prelude::*;

/// A migration implementation meant to be used for migrating
/// the database in its current state.
///
/// When writing new migrations, make sure to include them in this
/// migration.
///
/// sea-orm maintains a list of migrations in `seaql_migrations`.
/// Migrations here should always be placed in the order they are meant to be applied.
///
/// In the event that a full teardown / rebuild is needed, use `refresh()` as the
/// migration strategy.
///
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        // Migrations to apply should be placed in order of application here
        vec![Box::new(m20240813_000001_create_widget_table::Migration)]
    }
}
