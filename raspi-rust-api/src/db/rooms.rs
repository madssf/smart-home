use sqlx::postgres::PgRow;
use sqlx::{FromRow, Row};
use uuid::Uuid;

use crate::domain::Room;

use super::{DbConfig, DbError};

pub struct RoomsClient {
    db_config: DbConfig,
}

impl RoomsClient {
    pub fn new(db_config: DbConfig) -> Self {
        Self { db_config }
    }

    pub async fn get_rooms(&self) -> Result<Vec<Room>, DbError> {
        let res: Vec<Room> = sqlx::query_as::<_, Room>("SELECT * FROM rooms")
            .fetch_all(&self.db_config.pool)
            .await?;
        Ok(res)
    }

    pub async fn create_room(&self, name: &str) -> Result<(), DbError> {
        let uuid = Uuid::new_v4();
        sqlx::query!(
            r#"
            INSERT INTO rooms (id, name)
            VALUES ($1, $2)
            "#,
            uuid,
            name,
        )
        .execute(&self.db_config.pool)
        .await?;

        Ok(())
    }

    pub async fn update_room(&self, room: &Room) -> Result<(), DbError> {
        sqlx::query!(
            r#"
            UPDATE rooms
            SET name = $2
            WHERE id = $1
            "#,
            room.id,
            room.name,
        )
        .execute(&self.db_config.pool)
        .await?;

        Ok(())
    }

    pub async fn delete_room(&self, id: &Uuid) -> Result<(), DbError> {
        sqlx::query!(
            r#"
            DELETE FROM rooms WHERE id = $1
            "#,
            id
        )
        .execute(&self.db_config.pool)
        .await?;

        Ok(())
    }
}

impl FromRow<'_, PgRow> for Room {
    fn from_row(row: &PgRow) -> sqlx::Result<Self> {
        Ok(Self {
            id: row.get::<Uuid, &str>("id"),
            name: row.get("name"),
        })
    }
}
