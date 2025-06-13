use actix_web::{App, HttpRequest, HttpResponse, HttpServer, ResponseError, get, web};
use actix_ws::{Message, handle};
use futures_util::StreamExt;
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use std::time::Duration;
use thiserror::Error;
use tokio::sync::broadcast;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("MQTT error: {0}")]
    Mqtt(#[from] rumqttc::ClientError),
    #[error("Web error: {0}")]
    Web(#[from] actix_web::Error),
    #[error("UTF-8 decode error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::InternalServerError().body(self.to_string())
    }
}

#[get("/ws")]
async fn ws_handler(
    req: HttpRequest,
    body: web::Payload,
    tx: web::Data<broadcast::Sender<String>>,
) -> Result<HttpResponse, AppError> {
    let (res, session, mut msg_stream) = handle(&req, body)?;
    let mut rx = tx.subscribe();

    let mut session_tx = session.clone();
    actix_web::rt::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if let Err(e) = session_tx.text(msg).await {
                eprintln!("❌ WebSocket send error: {:?}", e);
                break;
            }
        }
    });

    actix_web::rt::spawn(async move {
        while let Some(Ok(msg)) = msg_stream.next().await {
            if let Message::Text(txt) = msg {
                println!("Client said: {}", txt);
            }
        }
    });

    Ok(res)
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let (mqtt_client, mut event_loop) = connect_mqtt();
    mqtt_client
        .subscribe("probe/data", QoS::AtLeastOnce)
        .await?;

    let (tx, _) = broadcast::channel::<String>(100);
    let tx_clone = tx.clone();

    tokio::spawn(async move {
        while let Ok(notification) = event_loop.poll().await {
            if let Event::Incoming(Packet::Publish(p)) = notification {
                if let Ok(payload) = String::from_utf8(p.payload.to_vec()) {
                    let _ = tx_clone.send(payload);
                }
            }
        }
    });

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(tx.clone()))
            .service(ws_handler)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    Ok(())
}

fn connect_mqtt() -> (AsyncClient, rumqttc::EventLoop) {
    let mut options = MqttOptions::new("marine_probe_client", "localhost", 1883);
    options.set_keep_alive(Duration::from_secs(5));
    AsyncClient::new(options, 10)
}
