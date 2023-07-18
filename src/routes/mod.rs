use crate::utils::conversation::Query;
use crate::{db::RepositoryEmbeddingsDB, github::Repository};
use actix_web::{
    post,
    web::{self, Json},
    HttpResponse, Responder,
};
use reqwest::StatusCode;
use std::sync::Arc;

use crate::{db::QdrantDB, embeddings::Onnx, github::embed_repo};

#[post("/embeddings")]
async fn embeddings(
    data: Json<Repository>,
    db: web::Data<Arc<QdrantDB>>,
    model: web::Data<Arc<Onnx>>,
) -> impl Responder {
    let embeddings = embed_repo(data.into_inner(), model.get_ref().as_ref())
        .await
        .unwrap();

    match db.get_ref().insert_repo_embeddings(embeddings).await {
        Ok(_) => HttpResponse::new(StatusCode::CREATED),
        Err(e) => {
            dbg!(e);
            return HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
}

#[post("/query")]
async fn query(
    data: Json<Query>,
    db: web::Data<Arc<QdrantDB>>,
    model: web::Data<Arc<Onnx>>,
) -> impl Responder {
    HttpResponse::new(StatusCode::SERVICE_UNAVAILABLE)
}
