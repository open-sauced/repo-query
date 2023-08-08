use crate::embeddings::Embeddings;
use crate::github::{Repository, RepositoryEmbeddings, RepositoryFilePaths};
use crate::prelude::*;

mod chroma;

pub use chroma::*;

pub trait RepositoryEmbeddingsDB {
    fn insert_repo_embeddings(&self, repo: RepositoryEmbeddings) -> Result<()>;

    fn get_relevant_file_paths(
        &self,
        repository: &Repository,
        query_embeddings: Embeddings,
        limit: usize,
    ) -> Result<RepositoryFilePaths>;

    fn get_file_paths(&self, repository: &Repository) -> Result<RepositoryFilePaths>;

    fn is_indexed(&self, repository: &Repository) -> Result<bool>;
}
