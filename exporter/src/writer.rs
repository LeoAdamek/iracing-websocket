use actix::prelude::*;
use actix_codec::Framed;
use actix::io::SinkWrite;
use crate::reader::{TelemetryMessage, SessionMessage};
use awc::{error::WsProtocolError, ws::{Codec,Frame,Message}, BoxedSocket, Client};
use futures::stream::{SplitSink, StreamExt};
use serde_json::to_string as json;

pub struct WebsocketWriter(SinkWrite<Message, SplitSink<Framed<BoxedSocket, Codec>, Message>>);

impl WebsocketWriter {
    pub fn new(s: SinkWrite<Message, SplitSink<Framed<BoxedSocket, Codec>, Message>>) -> Self {
        Self(s)
    }
}

impl Actor for WebsocketWriter {
    type Context = Context<Self>;
}

impl StreamHandler<Result<Frame, WsProtocolError>> for WebsocketWriter {
    fn handle(&mut self, _: Result<Frame, WsProtocolError>, _: &mut Context<Self>) {

    }
}

impl Handler<TelemetryMessage> for WebsocketWriter {
    type Result = ();

    fn handle(&mut self, msg: TelemetryMessage, _ctx: &mut Self::Context) {
        trace!("Sending Telemetry: {:?}", msg);

        let data: String = json(&msg).unwrap();
        let mut content = "T".to_owned();

        content.push_str(&data);

        self.0.write(Message::Text(content));
    }
}

impl Handler<SessionMessage> for WebsocketWriter {
    type Result = ();

    fn handle(&mut self, msg: SessionMessage, _ctx: &mut Self::Context) {
        trace!("Sending Session");

        let data: String = json(&msg).unwrap();

        let mut content = "S".to_owned();

        content.push_str(&data);

        self.0.write(Message::Text(content));
    }
}

impl actix::io::WriteHandler<WsProtocolError> for WebsocketWriter {}