use std::sync::Arc;
use std::time::Duration;

use log::info;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use thiserror::Error;

use crate::db::plugs::PlugsClient;
use crate::db::rooms::RoomsClient;
use crate::db::schedules::SchedulesClient;
use crate::db::temp_actions::TempActionsClient;
use crate::db::temperature_logs::TemperatureLogsClient;

pub mod plugs;
pub mod rooms;
pub mod schedules;
pub mod temp_actions;
pub mod temperature_logs;

#[derive(Debug, Clone)]
pub struct DbConfig {
    pool: PgPool,
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
            .acquire_timeout(Duration::from_secs(10))
            .max_connections(10)
            .connect(db_url)
            .await?;

        DB_MIGRATOR.run(&pool).await?;

        info!("Finished migrations!");

        Ok(Self { pool })
    }
}

#[derive(Clone)]
pub struct DbClients {
    pub rooms: Arc<RoomsClient>,
    pub plugs: Arc<PlugsClient>,
    pub schedules: Arc<SchedulesClient>,
    pub temp_actions: Arc<TempActionsClient>,
    pub temperature_logs: Arc<TemperatureLogsClient>,
}

impl DbClients {
    pub fn new(db_config: &DbConfig) -> Self {
        Self {
            rooms: Arc::new(RoomsClient::new(db_config.clone())),
            plugs: Arc::new(PlugsClient::new(db_config.clone())),
            schedules: Arc::new(SchedulesClient::new(db_config.clone())),
            temp_actions: Arc::new(TempActionsClient::new(db_config.clone())),
            temperature_logs: Arc::new(TemperatureLogsClient::new(db_config.clone())),
        }
    }
}
