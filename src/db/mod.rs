use crate::embeddings::Embeddings;
use crate::github::{File, Repository, RepositoryEmbeddings, RepositoryFilePaths};
use crate::prelude::*;
mod qdrant;
use async_trait::async_trait;

pub use qdrant::*;

#[async_trait]
pub trait RepositoryEmbeddingsDB {
    async fn insert_repo_embeddings(&self, repo: RepositoryEmbeddings) -> Result<()>;

    async fn get_relevant_files(
        &self,
        repository: Repository,
        query_embeddings: Embeddings,
        limit: u64,
    ) -> Result<Vec<File>>;

    async fn get_file_paths(
        &self,
        repository: Repository
    ) -> Result<RepositoryFilePaths>;
}
