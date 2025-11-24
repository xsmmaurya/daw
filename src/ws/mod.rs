// src/ws/mod.rs

use std::time::{Duration, Instant};

use actix::{
    Actor,
    ActorContext,
    AsyncContext,
    StreamHandler,
    Addr,
    Handler,
    Message as ActixMessage,
};
use actix_web::{web, HttpRequest, HttpResponse, Error};
use actix_web_actors::ws;
use once_cell::sync::OnceCell;
use tokio::sync::broadcast;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::utils::jwt_util::decode_jwt_token;
use crate::utils::ws_auth::validate_and_extract;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsNotification {
    pub user_id: Uuid,
    pub kind: String,
    pub payload: Value,
}

static NOTIFY_TX: OnceCell<broadcast::Sender<WsNotification>> = OnceCell::new();

/// Initialize global WS hub (call once at startup)
pub fn init_ws_hub() {
    let (tx, _rx) = broadcast::channel::<WsNotification>(1024);
    let _ = NOTIFY_TX.set(tx);
}

/// Best-effort: broadcast a notification to a given user_id
pub async fn notify_user(user_id: Uuid, kind: &str, payload: Value) -> anyhow::Result<()> {
    if let Some(tx) = NOTIFY_TX.get() {
        let msg = WsNotification {
            user_id,
            kind: kind.to_string(),
            payload,
        };
        let _ = tx.send(msg);
    }
    Ok(())
}

/// Internal message type to push text into a WsSession
#[derive(Debug)]
struct WsText(String);

impl ActixMessage for WsText {
    type Result = ();
}

struct WsSession {
    user_id: Uuid,
    rx: broadcast::Receiver<WsNotification>,
    hb: Instant,
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb = Instant::now();
        Self::spawn_heartbeat(ctx);
        Self::spawn_broadcast_reader(self.user_id, ctx, self.rx.resubscribe());
    }
}

impl Handler<WsText> for WsSession {
    type Result = ();

    fn handle(&mut self, msg: WsText, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

impl WsSession {
    fn spawn_heartbeat(ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(Duration::from_secs(30), |act, ctx| {
            if Instant::now().duration_since(act.hb) > Duration::from_secs(60) {
                ctx.stop();
                return;
            }
            ctx.ping(b"ping");
        });
    }

    fn spawn_broadcast_reader(
        user_id: Uuid,
        ctx: &mut ws::WebsocketContext<Self>,
        mut rx: broadcast::Receiver<WsNotification>,
    ) {
        // Address of this WS actor
        let addr: Addr<WsSession> = ctx.address();

        // Background task: listen on broadcast channel, forward to this actor
        actix::spawn(async move {
            while let Ok(msg) = rx.recv().await {
                if msg.user_id == user_id {
                    if let Ok(text) = serde_json::to_string(&msg) {
                        let _ = addr.do_send(WsText(text));
                    }
                }
            }
            // when channel closes, task exits
        });
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(_txt)) => {
                // handle incoming client messages if needed
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => {}
        }
    }
}

/// GET /api/v1/ws?token=JWT
pub async fn ws_handler(
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, Error> {
    let user_id: Uuid = validate_and_extract(&req)?;

    let tx = NOTIFY_TX
        .get()
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("WS hub not initialized"))?;
    let rx = tx.subscribe();

    let session = WsSession {
        user_id,
        rx,
        hb: Instant::now(),
    };

    ws::start(session, &req, stream)
}