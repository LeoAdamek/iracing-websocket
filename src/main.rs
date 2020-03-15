extern crate actix;
extern crate actix_web;
extern crate actix_cors;
extern crate env_logger;
extern crate iracing;
extern crate serde_json;
#[macro_use]
extern crate log;

use std::io;

use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpResponse, HttpServer, Responder};

mod telem;

pub struct AppState {
  pub session: iracing::session::SessionDetails,
}

async fn get_session(data: web::Data<AppState>) -> impl Responder {
  HttpResponse::Ok().json(&data.session)
}

#[actix_rt::main]
async fn main() -> io::Result<()> {
  env_logger::builder()
    .filter_level(log::LevelFilter::Info)
    .init();

  info!("Starting Application");
  info!("Connecting to iRacing");

  let mut conn =
    iracing::Connection::new().expect("Unable to connect to iRacing. Is iRacing running?");
  let session = conn
    .session_info()
    .expect("Unable to get iRacing Session Details");

  let data = web::Data::new(AppState { session: session });


  HttpServer::new(move || {
    App::new()
      .wrap(middleware::Logger::default())
      .wrap(middleware::Compress::default())
      .wrap(Cors::new()
        .allowed_origin("http://192.168.124.130:8080")
        .allowed_methods(vec!["GET"])
        .allowed_headers(vec!["Content-Type","Accept","Accept-Encoding","DNT","Connection","Upgrade"])
        .max_age(3600).finish())
      .app_data(data.clone())
      .service(web::resource("/session").route(web::get().to(get_session)))
      .route("/telemetry", web::get().to(telem::connect_ws))
  })
  .bind("0.0.0.0:8088")?
  .run()
  .await
}
