use futures::{stream, StreamExt};
use futures::{FutureExt, TryStreamExt};
use manager::{DatabaseConfig, Manager};
use serde::Deserialize;
use tokio_postgres::NoTls;

#[derive(Clone, Debug, Deserialize)]
struct Payload {
    widget_id: i64,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let db_config = DatabaseConfig::default();
    let manager = Manager::try_new(&db_config, false).await.expect("success");
    let connection_string = format!(
        "host={} user={} dbname={} port={} password={}",
        db_config.host, db_config.username, db_config.db_name, db_config.port, db_config.password
    );
    let (client, mut connection) = tokio_postgres::connect(&connection_string, NoTls)
        .await
        .unwrap();

    // Make transmitter and receiver.
    let (tx, mut rx) = futures_channel::mpsc::unbounded();
    let stream =
        stream::poll_fn(move |cx| connection.poll_message(cx)).map_err(|e| panic!("{}", e));
    let connection = stream.forward(tx).map(|r| r.unwrap());
    tokio::spawn(connection);

    if let Err(e) = client.execute("LISTEN widget_notification;", &[]).await {
        eprintln!("Error {}", e);
    }

    // Wait for notifications in seperate thread.
    println!("Testing notifications");
    while let Some(notification) = rx.next().await {
        {
            match notification {
                tokio_postgres::AsyncMessage::Notification(notification) => {
                    println!("Notification {:?}", notification);
                    let payload: Payload = serde_json::from_str(notification.payload()).unwrap();
                    manager
                        .insert_notified(payload.widget_id)
                        .await
                        .expect("ok");
                    futures_util::future::ready(Some(notification))
                }
                _ => futures_util::future::ready(None),
            }
            .await;
        }
    }
}
