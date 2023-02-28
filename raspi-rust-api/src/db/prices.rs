use std::str::FromStr;

use bigdecimal::{BigDecimal, FromPrimitive, ToPrimitive};
use chrono::{Duration, DurationRound, NaiveDateTime};
use sqlx::PgPool;

use crate::db::DbError;
use crate::domain::{PriceInfo, PriceLevel};

struct PriceInfoEntity {
    pub amount: BigDecimal,
    pub currency: String,
    pub ext_price_level: String,
    pub price_level: Option<String>,
    pub starts_at: NaiveDateTime,
}

impl From<PriceInfoEntity> for PriceInfo {
    fn from(entity: PriceInfoEntity) -> Self {
        Self {
            amount: entity.amount.to_f64().expect("Failed to convert to f64"),
            currency: entity.currency.to_string(),
            ext_price_level: PriceLevel::from_str(&entity.ext_price_level)
                .expect("Failed to convert string to PriceLevel"),
            price_level: entity.price_level.map(|string| {
                PriceLevel::from_str(&string).expect("Failed to convert string to PriceLevel")
            }),
            starts_at: entity.starts_at,
        }
    }
}

pub async fn insert_prices(pool: &PgPool, prices: &Vec<PriceInfo>) -> Result<(), DbError> {
    let timestamps: Vec<NaiveDateTime> = prices.iter().map(|price| price.starts_at).collect();

    let mut tx = pool.begin().await?;

    sqlx::query!(
        r#"
        DELETE FROM prices WHERE starts_at = any($1)
        "#,
        &timestamps
    )
    .fetch_all(&mut tx)
    .await?;

    for price_info in prices {
        sqlx::query!(
            r#"
        INSERT INTO prices (starts_at, amount, currency, ext_price_level, price_level)
        VALUES ($1, $2, $3, $4, $5)
        "#,
            price_info.starts_at,
            BigDecimal::from_f64(price_info.amount).unwrap(),
            price_info.currency,
            price_info.ext_price_level.to_string(),
            price_info
                .price_level
                .map(|price_level| { price_level.to_string() }),
        )
        .execute(&mut tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

pub async fn get_price(pool: &PgPool, time: &NaiveDateTime) -> Result<Option<PriceInfo>, DbError> {
    let entity = sqlx::query_as!(
        PriceInfoEntity,
        "SELECT * FROM prices WHERE starts_at = $1",
        time.duration_trunc(Duration::hours(1))
            .expect("Failed to truncate timestamp")
    )
    .fetch_optional(pool)
    .await?;
    Ok(entity.map(|e| e.into()))
}

pub async fn get_prices(
    pool: &PgPool,
    from: &NaiveDateTime,
    to: &NaiveDateTime,
) -> Result<Vec<PriceInfo>, DbError> {
    let entities = sqlx::query_as!(
        PriceInfoEntity,
        "SELECT * FROM prices WHERE starts_at > $1 AND starts_at < $2",
        from,
        to
    )
    .fetch_all(pool)
    .await?;
    Ok(entities.into_iter().map(|e| e.into()).collect())
}
