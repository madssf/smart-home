use std::{time::Duration, thread, env};
use chrono::{TimeZone, Utc};
use chrono_tz::Tz;

mod prices;
mod scheduling;
mod plugs;
mod shelly_client;
mod firestore;


#[tokio::main]
async fn main() {

    loop {
        println!("\n-STARTING NEW RUN-\n");
        let price = prices::get_current_price();
        println!("Current price: {} {}, level: {}", &price.amount, &price.currency, &price.level);
    
        let utc = Utc::now().naive_utc();
        let tz: Tz = env::var("TIME_ZONE").expect("Missing TIME_ZONE env var").parse().expect("Failed to parse timezone");
        let time = tz.from_utc_datetime(&utc).naive_local();

        println!("Current time: {}", &time);

        println!("\n");

        let action = scheduling::get_action(&price.level, &time);

        println!("Got action: {}", action.to_string());
        
        println!("\n");


        let plugs = plugs::get_plugs_from_env();

        for plug in plugs {
            println!("Proccesing plug: {}", &plug.name);
            let power_usage = shelly_client::get_status(&plug).unwrap();
            println!("Current power usage: {} W", power_usage);
            println!("Equals hourly price of: {:.3} {}", price.amount / 1000.0 * power_usage, price.currency);
            shelly_client::execute_action(&plug, &action).unwrap();
            println!("\n");
        }

        let sleep_duration: u64= 1*60;

        println!("\n-RUN FINISHED, SLEEPING FOR {} MINUTES-\n", {&sleep_duration/60});


        thread::sleep(Duration::from_secs(sleep_duration))
    };

}
