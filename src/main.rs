mod constants;
mod conversation;
mod db;
mod embeddings;
mod github;
mod prelude;
mod routes;
mod utils;
use std::{path::Path, sync::Arc};

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use constants::HOME_ROUTE_REDIRECT_URL;
use env_logger::Env;
use tracing_actix_web::TracingLogger;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let model: Arc<embeddings::Onnx> = Arc::new(embeddings::Onnx::new(Path::new("model")).unwrap());
    let db: Arc<db::ChromaDB> = Arc::new(db::ChromaDB::initialize().unwrap());

    let port = std::env::var("WEBSERVER_PORT").expect("WEBSERVER_PORT not set");
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
