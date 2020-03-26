//! `TelemetryServer` is an actor that maintains the client connections and manages data streams.

use actix::prelude::*;
use crate::session::SessionDetails;
use serde::{Deserialize,Serialize};


// Messages are encoded as (and passed as) strings.
#[derive(Message,Clone)]
#[rtype(result = "()")]
pub enum Message {
    Telemetry(TelemetryData),
    Session(SessionDetails)
}


#[derive(Debug,Clone)]
pub struct TelemetryServer {
    connections: Vec<Recipient<Message>>,
    session_data: Option<SessionDetails>
}

#[derive(Message,Debug,Default,Serialize,Deserialize,Clone)]
#[rtype(result = "()")]
pub struct TelemetryData {
    pub air_temperature: f32,
    pub state: i32,
    pub flags: u32,
    pub track_temperature: f32,
    pub car_class_positions: Vec<i32>,
    pub car_positions: Vec<i32>,
    pub car_gears: Vec<i32>,
    pub car_rpms: Vec<f32>,
    pub car_steers: Vec<f32>,
    pub car_laps: Vec<i32>,
    pub car_laps_perc: Vec<f32>,
    pub car_pits: Vec<bool>
}

#[derive(Message, Debug)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<Message>
}


impl Default for TelemetryServer {
    fn default() -> Self {
        Self {
            session_data: None,
            connections: Vec::new()
        }
    }
}


impl TelemetryServer {
    fn broadcast(&self, msg: &Message) {
        for con in self.connections.iter() {
            let _ = con.do_send(msg.to_owned());
        }
    }
}

impl Actor for TelemetryServer {
    type Context = Context<Self>;
}

impl Handler<Connect>  for TelemetryServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _ctx: &mut Context<Self>) -> Self::Result {
        info!("User Connected: {:?}", msg);

        let id: usize = 0;

        self.connections.insert(id, msg.addr);

        id
    }
}

impl Handler<TelemetryData> for TelemetryServer {
    type Result = ();

    // Handle receipt of a new telemetry by broadcasting to all clients
    fn handle(&mut self, msg: TelemetryData, _ctx: &mut Context<Self>) {
        self.broadcast(&Message::Telemetry(msg));
    }
}

impl Handler<SessionDetails> for TelemetryServer {
    type Result = ();
    
    // Handle receipt of a new session by broadcasting to all clients
    fn handle(&mut self, msg: SessionDetails, _ctx: &mut Context<Self>) {
        self.session_data = Some(msg.clone());
        self.broadcast(&Message::Session(msg));
    }
}