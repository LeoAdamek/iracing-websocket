//! Source is a singleton actor which receives the session & telemetry data
//! from the iRacing exporter and passes it to the TelemetryServer

use actix::prelude::*;
use actix_web_actors::ws;

use crate::server;

#[derive(Clone,Debug)]
pub struct Source {
    password: String,
    server: Addr<server::TelemetryServer>
}

impl Actor for Source {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Source {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
    }
}

impl Source {
    pub fn new(password: String, server_addr: Addr<server::TelemetryServer>) -> Self {
        Self {
            password: password,
            server: server_addr
        }
    }
}