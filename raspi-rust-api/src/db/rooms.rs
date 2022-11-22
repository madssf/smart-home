use bigdecimal::{BigDecimal, FromPrimitive, ToPrimitive};
use sqlx::{FromRow, PgPool, Row};
use sqlx::postgres::PgRow;
use uuid::Uuid;

use crate::domain::Room;

use super::DbError;

pub async fn get_rooms(pool: &PgPool) -> Result<Vec<Room>, DbError> {
    let res: Vec<Room> = sqlx::query_as::<_, Room>("SELECT * FROM rooms")
        .fetch_all(pool)
        .await?;
    Ok(res)
}

pub async fn create_room(pool: &PgPool, name: &str, min_temp: &Option<f64>) -> Result<(), DbError> {
    let uuid = Uuid::new_v4();
    let min_temp = min_temp.as_ref().map(|temp| BigDecimal::from_f64(*temp).unwrap());
    sqlx::query!(
        r#"
        INSERT INTO rooms (id, name, min_temp)
        VALUES ($1, $2, $3)
        "#,
        uuid,
        name,
        min_temp
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn update_room(pool: &PgPool, room: &Room) -> Result<(), DbError> {
    let min_temp = room
        .min_temp
        .map(|temp| BigDecimal::from_f64(temp).unwrap());
    sqlx::query!(
        r#"
        UPDATE rooms
        SET name = $2, min_temp = $3
        WHERE id = $1
        "#,
        room.id,
        room.name,
        min_temp,
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
            min_temp: match row.get::<Option<BigDecimal>, &str>("min_temp") {
                None => None,
                Some(temp) => temp.to_f64(),
            },
        })
    }
}
