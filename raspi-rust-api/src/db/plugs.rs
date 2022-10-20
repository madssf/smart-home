use std::str::FromStr;

use sqlx::postgres::PgRow;
use sqlx::types::ipnetwork::IpNetwork;
use sqlx::{FromRow, Row};
use uuid::Uuid;

use crate::Plug;

use super::DbConfig;
use super::DbError;

#[derive(thiserror::Error, Debug)]
pub enum CreatePlugError {
    #[error("SQL Error: {0}")]
    UnknownToken(#[from] sqlx::Error),
    #[error("IP Parse error")]
    IpParseError,
}

pub struct PlugsClient {
    db_config: DbConfig,
}

impl PlugsClient {
    pub fn new(db_config: DbConfig) -> Self {
        Self { db_config }
    }

    pub async fn get_plugs(&self) -> Result<Vec<Plug>, DbError> {
        let res: Vec<Plug> = sqlx::query_as::<_, Plug>("SELECT * FROM plugs")
            .fetch_all(&self.db_config.pool)
            .await?;

        Ok(res)
    }

    pub async fn create_plug(
        &self,
        name: &str,
        ip: &str,
        username: &str,
        password: &str,
    ) -> Result<(), CreatePlugError> {
        let uuid = Uuid::new_v4();
        let ip = match IpNetwork::from_str(ip) {
            Ok(ip) => ip,
            Err(_) => return Err(CreatePlugError::IpParseError),
        };
        sqlx::query!(
            r#"
        INSERT INTO plugs (id, name, ip, username, password)
        VALUES ($1, $2, $3, $4, $5)
        "#,
            uuid,
            name,
            ip,
            username,
            password
        )
        .execute(&self.db_config.pool)
        .await?;

        Ok(())
    }

    pub async fn delete_plug(&self, id: &Uuid) -> Result<(), DbError> {
        sqlx::query!(
            r#"
            DELETE FROM plugs WHERE id = $1
            "#,
            id
        )
        .execute(&self.db_config.pool)
        .await?;

        Ok(())
    }

    pub async fn update_plug(
        &self,
        id: &Uuid,
        name: &str,
        ip: &str,
        username: &str,
        password: &str,
    ) -> Result<(), CreatePlugError> {
        let ip = match IpNetwork::from_str(ip) {
            Ok(ip) => ip,
            Err(_) => return Err(CreatePlugError::IpParseError),
        };
        sqlx::query!(
            r#"
            UPDATE plugs
            SET name = $2, ip = $3, username = $4, password = $5
            WHERE id = $1
            "#,
            id,
            name,
            ip,
            username,
            password
        )
        .execute(&self.db_config.pool)
        .await?;

        Ok(())
    }
}

impl FromRow<'_, PgRow> for Plug {
    fn from_row(row: &PgRow) -> sqlx::Result<Self> {
        Ok(Self {
            id: row.get::<Uuid, &str>("id"),
            name: row.get("name"),
            ip: row.get::<IpNetwork, &str>("ip").ip().to_string(),
            username: row.get("username"),
            password: row.get("password"),
        })
    }
}
