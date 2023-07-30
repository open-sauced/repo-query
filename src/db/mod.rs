use crate::conversation::RelevantChunk;
use crate::embeddings::Embeddings;
use crate::github::{Repository, RepositoryEmbeddings, RepositoryFilePaths};
use crate::prelude::*;
mod qdrant;
use async_trait::async_trait;

pub use qdrant::*;

#[async_trait]
pub trait RepositoryEmbeddingsDB {
    async fn insert_repo_embeddings(&self, repo: RepositoryEmbeddings) -> Result<()>;

    async fn get_relevant_chunks_from_codebase(
        &self,
        repository: &Repository,
        query_embeddings: Embeddings,
        limit: usize,
    ) -> Result<Vec<RelevantChunk>>;

    async fn get_paths(&self, repository: &Repository) -> Result<RepositoryFilePaths>;

    async fn get_relevant_chunks_from_path(
        &self,
        repository: &Repository,
        query_embeddings: Embeddings,
        file_path: &str,
        limit: usize,
    ) -> Result<Vec<RelevantChunk>>;

    async fn is_indexed(&self, repository: &Repository) -> Result<bool>;
}
