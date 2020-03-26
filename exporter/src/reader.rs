use std::time::Duration;
use serde::{Serialize, Deserialize};
use actix::prelude::*;
use iracing::session::SessionDetails;
use iracing::telemetry::Value;
use iracing::states::Flags;
use iracing;

const TELEMETRY_FREQUENCY: Duration = Duration::from_millis(100);


#[derive(Message,Debug,Default,Serialize,Deserialize,Clone)]
#[rtype(result = "()")]
pub struct TelemetryMessage {
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

#[derive(Message,Debug,Serialize,Deserialize,Clone)]
#[rtype(result = "()")]
pub struct SessionMessage(SessionDetails);

#[derive(Message,Debug,Serialize,Deserialize,Clone)]
#[rtype(result = "TelemetryMessage")]
pub struct TelemetryRequest;

#[derive(Message,Debug,Serialize,Deserialize,Clone)]
#[rtype(result = "SessionMessage")]
pub struct SessionRequest;

pub struct TelemetryReader {
    writer: Recipient<TelemetryMessage>,
    src: Recipient<TelemetryRequest>
}

impl Actor for TelemetryReader {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        unsafe { self.read_telemetry(ctx) };
    }
}

impl TelemetryReader {
    pub fn new(src: Recipient<TelemetryRequest>, writer_addr: Recipient<TelemetryMessage>) -> Self {
        TelemetryReader { src: src, writer: writer_addr }
    }

    /// Telemetry read loop
    /// 
    /// Reads telemetry data and sends it to the Writer
    pub unsafe fn read_telemetry(&mut self, ctx: &mut <Self as Actor>::Context) {
            ctx.run_interval(TELEMETRY_FREQUENCY, |act, ctx| {
            act.src.send(TelemetryRequest).into_actor(act).then(|res, act, _ctx| {
                match res {
                    Ok(t) => {
                        info!("Got Telem");
                        act.writer.do_send(t)
                    },

                    Err(e) => {
                        error!("Unable to get telemetry: {}", e);

                        Ok(())
                    }
                };

                fut::ready(())
            }).wait(ctx);
        });
    }
}

pub struct SessionReader {
    src: Recipient<SessionRequest>,
    writer: Recipient<SessionMessage>
}

impl SessionReader {
    pub fn new(src_addr: Recipient<SessionRequest>, writer_addr: Recipient<SessionMessage>) -> Self {
        Self { src: src_addr, writer: writer_addr }
    }
}

impl Actor for SessionReader {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(Duration::from_secs(5), |act, ctx| { 
            act.src.send(SessionRequest).into_actor(act).then(|res, act, _ctx| {
                match res {
                    Ok(s) => {
                        info!("Got Session");
                        act.writer.do_send(s)
                    },

                    Err(e) => {
                        error!("Unable to get session: {}", e);
                        Ok(())
                    }
                };

                fut::ready(())
            }).wait(ctx);
        });
    }
}

pub struct IRacingReader {
    conn: iracing::Connection
}

impl IRacingReader {
    pub fn new(conn: iracing::Connection) -> Self {
        Self { conn: conn }
    }
}

impl Actor for IRacingReader {
    type Context = Context<Self>;
}

impl Handler<TelemetryRequest> for IRacingReader {
    type Result = MessageResult<TelemetryRequest>;

    fn handle(&mut self, _: TelemetryRequest, _ctx: &mut Self::Context) -> Self::Result {
        match self.conn.telemetry() {
            Err(e) => {
                panic!("Error getting telemetry: {}", e);
            }

            Ok(telem) => {
                let car_positions = telem.get("CarIdxPosition").unwrap();
                let car_class_positions = telem.get("CarIdxClassPosition").unwrap();
                let car_laps = telem.get("CarIdxLap").unwrap();
                let car_laps_perc = telem.get("CarIdxLapDistPct").unwrap();
                let car_pits = telem.get("CarIdxOnPitRoad").unwrap();
                let car_steers = telem.get("CarIdxSteer").unwrap();
                let car_gears = telem.get("CarIdxGear").unwrap();
                let car_rpms = telem.get("CarIdxRPM").unwrap();
                let air_temperature: f32 = telem.get("AirTemp").unwrap_or(Value::FLOAT(-273f32)).into();
                let track_temp: f32 = telem.get("TrackTemp").unwrap_or(Value::FLOAT(-273f32)).into();
                let state: i32 = telem.get("SessionState").unwrap_or(Value::INT(0i32)).into();
                let raw_flags: u32 = telem.get("SessionFlags").unwrap_or(Value::BITS(0u32)).into();

                let flags = Flags::from_bits(raw_flags);

                debug!("Flags = {:?} (Raw = {:b})", flags, raw_flags);

                let data = TelemetryMessage {
                    air_temperature: air_temperature,
                    flags: raw_flags,
                    track_temperature: track_temp,
                    state: state,
                    car_positions: match car_positions { Value::IntVec(ints) => ints, _ => vec![0i32; 64] },
                    car_class_positions: match car_class_positions { Value::IntVec(ints) => ints, _ => vec![0i32; 64] },
                    car_pits: vec![false; 64],
                    car_gears: match car_gears { Value::IntVec(ints) => ints, _ => vec![0i32; 64] },
                    car_rpms: match car_rpms { Value::FloatVec(floats) => floats, _ => vec![0f32; 64] },
                    car_laps: match car_laps { Value::IntVec(ints) => ints, _ => vec![0i32; 64] },
                    car_laps_perc: match car_laps_perc { Value::FloatVec(floats) => floats, _ => vec![0f32; 64] },
                    car_steers: match car_steers { Value::FloatVec(floats) => floats, _ => vec![0f32; 64] }
                };

                MessageResult(data)
            }
        }
    }
}

impl Handler<SessionRequest> for IRacingReader {
    type Result = MessageResult<SessionRequest>;

    fn handle(&mut self, _: SessionRequest, _: &mut Self::Context) -> Self::Result {
        MessageResult( SessionMessage( self.conn.session_info().unwrap() ) )
    }
}