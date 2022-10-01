use std::env;
use std::time::Duration;

use chrono::{TimeZone, Utc};
use chrono_tz::Tz;
use tokio::time::sleep;

use crate::clients::{FirestoreClient, ShellyClient};
use crate::prices::PriceInfo;
use crate::scheduling::ActionType;
use crate::{db, prices, scheduling, shelly_client};

pub async fn start(
    firestore_client: &FirestoreClient,
    shelly_client: &ShellyClient,
    sleep_duration_minutes: u64,
) {
    loop {
        println!("\n-STARTING NEW RUN-\n");

        let utc = Utc::now().naive_utc();
        let tz: Tz = env::var("TIME_ZONE")
            .expect("Missing TIME_ZONE env var")
            .parse()
            .expect("Failed to parse timezone");
        let time = tz.from_utc_datetime(&utc).naive_local();

        println!("Current time: {}", &time);

        let price: PriceInfo = if let Ok(price) = prices::get_current_price().await {
            println!(
                "Current price: {} {}, level: {}",
                &price.amount,
                &price.currency,
                &price.level.to_string()
            );
            price
        } else {
            println!("Failed to get price, sleeping for 10 seconds");
            sleep(Duration::from_secs(10)).await;
            continue;
        };

        let action: ActionType =
            match scheduling::get_action(firestore_client, &price.level, &time).await {
                Ok(action) => {
                    println!("Got action: {}", action.to_string());
                    action
                }
                Err(e) => {
                    println!(
                        "Failed to get action, sleeping for 10 seconds, error: {}",
                        e
                    );
                    sleep(Duration::from_secs(10)).await;
                    continue;
                }
            };

        let plugs = match db::get_plugs(firestore_client).await {
            Ok(plugs) => plugs,
            Err(e) => {
                println!("{}", e);
                panic!("Failed to get plugs from DB");
            }
        };

        for plug in plugs {
            println!("Processing plug: {}", &plug.name);
            if let Ok(power_usage) = shelly_client::get_status(shelly_client, &plug).await {
                println!("Current power usage: {} W", power_usage);
                println!(
                    "Equals hourly price of: {:.3} {}",
                    price.amount / 1000.0 * power_usage,
                    price.currency
                );
            };

            match shelly_client::execute_action(shelly_client, &plug, &action).await {
                Ok(_) => println!(
                    "Action executed on plug {}: {}",
                    &plug.name,
                    &action.to_string()
                ),
                Err(e) => println!(
                    "Action failed on plug {}: {} - error: {}",
                    &plug.name,
                    &action.to_string(),
                    e,
                ),
            }
        }

        println!("\n-RUN FINISHED, SLEEPING FOR {} MINUTES-\n", {
            sleep_duration_minutes
        });

        sleep(Duration::from_secs(sleep_duration_minutes * 60)).await;
    }
}
