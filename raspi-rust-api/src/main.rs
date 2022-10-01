use rust_home::clients::get_clients;
use rust_home::looper;

#[tokio::main]
async fn main() {
    let sleep_duration_in_minutes: u64 = 2;

    let (shelly_client, firestore_client) = get_clients();

    looper::start(&firestore_client, &shelly_client, sleep_duration_in_minutes).await
}
