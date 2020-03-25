//! TelemetryClient is an actor which represents a telemetry receipient connection
use crate::server;

use std::time::{Instant, Duration};

use actix::prelude::*;
use actix_web_actors::ws;
use serde_json::to_string as json;

pub struct WsTelemetryClient {
    hb: Instant,
    server: Addr<server::TelemetryServer>
}

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(1);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

impl Actor for WsTelemetryClient {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);

        let addr = ctx.address();

        self.server.send(server::Connect {
            addr: addr.recipient()
        }).into_actor(self).then(|res, _act, ctx| { 
            match res {
                Ok(_) => (),
                _ => ctx.stop(),
            }

            fut::ready(())
        }).wait(ctx);
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        Running::Stop
    }
}

impl Handler<server::Message> for WsTelemetryClient {
    type Result = ();

    fn handle(&mut self, msg: server::Message, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            server::Message::Telemetry(telem) => {
                let data = ('T', telem);

                ctx.text(json(&data).unwrap());
            },

            server::Message::Session(session) => {
                let data = ('S', session);

                ctx.text(json(&data).unwrap());
            }
        };
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsTelemetryClient {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Err(_) => { ctx.stop(); return; }
            Ok(msg) => msg
        };

        match msg {
            ws::Message::Ping(ping) => {
                self.hb = Instant::now();
                ctx.pong(&ping);
            }

            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }

            ws::Message::Close(_) => {
                ctx.stop();
            }

            _ => (),
        }
    }
}

impl WsTelemetryClient {
    pub fn new(server_addr: Addr<server::TelemetryServer>) -> Self {
        Self {
            hb: Instant::now(),
            server: server_addr
        }
    }

    fn hb(&mut self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                ctx.stop();
                return;
            }

            ctx.ping(b"PING");
        });
    }
}