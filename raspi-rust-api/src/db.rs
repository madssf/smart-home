use std::collections::HashMap;
use std::time::Duration;

use gcp_auth::AuthenticationManager;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub fn config_env_var(name: &str) -> Result<String, String> {
    std::env::var(name).map_err(|e| format!("{}: {}", name, e))
}

/*
pub fn get_collection_name(user_id: &str, collection: Collection) -> String {
    format!("{}/{}", user_id, collection.name())
}
 */
#[derive(Debug, Deserialize)]
struct FirestoreJson {
    documents: Vec<HashMap<String, HashMap<String, String>>>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
struct ScheduleEntity {
    id: String,
    level: String,
    days: Vec<String>,
    hours: Vec<HoursEntity>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct HoursEntity {
    from: String,
    to: String,
}

pub async fn get() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Create an instance
    let auth_manager = AuthenticationManager::new().await?;

    let scopes = &["https://www.googleapis.com/auth/datastore"];
    let token = auth_manager.get_token(scopes).await?;

    println!("{}", token.as_str());

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .expect("Failed to build client");

    let res = client
        .get("https://firestore.googleapis.com/v1/projects/smart-home-3d61e/databases/(default)/documents/users/wdUmP7v4kTbzRgXQPTgTwzXEOSt1/schedules")
        .header("Authorization", format!("Bearer {}", token.as_str()))
        .send().await?;

    let text1 = res.text().await?;

    dbg!(&text1);

    let v: Value = serde_json::from_str(&text1).expect("Failed to parse json");

    dbg!(v);

    const TEST_COLLECTION_NAME: &'static str = "schedules";

    Ok(())
}
