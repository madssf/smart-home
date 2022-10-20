use std::str::FromStr;

use anyhow::Context;
use chrono::NaiveDateTime;
use uuid::Uuid;

use crate::db::{DbConfig, DbError};
use crate::domain::{ActionType, TempAction};

struct TempActionEntity {
    id: Uuid,
    room_ids: Vec<Uuid>,
    action: String,
    expires_at: NaiveDateTime,
}

pub struct TempActionsClient {
    db_config: DbConfig,
}

impl TempActionsClient {
    pub fn new(db_config: DbConfig) -> Self {
        Self { db_config }
    }

    pub async fn get_temp_actions(&self) -> Result<Vec<TempAction>, DbError> {
        let entities: Vec<TempActionEntity> =
            sqlx::query_as!(TempActionEntity, "SELECT * FROM temp_actions")
                .fetch_all(&self.db_config.pool)
                .await?;

        entities
            .iter()
            .map(|entity| {
                let action_type = ActionType::from_str(&entity.action)
                    .context(format!("Could not parse as Action: {}", entity.action))?;
                Ok(TempAction {
                    id: entity.id,
                    room_ids: entity.room_ids.clone(),
                    action_type,
                    expires_at: entity.expires_at,
                })
            })
            .collect()
    }

    pub async fn create_temp_action(&self, new_temp_action: TempAction) -> Result<(), DbError> {
        sqlx::query!(
            r#"
        INSERT INTO temp_actions (id, room_ids, action, expires_at)
        VALUES ($1, $2, $3, $4)
        "#,
            new_temp_action.id,
            &new_temp_action.room_ids,
            new_temp_action.action_type.to_string(),
            new_temp_action.expires_at,
        )
        .execute(&self.db_config.pool)
        .await?;

        Ok(())
    }

    pub async fn update_temp_action(&self, temp_action: TempAction) -> Result<(), DbError> {
        sqlx::query!(
            r#"
        UPDATE temp_actions
        SET room_ids = $2, action = $3, expires_at = $4
        WHERE id = $1
        "#,
            temp_action.id,
            &temp_action.room_ids,
            temp_action.action_type.to_string(),
            temp_action.expires_at,
        )
        .execute(&self.db_config.pool)
        .await?;

        Ok(())
    }

    pub async fn delete_temp_action(&self, id: &Uuid) -> Result<(), DbError> {
        sqlx::query!(
            r#"
            DELETE FROM temp_actions WHERE id = $1
            "#,
            id
        )
        .execute(&self.db_config.pool)
        .await?;

        Ok(())
    }
}
