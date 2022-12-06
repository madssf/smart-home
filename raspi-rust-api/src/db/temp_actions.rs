use anyhow::anyhow;
use bigdecimal::{BigDecimal, FromPrimitive, ToPrimitive};
use chrono::NaiveDateTime;
use sqlx::PgPool;
use uuid::Uuid;

use crate::db::DbError;
use crate::domain::{TempAction, TempActionType};

struct TempActionEntity {
    id: Uuid,
    room_ids: Vec<Uuid>,
    action: String,
    temp: Option<BigDecimal>,
    expires_at: NaiveDateTime,
}

fn parse_action_type(
    action_type: &str,
    db_temp: &Option<BigDecimal>,
) -> Result<TempActionType, anyhow::Error> {
    match action_type {
        "OFF" => Ok(TempActionType::OFF),
        "ON" => Ok(TempActionType::ON(
            db_temp.as_ref().map(|v| v.to_f64().unwrap()),
        )),
        _ => Err(anyhow!("Unknown temp action type: {}", action_type)),
    }
}

fn action_type_to_entity(action: &TempActionType) -> (String, Option<BigDecimal>) {
    match action {
        TempActionType::ON(temp) => (
            "ON".to_string(),
            temp.map(|t| BigDecimal::from_f64(t).unwrap()),
        ),
        TempActionType::OFF => ("OFF".to_string(), None),
    }
}

pub async fn get_temp_actions(pool: &PgPool) -> Result<Vec<TempAction>, DbError> {
    let entities: Vec<TempActionEntity> =
        sqlx::query_as!(TempActionEntity, "SELECT * FROM temp_actions")
            .fetch_all(pool)
            .await?;

    entities
        .iter()
        .map(|entity| {
            Ok(TempAction {
                id: entity.id,
                room_ids: entity.room_ids.clone(),
                action_type: parse_action_type(&entity.action, &entity.temp)?,
                expires_at: entity.expires_at,
            })
        })
        .collect()
}

pub async fn create_temp_action(pool: &PgPool, new_temp_action: TempAction) -> Result<(), DbError> {
    let (action_type, temp) = action_type_to_entity(&new_temp_action.action_type);
    sqlx::query!(
        r#"
    INSERT INTO temp_actions (id, room_ids, action, temp, expires_at)
    VALUES ($1, $2, $3, $4, $5)
    "#,
        new_temp_action.id,
        &new_temp_action.room_ids,
        action_type,
        temp,
        new_temp_action.expires_at,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn update_temp_action(pool: &PgPool, temp_action: TempAction) -> Result<(), DbError> {
    let (action_type, temp) = action_type_to_entity(&temp_action.action_type);

    sqlx::query!(
        r#"
    UPDATE temp_actions
    SET room_ids = $2, action = $3, temp = $4, expires_at = $5
    WHERE id = $1
    "#,
        temp_action.id,
        &temp_action.room_ids,
        action_type,
        temp,
        temp_action.expires_at,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_temp_action(pool: &PgPool, id: &Uuid) -> Result<(), DbError> {
    sqlx::query!(
        r#"
        DELETE FROM temp_actions WHERE id = $1
        "#,
        id
    )
    .execute(pool)
    .await?;

    Ok(())
}
