use sqlx::postgres::PgRow;
use sqlx::{FromRow, PgPool, Row};
use uuid::Uuid;

use crate::domain::Room;

use super::DbError;

pub async fn get_rooms(pool: &PgPool) -> Result<Vec<Room>, DbError> {
    let res: Vec<Room> = sqlx::query_as::<_, Room>("SELECT * FROM rooms")
        .fetch_all(pool)
        .await?;
    Ok(res)
}

pub async fn create_room(pool: &PgPool, name: &str) -> Result<(), DbError> {
    let uuid = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO rooms (id, name)
        VALUES ($1, $2)
        "#,
        uuid,
        name,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn update_room(pool: &PgPool, room: &Room) -> Result<(), DbError> {
    sqlx::query!(
        r#"
        UPDATE rooms
        SET name = $2
        WHERE id = $1
        "#,
        room.id,
        room.name,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_room(pool: &PgPool, id: &Uuid) -> Result<(), DbError> {
    sqlx::query!(
        r#"
        DELETE FROM rooms WHERE id = $1
        "#,
        id
    )
    .execute(pool)
    .await?;

    Ok(())
}

impl FromRow<'_, PgRow> for Room {
    fn from_row(row: &PgRow) -> sqlx::Result<Self> {
        Ok(Self {
            id: row.get::<Uuid, &str>("id"),
            name: row.get("name"),
        })
    }
}
