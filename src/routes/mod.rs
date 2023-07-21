#![allow(unused_must_use)]
mod events;

use crate::github::fetch_repo_files;
use crate::utils::conversation::{Conversation, Query};
use crate::{db::RepositoryEmbeddingsDB, github::Repository};
use actix_web::{
    post,
    web::{self, Json},
    Responder,
};
use actix_web_lab::sse;
use std::sync::Arc;

use crate::{db::QdrantDB, embeddings::Onnx, github::embed_repo};
use events::{emit, EmbedEvent};

#[post("/embed")]
async fn embeddings(
    data: Json<Repository>,
    db: web::Data<Arc<QdrantDB>>,
    model: web::Data<Arc<Onnx>>,
) -> impl Responder {
    let (tx, rx) = sse::channel(1);

    actix_rt::spawn(async move {
        let repository = data.into_inner();

        emit(&tx, EmbedEvent::FetchRepo.into()).await;
        let files = fetch_repo_files(&repository).await?;

        emit(&tx, EmbedEvent::EmbedRepo.into()).await;
        let embeddings = embed_repo(&repository, files, model.get_ref().as_ref()).await?;

        emit(&tx, EmbedEvent::SaveEmbeddings.into()).await;
        db.get_ref().insert_repo_embeddings(embeddings).await?;

        Ok::<(), anyhow::Error>(())
    });

    rx
}

#[post("/query")]
async fn query(
    data: Json<Query>,
    db: web::Data<Arc<QdrantDB>>,
    model: web::Data<Arc<Onnx>>,
) -> impl Responder {
    let (tx, rx) = sse::channel(1);

    actix_rt::spawn(async move {
        let mut conversation = Conversation::new(
            data.into_inner(),
            db.get_ref().clone(),
            model.get_ref().clone(),
            tx,
        );
        let _ = conversation.generate().await;
    });

    rx
}
