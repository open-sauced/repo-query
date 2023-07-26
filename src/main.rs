mod constants;
mod conversation;
mod db;
mod embeddings;
mod github;
mod prelude;
mod routes;
mod utils;
use std::{path::Path, sync::Arc};

use actix_web::{web, App, HttpResponse, HttpServer};
use constants::ACTIX_WEB_SERVER_PORT;
use env_logger::Env;
use tracing_actix_web::TracingLogger;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    env_logger::init_from_env(Env::default().default_filter_or("info"));

    let model: Arc<embeddings::Onnx> = Arc::new(embeddings::Onnx::new(Path::new("model")).unwrap());
    let db: Arc<db::QdrantDB> = Arc::new(db::QdrantDB::initialize().unwrap());

    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/", web::get().to(HttpResponse::Ok))
            .service(routes::embeddings)
            .service(routes::query)
            .service(routes::repo)
            .app_data(web::Data::new(model.clone()))
            .app_data(web::Data::new(db.clone()))
    })
    .bind(("0.0.0.0", ACTIX_WEB_SERVER_PORT as u16))?
    .run()
    .await
}
