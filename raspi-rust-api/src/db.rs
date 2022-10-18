use log::info;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use thiserror::Error;

pub mod plugs;

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
}

pub static DB_MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("db/migrations");

impl DbConfig {
    pub async fn new(db_url: &str) -> Result<Self, DbError> {
        info!("Connecting to DB!");

        let pool = PgPoolOptions::new()
            .max_connections(3)
            .connect(db_url)
            .await?;

        DB_MIGRATOR.run(&pool).await?;

        info!("Finished migrations!");

        Ok(Self { pool })
    }
}
