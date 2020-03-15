use actix_web::HttpResponse;
use actix_web::HttpRequest;
use std::collections::HashMap;
use iracing::telemetry;
use actix::prelude::*;
use actix_web::{web};
use actix_web_actors::ws;
use std::time::Duration;
use serde::Deserialize;
use serde_json::to_string as json;
use std::cmp::max;
use rmp_serde::to_vec as msgpack;

pub type TelemetryData = HashMap<String, telemetry::Value>;
pub struct TelemMessage(pub TelemetryData);

const SAMPLE_TIMEOUT: Duration = Duration::from_millis(100);

pub enum TelemetryEncoder {
  Json,
  MsgPack
}

pub struct TelemetryStream {
  update_interval: Duration,
  encoder: TelemetryEncoder,
  src: iracing::telemetry::Blocking
}

#[derive(Debug, Deserialize)]
pub struct TelemetryParameters {
  i: Option<u64>,
  encoding: Option<String>,
}

impl TelemetryStream {
  fn new(interval: Duration, encoding: TelemetryEncoder) -> Self {

    let conn = iracing::Connection::new().expect("Unable to connect to iRacing. Is iRacing running?");
    let src = conn.blocking().unwrap();

    Self {
      update_interval: interval,
      encoder: encoding,
      src: src
    }
  }

  fn stream_telemetry(&self, ctx: &mut <Self as Actor>::Context) {
    ctx.run_interval(self.update_interval, |act, ctx| {
      let data: TelemetryData = act.src.sample(SAMPLE_TIMEOUT).unwrap().all().iter().map(|v| {
            (v.name.clone(), v.value.clone())
        }).collect();

      match act.encoder {
        TelemetryEncoder::Json => ctx.text(json(&data).unwrap()),
        TelemetryEncoder::MsgPack => ctx.binary(msgpack(&data).unwrap())
      };
    });
  }
}

impl actix::Actor for TelemetryStream {
  type Context = ws::WebsocketContext<Self>;

  fn started(&mut self, ctx: &mut <Self as Actor>::Context) {
    self.stream_telemetry(ctx);
  }

  fn stopped(&mut self, _ctx: &mut <Self as Actor>::Context) {
    match self.src.close() {
      Ok(()) => info!("Closed iRacing Telem Socket"),
      Err(e) => error!("Unable to close iRacing Telem Handle {}", e)
    };
  }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for TelemetryStream {
  fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
    match msg {
      Ok(ws::Message::Ping(m)) => ctx.pong(&m),
      Ok(ws::Message::Close(_)) => ctx.stop(),
      Ok(ws::Message::Text(t)) => {
        // Client may wish to renegotiate update rate
        match u64::from_str_radix(&t, 10) {
          Ok(v) => self.update_interval = Duration::from_millis(v),
          _ => {}
        };
      }
      _ => ctx.stop(),
    }
  }
}

pub async fn connect_ws(req: HttpRequest, stream: web::Payload, data: web::Data<crate::AppState>, query: web::Query<TelemetryParameters>) -> actix_web::Result<HttpResponse> {

  let enc = match &query.encoding {
    None => TelemetryEncoder::MsgPack,
    Some(enc) => {
      match enc.as_str() {
        "text" => TelemetryEncoder::Json,
        _ => TelemetryEncoder::MsgPack
      }
    }
  };

  let duration = match &query.i {
    None => Duration::from_millis(100),
    Some(i) => Duration::from_millis(max(50, *i))
  };

  info!("Client Params: {:?}", query);

  let tap = TelemetryStream::new(duration, enc);


  let rsp = ws::start(tap, &req, stream);

  rsp
}