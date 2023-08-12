#![allow(unused_must_use)]
pub mod events;
use crate::{
    constants::SSE_CHANNEL_BUFFER_SIZE,
    conversation::{Conversation, Query},
    db::{ChromaDB, RepositoryEmbeddingsDB},
    embeddings::Onnx,
    github::{fetch_repo_files, Repository, embed_repo, fetch_license_info},
    routes::events::QueryEvent,
};

use actix_web::{web::{Query as ActixQuery, self, Json}, error::{ErrorForbidden, ErrorBadRequest, ErrorNotFound}, HttpResponse, get, post, Responder, Result};
use actix_web_lab::sse;
use serde_json::json;
use std::sync::Arc;

use events::{emit, EmbedEvent};

#[post("/embed")]
async fn embeddings(
    data: Json<Repository>,
    db: web::Data<Arc<ChromaDB>>,
    model: web::Data<Arc<Onnx>>,
) -> Result<impl Responder> {
    let license_info = fetch_license_info(&data).await.map_err(ErrorBadRequest)?;
    if !license_info.permissible {
        return Err(ErrorForbidden(license_info.error.unwrap_or_default()));
    }

    let (sender, rx) = sse::channel(SSE_CHANNEL_BUFFER_SIZE);

    actix_rt::spawn(async move {
        let handle_embed = async {
            let repository = data.into_inner();

            emit(&sender, EmbedEvent::FetchRepo(None)).await;
            let files = fetch_repo_files(&repository).await?;

            emit(
                &sender,
                EmbedEvent::EmbedRepo(Some(json!({
                    "files": files.len(),
                }))),
            )
            .await;
            let embeddings = embed_repo(&repository, files, model.get_ref().as_ref(), &sender).await?;

            emit(&sender, EmbedEvent::SaveEmbeddings(None)).await;
            db.get_ref().insert_repo_embeddings(embeddings)?;

            emit(&sender, EmbedEvent::Done(None)).await;
            Ok::<(), anyhow::Error>(())
        };

        if let Err(e) = handle_embed.await {
            eprintln!("/embed error: {}", e);
            emit(&sender, EmbedEvent::Error(Some(e.to_string().into()))).await;
        }
    });

    Ok(rx)
}

#[post("/query")]
async fn query(
    data: Json<Query>,
    db: web::Data<Arc<ChromaDB>>,
    model: web::Data<Arc<Onnx>>,
) -> Result<impl Responder> {
    if db.is_indexed(&data.repository).unwrap_or_default() {
        let (sender, rx) = sse::channel(SSE_CHANNEL_BUFFER_SIZE);

        actix_rt::spawn(async move {
            let result = async {
                let mut conversation = Conversation::initiate(
                    data.into_inner(),
                    db.get_ref().clone(),
                    model.get_ref().clone(),
                    sender.clone(),
                )
                .await?;
                conversation.generate().await?;

                Ok::<(), anyhow::Error>(())
            };
            if let Err(e) = result.await {
                eprintln!("/query error: {}", e);
                emit(&sender, QueryEvent::Error(Some(e.to_string().into()))).await;
            }
        });

        Ok(rx)
    } else {
        Err(ErrorNotFound("Repository is not indexed"))
    }
}

#[get("/collection")]
async fn repo(
    data: ActixQuery<Repository>,
    db: web::Data<Arc<ChromaDB>>,
) -> Result<impl Responder> {
    let is_indexed = db.is_indexed(&data.into_inner()).unwrap_or_default();

    if is_indexed {
        Ok(HttpResponse::Ok())
    } else {
        Err(ErrorNotFound("Repository is not indexed"))
    }
}
