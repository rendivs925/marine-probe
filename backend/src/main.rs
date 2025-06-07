use actix::prelude::*;
use actix_web::{App, HttpRequest, HttpServer, Responder, get, web};
use actix_web_actors::ws;
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use std::time::Duration;
use tokio::sync::broadcast;

struct WsSession {
    rx: broadcast::Receiver<String>,
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address();
        let mut rx = self.rx.resubscribe();

        actix::spawn(async move {
            while let Ok(msg) = rx.recv().await {
                let _ = addr.do_send(WsMessage(msg));
            }
        });
    }
}

#[derive(Message)]
#[rtype(result = "()")]
struct WsMessage(String);

impl Handler<WsMessage> for WsSession {
    type Result = ();

    fn handle(&mut self, msg: WsMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
    fn handle(&mut self, _: Result<ws::Message, ws::ProtocolError>, _: &mut Self::Context) {}
}

#[get("/ws")]
async fn ws_handler(
    req: HttpRequest,
    stream: web::Payload,
    tx: web::Data<broadcast::Sender<String>>,
) -> impl Responder {
    let session = WsSession { rx: tx.subscribe() };
    ws::start(session, &req, stream)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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
