extern crate actix;
extern crate actix_web_actors;
#[macro_use] extern crate log;
extern crate env_logger;

use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use actix::{Actor, Addr, Context};

use std::io;

pub mod session;
pub mod server;
mod source;
mod client;

#[derive(Clone)]
pub struct AppState {
    stream_password: String,
    server: Addr<server::TelemetryServer>,
}

#[actix_rt::main]
pub async fn main() -> io::Result<()> {
    env_logger::builder().filter_level(log::LevelFilter::Info).init();

    info!("Initalizing Telemetry Server");

    let state = AppState::new();

    HttpServer::new(move || {
        App::new()
            .data(state.clone())
            .service(web::resource("/telemetry").to(connect_client))
            .service(web::resource("/source").to(connect_source))
    }).bind("0.0.0.0:8080")?.run().await
}


pub async fn connect_client(req: HttpRequest, stream: web::Payload, state: web::Data<AppState>) -> Result<HttpResponse, Error> {
    ws::start(client::WsTelemetryClient::new(state.get_ref().server.clone()), &req, stream)
}

pub async fn connect_source(req: HttpRequest, stream: web::Payload, state: web::Data<AppState>) -> Result<HttpResponse, Error> {
    ws::start(source::Source::new(state.get_ref().server.clone()), &req, stream)
}

impl AppState {
    pub fn new() -> Self {
        Self {
            server: server::TelemetryServer::default().start(),
            stream_password: String::default()
        }
    }
}
