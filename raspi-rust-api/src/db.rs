use std::str::FromStr;

use chrono::{Duration as CDuration, NaiveDateTime, NaiveTime, Weekday};
use gcp_auth::Error as GCPAuthError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::clients::FirestoreClient;
use crate::scheduling::ScheduleData;
use crate::{config_env_var, ActionType, Plug, PriceLevel, TempAction};

const SCHEDULES_COLLECTION_NAME: &str = "schedules";
const PLUGS_COLLECTION_NAME: &str = "plugs";
const TEMP_ACTIONS_COLLECTION__NAME: &str = "temp_actions";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScheduleEntity {
    pub id: String,
    pub price_level: String,
    pub days: Vec<String>,
    pub hours: Vec<HoursEntity>,
}

impl ScheduleEntity {
    pub fn to_domain(&self) -> ScheduleData {
        ScheduleData {
            price_level: PriceLevel::from_str(&self.price_level)
                .expect("Failed to parse price level"),
            days: self
                .days
                .iter()
                .map(|day| Weekday::from_str(day).expect("Failed to parse weekday"))
                .collect::<Vec<Weekday>>(),
            windows: self
                .hours
                .iter()
                .map(|x| {
                    let start = NaiveTime::from_str(&format!("{}:00", x.from))
                        .expect("Failed to parse NaiveTime");
                    let stop = NaiveTime::from_str(&format!("{}:00", x.to))
                        .expect("Failed to parse NaiveTime");
                    let duration = stop.signed_duration_since(start);
                    (start, duration)
                })
                .collect::<Vec<(NaiveTime, CDuration)>>(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HoursEntity {
    pub from: String,
    pub to: String,
}
#[derive(Error, Debug)]
pub enum DbError {
    #[error("Unexpected json format")]
    UnexpectedJsonFormat,
    #[error("AuthClientError: {0}")]
    AuthClientError(#[from] GCPAuthError),
    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("Parse error: {0}")]
    EnumParseError(#[from] strum::ParseError),
    #[error("Parse error: {0}")]
    TimeParseError(#[from] chrono::ParseError),
}

pub async fn get_schedules(
    firestore_client: &FirestoreClient,
) -> Result<Vec<ScheduleData>, DbError> {
    match get_schedule_entities(firestore_client).await {
        Ok(entities) => Ok(entities
            .iter()
            .map(ScheduleEntity::to_domain)
            .collect::<Vec<ScheduleData>>()),
        Err(e) => Err(e),
    }
}

async fn get_schedule_entities(
    firestore_client: &FirestoreClient,
) -> Result<Vec<ScheduleEntity>, DbError> {
    let schedules: Result<Vec<ScheduleEntity>, DbError> =
        match get_documents(firestore_client, SCHEDULES_COLLECTION_NAME).await {
            Err(e) => return Err(e),
            Ok(documents) => documents.iter().map(parse_as_schedule).collect(),
        };
    schedules
}

pub async fn get_plugs(firestore_client: &FirestoreClient) -> Result<Vec<Plug>, DbError> {
    match get_documents(firestore_client, PLUGS_COLLECTION_NAME).await {
        Err(e) => Err(e),
        Ok(documents) => documents.iter().map(parse_as_plug).collect(),
    }
}

pub async fn get_temp_actions(
    firestore_client: &FirestoreClient,
) -> Result<Vec<TempAction>, DbError> {
    match get_documents(firestore_client, TEMP_ACTIONS_COLLECTION__NAME).await {
        Ok(documents) => documents.iter().map(parse_as_temp_action).collect(),
        Err(e) => Err(e),
    }
}

pub async fn delete_temp_action(
    firestore_client: &FirestoreClient,
    plug_id: &str,
) -> Result<(), DbError> {
    delete_document(firestore_client, TEMP_ACTIONS_COLLECTION__NAME, plug_id).await
}

fn get_base_url(collection_name: &str) -> String {
    let project_id = config_env_var("PROJECT_ID");
    let user_id = config_env_var("USER_ID");

    format!(
        "https://firestore.googleapis.com/v1/projects/{}/databases/(default)/documents/users/{}/{}",
        project_id, user_id, collection_name
    )
}

async fn get_documents(
    firestore_client: &FirestoreClient,
    collection_name: &str,
) -> Result<Vec<Value>, DbError> {
    let res: Value = firestore_client
        .client
        .get(get_base_url(collection_name))
        .header(
            "Authorization",
            format!("Bearer {}", firestore_client.get_token().await?.as_str()),
        )
        .send()
        .await?
        .json()
        .await?;

    let documents = match res.get("documents") {
        None => return Err(DbError::UnexpectedJsonFormat),
        Some(docs) => docs,
    };

    match documents.as_array() {
        None => Err(DbError::UnexpectedJsonFormat),
        Some(docs_array) => Ok(docs_array.clone()),
    }
}

async fn delete_document(
    firestore_client: &FirestoreClient,
    collection_name: &str,
    document_id: &str,
) -> Result<(), DbError> {
    let url = format!("{}/{}", get_base_url(collection_name), document_id);
    firestore_client
        .client
        .delete(url)
        .header(
            "Authorization",
            format!("Bearer {}", firestore_client.get_token().await?.as_str()),
        )
        .send()
        .await?
        .error_for_status()?;
    Ok(())
}

fn parse_as_temp_action(doc_json: &Value) -> Result<TempAction, DbError> {
    let id = get_document_id(doc_json)?;
    let plug_ids = get_string_vec(doc_json, "fields/plug_ids")?;
    let action_type = ActionType::from_str(&get_string_value(doc_json, "/fields/action_type")?)?;
    let expires_at = NaiveDateTime::from_str(&get_string_value(doc_json, "/fields/expires_at")?)?;
    Ok(TempAction {
        id,
        plug_ids,
        action_type,
        expires_at,
    })
}

fn parse_as_plug(doc_json: &Value) -> Result<Plug, DbError> {
    let id = get_document_id(doc_json)?;
    let name = get_string_value(doc_json, "/fields/name")?;
    let ip = get_string_value(doc_json, "/fields/ip")?;
    let username = get_string_value(doc_json, "/fields/username")?;
    let password = get_string_value(doc_json, "/fields/password")?;
    Ok(Plug {
        id,
        name,
        ip,
        username,
        password,
    })
}

fn parse_as_schedule(doc_json: &Value) -> Result<ScheduleEntity, DbError> {
    let days = get_string_vec(doc_json, "fields/days")?;
    let hours: Result<Vec<HoursEntity>, DbError> =
        match doc_json.pointer("/fields/hours/arrayValue/values") {
            None => return Err(DbError::UnexpectedJsonFormat),
            Some(hours) => match hours.as_array() {
                None => return Err(DbError::UnexpectedJsonFormat),
                Some(hours_obj) => hours_obj
                    .iter()
                    .map(|hour| {
                        let from = get_string_value(hour, "/mapValue/fields/from")?;
                        let to = get_string_value(hour, "/mapValue/fields/to")?;
                        Ok(HoursEntity { from, to })
                    })
                    .collect(),
            },
        };

    let hours = match hours {
        Ok(hours) => hours,
        Err(e) => return Err(e),
    };

    let price_level = get_string_value(doc_json, "/fields/priceLevel")?;

    let id = get_document_id(doc_json)?;

    Ok(ScheduleEntity {
        id,
        price_level,
        days,
        hours,
    })
}

fn get_string_value(value: &Value, field_path: &str) -> Result<String, DbError> {
    match value.pointer(&*format!("{}/stringValue", field_path)) {
        None => Err(DbError::UnexpectedJsonFormat),
        Some(maybe_str) => match maybe_str.as_str() {
            None => Err(DbError::UnexpectedJsonFormat),
            Some(str) => Ok(str.to_string()),
        },
    }
}

fn get_string_vec(json: &Value, field_path: &str) -> Result<Vec<String>, DbError> {
    match json.pointer(&format!("/{}/arrayValue/values", field_path)) {
        None => Err(DbError::UnexpectedJsonFormat),
        Some(value) => match value.as_array() {
            None => Err(DbError::UnexpectedJsonFormat),
            Some(json_vec) => json_vec
                .iter()
                .map(|value| get_string_value(value, ""))
                .collect(),
        },
    }
}

fn get_document_id(value: &Value) -> Result<String, DbError> {
    match value.pointer("/name") {
        None => Err(DbError::UnexpectedJsonFormat),
        Some(id) => match id.as_str() {
            None => Err(DbError::UnexpectedJsonFormat),
            Some(id) => match id.split('/').last() {
                None => Err(DbError::UnexpectedJsonFormat),
                Some(id) => Ok(id.to_string()),
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, NaiveTime, Weekday};

    use crate::db::{HoursEntity, ScheduleEntity};
    use crate::PriceLevel;

    use super::ScheduleData;

    #[test]
    fn should_parse_to_domain() {
        let entity = ScheduleEntity {
            id: "test-id".to_string(),
            price_level: String::from("CHEAP"),
            days: vec![String::from("MON"), String::from("TUE")],
            hours: vec![HoursEntity {
                from: String::from("13:00"),
                to: String::from("14:00"),
            }],
        };
        let expected = ScheduleData {
            price_level: PriceLevel::CHEAP,
            days: vec![Weekday::Mon, Weekday::Tue],
            windows: vec![(NaiveTime::from_hms(13, 0, 0), Duration::hours(1))],
        };
        assert_eq!(ScheduleEntity::to_domain(&entity), expected)
    }
}
