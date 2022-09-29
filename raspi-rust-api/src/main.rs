use std::{env, time::Duration};

use chrono::{TimeZone, Utc};
use chrono_tz::Tz;
use tokio::time::sleep;

use rust_home::prices::PriceInfo;
use rust_home::{plugs, prices, scheduling, shelly_client};

#[tokio::main]
async fn main() {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to create client");

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

        let action = scheduling::get_action(&price.level, &time);

        println!("Got action: {}", action.to_string());

        let plugs = plugs::get_plugs_from_env();

        for plug in plugs {
            println!("Processing plug: {}", &plug.name);
            if let Ok(power_usage) = shelly_client::get_status(&client, &plug).await {
                println!("Current power usage: {} W", power_usage);
                println!(
                    "Equals hourly price of: {:.3} {}",
                    price.amount / 1000.0 * power_usage,
                    price.currency
                );
            };

            match shelly_client::execute_action(&client, &plug, &action).await {
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

        let sleep_duration: u64 = 60;

        println!("\n-RUN FINISHED, SLEEPING FOR {} MINUTES-\n", {
            &sleep_duration / 60
        });

        sleep(Duration::from_secs(sleep_duration)).await;
    }
}
