use std::str::FromStr;

use sqlx::postgres::PgRow;
use sqlx::types::ipnetwork::IpNetwork;
use sqlx::{FromRow, Row};
use uuid::Uuid;

use super::DbConfig;
use super::DbError;

#[derive(thiserror::Error, Debug)]
pub enum CreatePlugError {
    #[error("SQL Error: {0}")]
    UnknownToken(#[from] sqlx::Error),
    #[error("IP Parse error")]
    IpParseError,
}

pub struct Client {
    db_config: DbConfig,
}

impl Client {
    pub fn new(db_config: DbConfig) -> Self {
        Self { db_config }
    }

    pub async fn get_plugs(&self) -> Result<Vec<DbPlug>, DbError> {
        let res: Vec<DbPlug> = sqlx::query_as::<_, DbPlug>("SELECT * FROM plugs")
            .fetch_all(&self.db_config.pool)
            .await?;

        Ok(res)
    }

    pub async fn create_plug(
        &self,
        name: String,
        ip: String,
        username: String,
        password: String,
    ) -> Result<(), CreatePlugError> {
        let uuid = Uuid::new_v4();
        let ip = match IpNetwork::from_str(&ip) {
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
}

#[derive(Debug, Clone)]
pub struct DbPlug {
    pub id: Uuid,
    pub name: String,
    pub ip: String,
    pub username: String,
    pub password: String,
}

impl FromRow<'_, PgRow> for DbPlug {
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
