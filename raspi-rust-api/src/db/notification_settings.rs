use sqlx::PgPool;

use crate::db::DbError;
use crate::domain::NotificationSettings;

pub async fn get_notification_settings(
    pool: &PgPool,
) -> Result<Option<NotificationSettings>, DbError> {
    Ok(sqlx::query_as!(
        NotificationSettings,
        "SELECT * FROM notification_settings LIMIT 1"
    )
    .fetch_optional(pool)
    .await?)
}

pub async fn upsert_notification_settings(
    pool: &PgPool,
    settings: &NotificationSettings,
) -> Result<(), DbError> {
    sqlx::query!(
        r#"
        INSERT INTO notification_settings (max_consumption, max_consumption_timeout_minutes, ntfy_topic)
        VALUES ($1, $2, $3)
        ON CONFLICT (id) DO UPDATE
        SET max_consumption = $1, max_consumption_timeout_minutes = $2, ntfy_topic = $3
        "#,
        settings.max_consumption,
        settings.max_consumption_timeout_minutes,
        settings.ntfy_topic
    )
    .execute(pool)
    .await?;

    Ok(())
}
