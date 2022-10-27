use std::time::Duration;

use log::info;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use thiserror::Error;

pub mod plugs;
pub mod rooms;
pub mod schedules;
pub mod temp_actions;
pub mod temperature_logs;

#[derive(Debug, Clone)]
pub struct DbConfig {
    pub pool: PgPool,
}

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Error with DB client: {0}")]
    SQLXClientFailure(#[from] sqlx::Error),
    #[error("Error when running migrations: {0}")]
    MigrationFailure(#[from] sqlx::migrate::MigrateError),
    #[error("Error when parsing: {0}")]
    ParseError(#[from] anyhow::Error),
}

pub static DB_MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("db/migrations");

impl DbConfig {
    pub async fn new(db_url: &str) -> Result<Self, DbError> {
        info!("Connecting to DB!");

        let pool = PgPoolOptions::new()
            .acquire_timeout(Duration::from_millis(200))
            .max_connections(10)
            .connect_lazy(db_url)?;

        DB_MIGRATOR.run(&pool).await?;

        info!("Finished migrations!");

        Ok(Self { pool })
    }
}
