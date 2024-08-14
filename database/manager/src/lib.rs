use migration_lib::MigratorTrait;
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection, DbBackend, DbErr, Statement, ActiveValue, EntityTrait};
use std::time::Duration;
use entities::widget_table::ActiveModel as WidgetActive;
use entities::received_count_table::ActiveModel as ReceivedActive;

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub username: String,
    pub password: String,
    pub host: String,
    pub port: String,
    pub db_name: String,
    pub enable_sql_logging: bool,
    pub sql_logging_level: log::LevelFilter,
    pub max_connections: u32,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            username: std::env::var("DB_USERNAME").expect("DB_USERNAME not set"),
            password: std::env::var("DB_PASSWORD").expect("DB_PASSWORD not set"),
            host: std::env::var("DB_HOST").expect("DB_HOST not set"),
            port: std::env::var("DB_PORT").unwrap_or("5432".to_owned()),
            db_name: std::env::var("DB_NAME").expect("DB_NAME not set"),
            enable_sql_logging: std::env::var("ENABLE_SQL_LOG")
                .ok()
                .and_then(|enable| enable.parse::<bool>().ok())
                .unwrap_or(false),
            sql_logging_level: std::env::var("SQL_LOG_LEVEL")
                .ok()
                .and_then(|filter| filter.parse::<log::LevelFilter>().ok())
                .unwrap_or(log::LevelFilter::Error),
            max_connections: std::env::var("DB_MAX_CONNECTIONS")
                .ok()
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(8),
        }
    }
}

pub struct Manager {
    pub connection: DatabaseConnection,
}

impl Manager {
    pub async fn try_new(db_config: &DatabaseConfig, setup: bool) -> Result<Self, DbErr> {
        let handler = Self {
            connection: Self::try_new_db_connection(db_config, setup).await?,
        };
        if setup {
            handler.refresh_database().await?;
        } else {
            handler.update_schema().await?;
        }

        Ok(handler)
    }

    pub async fn update_schema(&self) -> Result<(), DbErr> {
        migration_lib::Migrator::up(&self.connection, None).await
    }

    pub async fn rollback_schema(&self) -> Result<(), DbErr> {
        migration_lib::Migrator::down(&self.connection, None).await
    }

    pub(crate) async fn refresh_database(&self) -> Result<(), DbErr> {
        migration_lib::Migrator::refresh(&self.connection).await
    }

    async fn try_new_db_connection(
        db_config: &DatabaseConfig,
        setup: bool,
    ) -> Result<DatabaseConnection, DbErr> {
        // the whole database URL string follows the following format:
        // "protocol://username:password@host:port/database"
        let database_url = if !setup {
            format!(
                "postgres://{}:{}@{}:{}/{}",
                db_config.username,
                db_config.password,
                db_config.host,
                db_config.port,
                db_config.db_name
            )
        } else {
            // If we are `setup` a new database, we need to connect to the root
            // postgres instance.  Our URL needs to be different.
            format!(
                "postgres://{}:{}@{}:{}",
                db_config.username, db_config.password, db_config.host, db_config.port
            )
        };

        let mut options: ConnectOptions = ConnectOptions::new(database_url.clone());
        // TODO Research logging; log level here is not filtering logs based on level set with RUST_LOG=
        options
            .max_connections(db_config.max_connections)
            .idle_timeout(Duration::from_secs(60))
            .max_lifetime(Duration::from_secs(60 * 60 * 24))
            .acquire_timeout(Duration::from_secs(2))
            .sqlx_logging(db_config.enable_sql_logging)
            .sqlx_logging_level(db_config.sql_logging_level);
        let connection = Database::connect(options).await?;

        if setup {
            // If setup, we pass a connection to the root instance (it will close when dropped)
            // and build a new connection with the properly formatted string once the appropriate
            // database of {db_name} has been created.
            Self::create_database(connection, &database_url, &db_config.db_name).await
        } else {
            Ok(connection)
        }
    }

    async fn create_database(
        db: DatabaseConnection,
        database_url: &str,
        database: &str,
    ) -> Result<DatabaseConnection, DbErr> {
        match db.get_database_backend() {
            DbBackend::Postgres => {
                db.execute(Statement::from_string(
                    db.get_database_backend(),
                    format!("DROP DATABASE IF EXISTS \"{}\";", database),
                ))
                    .await?;
                db.execute(Statement::from_string(
                    db.get_database_backend(),
                    format!("CREATE DATABASE \"{}\";", database),
                ))
                    .await?;

                let url = format!("{}/{}", database_url, database);
                Database::connect(&url).await
            }
            _ => Err(DbErr::Custom(
                "Unsupported Database Engine.  Only PostgreSQL supported.".to_string(),
            )),
        }
    }

    pub async fn new_widget(&self) -> Result<(), DbErr> {
        let widget = WidgetActive {
            id: ActiveValue::NotSet,
        };
        entities::widget_table::Entity::insert(widget).exec(&self.connection).await?;
        Ok(())
    }

    pub async fn insert_notified(&self, widget_id: i64) -> Result<(), DbErr> {
        let row = ReceivedActive {
            id: ActiveValue::NotSet,
            widget_id: ActiveValue::Set(widget_id),
            timestamp: ActiveValue::Set(chrono::offset::Utc::now().naive_utc()),
        };
        entities::received_count_table::Entity::insert(row).exec(&self.connection).await?;
        Ok(())
    }
}
