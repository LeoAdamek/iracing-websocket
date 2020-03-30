#![cfg(windows)]

extern crate actix;
extern crate iracing;
#[macro_use] extern crate log;
extern crate env_logger;
extern crate awc;
extern crate actix_codec;
extern crate futures;
extern crate config;

mod reader;
mod writer;

use std::time::Duration;
use std::thread::sleep;

use awc::Client;
use actix::prelude::*;
use futures::StreamExt;
use actix::io::SinkWrite;
use serde::{Serialize,Deserialize};

use std::process::exit;

#[derive(Deserialize,Serialize,Clone,Debug)]
pub struct Settings {
    pub telemetry_update_interval: u64,
    pub session_update_interval: u64,
    pub telemetry_service_url: String
}

pub fn main() {

    env_logger::init();

    let mut cfg: config::Config = config::Config::default();
    let _ = cfg.set_default("telemetry_service_url", "ws://127.0.0.1:8088/source");
    let _ = cfg.set_default("session_update_interval", 5000);
    let _ = cfg.set_default("telemetry_update_interval", 250);

    cfg.merge(config::File::with_name("exporter")).unwrap();

    let settings: Settings = match cfg.try_into() {
        Ok(s) => s,
        Err(e) => {
            error!("Invalid Configuration: {:?}", e);
            return;
        }
    };

    let system = System::new("Exporter");
    let url = settings.clone().telemetry_service_url;

    Arbiter::spawn(async {
        info!("Connecting to service @ {}", url);


        let (_, framed) =
               Client::build().connector(
                   awc::Connector::new().timeout(Duration::from_secs(10)).finish())
                   .timeout(Duration::from_secs(10)).finish()
                   .ws(url).connect().await
                .map_err(|e| { 
                    error!("Unable to connect to socket: {}", e); 
                    exit(10);
                }).unwrap();

        let (sink, stream) = framed.split();

        let wr = writer::WebsocketWriter::create(|ctx| {
            writer::WebsocketWriter::add_stream(stream, ctx);
            writer::WebsocketWriter::new(SinkWrite::new(sink, ctx))
        });

        info!("Connecting to iRacing");
        let src = reader::IRacingReader::new(get_iracing_connection()).start();
        let reader = reader::TelemetryReader::new(Duration::from_millis(250), src.clone().recipient(), wr.clone().recipient() );
        reader::SessionReader::new(src.recipient(), wr.recipient()).start();
        reader.start();
    });

    info!("Starting System");
    match system.run() {
        Ok(r) => info!("{:?}", r),
        Err(e) => error!("{}", e)
    };
}


fn get_iracing_connection() -> iracing::Connection {
   let conn: iracing::Connection;

   loop {
       match iracing::Connection::new() {
           Ok(c) => {
                conn = c;
                break;
           }
           Err(e) => {
               error!("Unable to get iRacing Connection. Is iRacing Running? {}", e);
               sleep(Duration::from_secs(2));
           }
       }
   } 

   conn
}