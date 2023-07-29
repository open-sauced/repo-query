use std::collections::HashMap;

use super::RepositoryEmbeddingsDB;
use crate::{
    constants::{EMBEDDINGS_DIMENSION, MAX_FILES_COUNT},
    embeddings::Embeddings,
    github::{FileEmbeddings, Repository, RepositoryEmbeddings, RepositoryFilePaths},
    prelude::*,
};
use anyhow::Ok;
use async_trait::async_trait;
use qdrant_client::{
    prelude::*,
    qdrant::{vectors_config::Config, ScrollPoints, VectorParams, VectorsConfig},
};
use rayon::prelude::*;

pub struct QdrantDB {
    client: QdrantClient,
}

#[async_trait]
impl RepositoryEmbeddingsDB for QdrantDB {
    async fn insert_repo_embeddings(&self, repo: RepositoryEmbeddings) -> Result<()> {
        if self.client.has_collection(&repo.repo_id).await? {
            self.client.delete_collection(&repo.repo_id).await?;
        }
        self.client
            .create_collection(&CreateCollection {
                collection_name: repo.repo_id.clone(),
                vectors_config: Some(VectorsConfig {
                    config: Some(Config::Params(VectorParams {
                        size: EMBEDDINGS_DIMENSION as u64,
                        distance: Distance::Cosine.into(),
                        ..Default::default()
                    })),
                }),
                ..Default::default()
            })
            .await?;

        let points: Vec<PointStruct> = repo
            .file_embeddings
            .into_par_iter()
            .enumerate()
            .map(|file| {
                let FileEmbeddings { path, embeddings } = file.1;
                let payload: Payload = HashMap::from([("path", path.into())]).into();

                PointStruct::new(file.0 as u64, embeddings, payload)
            })
            .collect();
        self.client
            .upsert_points(repo.repo_id, points, None)
            .await?;
        Ok(())
    }

    async fn get_relevant_files(
        &self,
        repository: &Repository,
        query_embeddings: Embeddings,
        limit: usize,
    ) -> Result<RepositoryFilePaths> {
        let search_response = self
            .client
            .search_points(&SearchPoints {
                collection_name: repository.to_string(),
                vector: query_embeddings,
                with_payload: Some(true.into()),
                limit: limit as u64,
                ..Default::default()
            })
            .await?;
        let paths: Vec<String> = search_response
            .result
            .into_iter()
            .map(|point| point.payload["path"].to_string().replace('\"', ""))
            .collect();
        Ok(RepositoryFilePaths {
            repo_id: repository.to_string(),
            file_paths: paths,
        })
    }

    async fn get_file_paths(&self, repository: &Repository) -> Result<RepositoryFilePaths> {
        let scroll_reponse = self
            .client
            .scroll(&ScrollPoints {
                collection_name: repository.to_string(),
                offset: None,
                filter: None,
                limit: Some(MAX_FILES_COUNT as u32),
                with_payload: Some(true.into()),
                with_vectors: None,
                read_consistency: None,
            })
            .await?;

        let file_paths: Vec<String> = scroll_reponse
            .result
            .par_iter()
            .map(|point| point.payload["path"].to_string().replace('\"', ""))
            .collect();
        Ok(RepositoryFilePaths {
            repo_id: repository.to_string(),
            file_paths,
        })
    }

    async fn is_indexed(&self, repository: &Repository) -> Result<bool> {
        self.client.has_collection(repository.to_string()).await
    }
}

impl QdrantDB {
    pub fn initialize() -> Result<QdrantDB> {
        let qdrant_url =
            std::env::var("QDRANT_URL").unwrap_or(String::from("http://localhost:6334"));
            dbg!(&qdrant_url);
        let config = QdrantClientConfig::from_url(&qdrant_url);
        let client = QdrantClient::new(Some(config))?;
        Ok(QdrantDB { client })
    }
}
