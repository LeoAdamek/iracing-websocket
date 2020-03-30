//! `TelemetryServer` is an actor that maintains the client connections and manages data streams.

use std::collections::BTreeMap;
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
    connections: BTreeMap<usize, Recipient<Message>>,
    pub cnt: usize,
    pub session_data: Option<SessionDetails>
}

#[derive(Message,Debug,Default,Serialize,Deserialize,Clone)]
#[rtype(result = "()")]
pub struct TelemetryData {
    pub air_temperature: f32,
    pub state: i32,
    pub flags: u32,
    pub session_number: i32,
    pub time_remaining: f32,
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

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: usize
}



impl Default for TelemetryServer {
    fn default() -> Self {
        Self {
            session_data: None,
            cnt: 0,
            connections: BTreeMap::new()
        }
    }
}


impl TelemetryServer {
    fn broadcast(&self, msg: &Message) {
        for con in self.connections.values() {
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

        self.cnt += 1;
        let id = self.cnt;

        self.connections.insert(id, msg.addr);

        info!("There are now {} connected users", self.connections.len());

        id
    }
}

impl Handler<Disconnect> for TelemetryServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _ctx: &mut Context<Self>) -> Self::Result {
        info!("User disconnected");

        self.connections.remove(&msg.id);

        info!("There are now {} connected users", self.connections.len());
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