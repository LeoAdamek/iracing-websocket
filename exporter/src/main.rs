#![cfg(windows)]

extern crate actix;
extern crate iracing;
#[macro_use] extern crate log;
extern crate env_logger;
extern crate awc;
extern crate actix_codec;
extern crate futures;

mod reader;
mod writer;

use std::time::Duration;
use std::thread::sleep;

use awc::Client;
use actix::prelude::*;
use futures::SinkExt;
use futures::StreamExt;
use actix::io::SinkWrite;

pub fn main() {

    env_logger::builder().filter_level(log::LevelFilter::Debug).init();


    let system = System::new("Exporter");

    Arbiter::spawn(async {
        info!("Connecting to service");
        let (rsp, framed) =
            Client::new().ws("ws://127.0.0.1:8080/source").connect().await
                .map_err(|e| { error!("Unable to connect to socket"); }).unwrap();

        let (sink, stream) = framed.split();

        let wr = writer::WebsocketWriter::create(|ctx| {
            writer::WebsocketWriter::add_stream(stream, ctx);
            writer::WebsocketWriter::new(SinkWrite::new(sink, ctx))
        });

        info!("Connecting to iRacing");
        let src = reader::IRacingReader::new(get_iracing_connection()).start();
        let reader = reader::TelemetryReader::new( src.clone().recipient(), wr.clone().recipient() );
        let session_reader = reader::SessionReader::new(src.recipient(), wr.recipient()).start();
        let reader_addr = reader.start();

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
               error!("Unable to get iRacing Connection. Is iRacing Running?");
               sleep(Duration::from_secs(2));
           }
       }
   } 

   conn
}