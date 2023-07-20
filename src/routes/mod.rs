use crate::utils::conversation::{Conversation, Query};
use crate::{db::RepositoryEmbeddingsDB, github::Repository};
use actix_web::{
    get, post,
    web::{self, Json},
    HttpResponse, Responder, Result,
};
use reqwest::StatusCode;
use std::sync::Arc;

use crate::{db::QdrantDB, embeddings::Onnx, github::embed_repo};

#[derive(serde::Deserialize)]
struct Q {
    q: String,
}

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

#[get("/query/{owner}/{name}/{branch}")]
async fn query(
    path: web::Path<Repository>,
    info: web::Query<Q>,
    db: web::Data<Arc<QdrantDB>>,
    model: web::Data<Arc<Onnx>>,
) -> Result<impl Responder> {
    let query = Query {
        query: info.q.to_owned(),
        repository: path.into_inner().to_owned(),
    };
    let mut conversation = Conversation::new(query, db.get_ref().clone(), model.get_ref().clone());
    match conversation.generate().await {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(e) => {
            dbg!(e);
            Ok(HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR))
        }
    }
}
