use uuid::Uuid;

use crate::domain::Plug;

use super::DbConfig;
use super::DbError;

pub struct PlugsClient {
    db_config: DbConfig,
}

impl PlugsClient {
    pub fn new(db_config: DbConfig) -> Self {
        Self { db_config }
    }

    pub async fn get_plugs(&self) -> Result<Vec<Plug>, DbError> {
        let plugs: Vec<Plug> = sqlx::query_as!(Plug, "SELECT * FROM plugs")
            .fetch_all(&self.db_config.pool)
            .await?;

        Ok(plugs)
    }

    pub async fn create_plug(&self, new_plug: Plug) -> Result<(), DbError> {
        sqlx::query!(
            r#"
        INSERT INTO plugs (id, name, ip, username, password, room_id)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
            new_plug.id,
            new_plug.name,
            new_plug.ip,
            new_plug.username,
            new_plug.password,
            new_plug.room_id
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

    pub async fn update_plug(&self, plug: Plug) -> Result<(), DbError> {
        sqlx::query!(
            r#"
            UPDATE plugs
            SET name = $2, ip = $3, username = $4, password = $5, room_id = $6
            WHERE id = $1
            "#,
            plug.id,
            plug.name,
            plug.ip,
            plug.username,
            plug.password,
            plug.room_id
        )
        .execute(&self.db_config.pool)
        .await?;

        Ok(())
    }
}
