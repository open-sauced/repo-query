mod constants;
mod conversation;
mod db;
mod embeddings;
mod github;
mod prelude;
mod routes;
mod utils;
use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use constants::{HOME_ROUTE_REDIRECT_URL, WEBSERVER_PORT_DEFAULT};
use env_logger::Env;
use tracing_actix_web::TracingLogger;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let model: Arc<embeddings::Fastembed> = Arc::new(embeddings::Fastembed::try_new().unwrap());
    let db: Arc<db::QdrantDB> = Arc::new(db::QdrantDB::initialize().unwrap());

    let mut port = std::env::var("WEBSERVER_PORT").unwrap_or(WEBSERVER_PORT_DEFAULT.into());
    if port.is_empty() {
        port = WEBSERVER_PORT_DEFAULT.to_string();
    }
    let port = port.parse::<u16>().expect("Invalid WEBSERVER_PORT");

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .wrap(TracingLogger::default())
            .service(web::redirect("/", HOME_ROUTE_REDIRECT_URL))
            .service(routes::embeddings)
            .service(routes::query)
            .service(routes::repo)
            .app_data(web::Data::new(model.clone()))
            .app_data(web::Data::new(db.clone()))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
