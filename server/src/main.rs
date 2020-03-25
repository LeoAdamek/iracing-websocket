extern crate actix;
extern crate actix_web_actors;
#[macro_use] extern crate log;
extern crate env_logger;

use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws::start as start_ws;
use actix::Addr;

use std::io;
use std::time::Instant;

pub mod session;

mod server;
mod source;
mod client;


#[derive(Clone)]
struct AppState {
    server: Addr<server::TelemetryServer>,
    source: Addr<source::Source>
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
    }).bind("0.0.0.0:8080")?.run().await
}


pub async fn connect_client(req: HttpRequest, stream: web::Payload, state: web::Data<AppState>) -> Result<HttpResponse, Error> {
    start_ws(client::WsTelemetryClient::new(state.get_ref().server.clone()), &req, stream)
}

impl AppState {
    pub fn new() -> Self {
        let srv = server::TelemetryServer::default().start();
        let src = source::Source::new(String::new(), srv).start();

        Self {
            server: srv,
            source: src
        }
    }
}
