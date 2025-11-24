// src/routes/ws.rs
use actix_web::web;
use crate::ws::ws_handler;

pub fn configure_ws_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/ws", web::get().to(ws_handler));
}
