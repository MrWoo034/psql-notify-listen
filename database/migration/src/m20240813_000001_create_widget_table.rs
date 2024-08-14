use sea_orm::{DbBackend, Statement, StatementBuilder};
use sea_orm_migration::prelude::*;

const FK_WIDGET_TO_NOTIFICATION: &str = "FK-WIDGET_TO_NOTIFICATION";
const FK_WIDGET_TO_RECEIVED: &str = "FK-WIDGET_TO_RECEIVED";

struct TriggerFunction;

impl StatementBuilder for TriggerFunction {
    fn build(&self, db_backend: &DbBackend) -> Statement {
        let trigger_function =
            "CREATE OR REPLACE FUNCTION insert_and_notify()
                RETURNS TRIGGER
                LANGUAGE PLPGSQL
            AS $$
            DECLARE
                payload TEXT;
            BEGIN
                INSERT INTO notified_count_table (widget_id, timestamp) VALUES (NEW.id, NOW());
                payload := json_build_object('widget_id', NEW.id);
                PERFORM pg_notify('widget_notification', payload);
                RETURN NEW;
            END;
            $$;";
        Statement::from_string(*db_backend, trigger_function)
    }
}
struct Trigger;

impl StatementBuilder for Trigger {
    fn build(&self, db_backend: &DbBackend) -> Statement {
        let trigger_sql =
            "CREATE TRIGGER notified_count_insert_trigger
            AFTER INSERT ON widget_table
            FOR EACH ROW
            EXECUTE FUNCTION insert_and_notify();";
        Statement::from_string(*db_backend, trigger_sql)
    }
}

// struct Channel;
//
// impl StatementBuilder for Channel {
//     fn build(&self, db_backend: &DbBackend) -> Statement {
//         let channel_sql = "CREATE CHANNEL widget_notification;";
//         Statement::from_string(*db_backend, channel_sql)
//     }
// }

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(WidgetTable::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(WidgetTable::Id)
                            .big_integer()
                            .auto_increment()
                            .not_null()
                            .primary_key(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(NotifiedCountTable::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(NotifiedCountTable::Id)
                            .big_integer()
                            .auto_increment()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(NotifiedCountTable::WidgetId)
                            .big_integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(FK_WIDGET_TO_NOTIFICATION)
                            .from(NotifiedCountTable::Table, NotifiedCountTable::WidgetId)
                            .to(WidgetTable::Table, WidgetTable::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(
                        ColumnDef::new(NotifiedCountTable::Timestamp)
                            .timestamp()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ReceivedCountTable::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ReceivedCountTable::Id)
                            .big_integer()
                            .auto_increment()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                    ColumnDef::new(ReceivedCountTable::WidgetId)
                        .big_integer()
                        .not_null(),
                )
                    .foreign_key(
                        ForeignKey::create()
                            .name(FK_WIDGET_TO_RECEIVED)
                            .from(ReceivedCountTable::Table, ReceivedCountTable::WidgetId)
                            .to(WidgetTable::Table, WidgetTable::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(
                        ColumnDef::new(ReceivedCountTable::Timestamp)
                            .timestamp()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;
        let trigger_function = TriggerFunction;
        manager.exec_stmt(trigger_function).await?;
        let trigger = Trigger;
        manager.exec_stmt(trigger).await
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

#[derive(DeriveIden)]
enum WidgetTable {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum NotifiedCountTable {
    Table,
    Id,
    WidgetId,
    Timestamp,
}

#[derive(DeriveIden)]
enum ReceivedCountTable {
    Table,
    Id,
    WidgetId,
    Timestamp,
}
