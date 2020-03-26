//! Source is a singleton actor which receives the session & telemetry data
//! from the iRacing exporter and passes it to the TelemetryServer

use actix::prelude::*;
use actix_web_actors::ws;

use crate::server;
use crate::session;
use serde_json::from_str;

#[derive(Clone,Debug)]
pub struct Source {
    server: Addr<server::TelemetryServer>
}

impl Actor for Source {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Source {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let payload = match msg {
            Ok(m) => m,
            Err(e) => {
                error!("Source Handler Error: {}", e);
                return;
            }
        };

        match payload {
            ws::Message::Text(txt) => {
                let raw = txt.as_str();

                match txt.chars().next().unwrap() {
                    'T' => {
                        let telem_pkt = &raw[1..];

                        match from_str::<server::TelemetryData>(telem_pkt) {
                            Ok(t) =>  {
                               self.server.do_send(t) 
                            }
                            Err(e) => {
                                error!("Invalid telemetry: {}", e);
                            } 
                        };

                    } 

                    'S' => {
                        let session_pkt = &raw[1..];

                        match from_str::<session::SessionDetails>(session_pkt) {
                            Ok(s) => {
                                self.server.do_send(s)
                            },

                            Err(e) => {
                                error!("Invalid Session: {}", e);
                            }
                        };
                    }

                    a @ _ => {
                        warn!("Unknown data type: '{:?}'", a);
                    }
                }
            }

            ws::Message::Close(_) => {
                ctx.stop();
            }

            _ => ()
        }
    }
}

impl Source {
    pub fn new(server_addr: Addr<server::TelemetryServer>) -> Self {
        Self {
            server: server_addr
        }
    }
}