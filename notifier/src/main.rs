use log::error;
use manager::{DatabaseConfig, Manager};
use rand::random;
use std::time::Duration;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let db = Manager::try_new(&DatabaseConfig::default(), true)
        .await
        .expect("Success");

    loop {
        if let Err(e) = db.new_widget().await {
            error!("{:?}", e);
        };
        let sleep_time: u64 = random::<u64>() % 15;
        tokio::time::sleep(Duration::from_secs(sleep_time)).await;
    }
}
