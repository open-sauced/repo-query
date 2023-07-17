mod db;
mod embeddings;
mod github;
mod prelude;
mod routes;
mod utils;
use std::{path::Path, sync::Arc};

use actix_web::{web, App, HttpResponse, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let model: Arc<embeddings::Onnx> = Arc::new(embeddings::Onnx::new(Path::new("model")).unwrap());
    let db: Arc<db::QdrantDB> = Arc::new(db::QdrantDB::initialize().unwrap());

    HttpServer::new(move || {
        App::new()
            .route("/", web::get().to(|| HttpResponse::Ok()))
            .service(routes::embeddings)
            .service(routes::query)
            .app_data(web::Data::new(model.clone()))
            .app_data(web::Data::new(db.clone()))
    })
    .bind(("0.0.0.0", 3001))?
    .run()
    .await
}
