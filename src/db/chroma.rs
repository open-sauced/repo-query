use chromadb::v1::{
    collection::{CollectionEntries, GetOptions, QueryOptions},
    ChromaClient,
};

use crate::{
    constants::MAX_FILES_COUNT,
    embeddings::Embeddings,
    github::{Repository, RepositoryEmbeddings, RepositoryFilePaths},
    prelude::*,
};

use super::RepositoryEmbeddingsDB;

pub struct ChromaDB {
    client: ChromaClient,
}

impl ChromaDB {
    pub fn initialize() -> Result<Self> {
        let client = ChromaClient::new(Default::default());
        Ok(ChromaDB { client })
    }
}

impl RepositoryEmbeddingsDB for ChromaDB {
    fn insert_repo_embeddings(&self, repo: RepositoryEmbeddings) -> Result<()> {
        let collection = self.client.get_or_create_collection(&repo.repo_id, None)?;
        let collection_entries = CollectionEntries {
            //Save the file paths as ids Eg: src/pages/index.js
            ids: repo
                .file_embeddings
                .iter()
                .map(|fe| fe.path.as_str())
                .collect(),
            embeddings: Some(
                repo.file_embeddings
                    .iter()
                    .map(|fe| fe.embeddings.clone())
                    .collect(),
            ),
            metadatas: None,
            documents: None,
        };
        collection.upsert(collection_entries, None)?;
        Ok(())
    }

    fn get_relevant_file_paths(
        &self,
        repository: &Repository,
        query_embeddings: Embeddings,
        limit: usize,
    ) -> Result<RepositoryFilePaths> {
        let collection = self
            .client
            .get_or_create_collection(&repository.to_string(), None)?;
        let query_options = QueryOptions {
            n_results: Some(limit),
            query_embeddings: Some(vec![query_embeddings]),
            query_texts: None,
            where_document: None,
            where_metadata: None,
            // We don't need the documents, embeddings, distances, or metadata. Only the ids.
            include: Some(vec![]),
        };

        let results = collection.query(query_options, None)?;

        Ok(RepositoryFilePaths {
            repo_id: repository.to_string(),
            file_paths: results.ids[0].clone(),
        })
    }

    fn get_file_paths(&self, repository: &Repository) -> Result<RepositoryFilePaths> {
        let collection = self
            .client
            .get_or_create_collection(&repository.to_string(), None)?;

        let get_options = GetOptions {
            include: Some(vec![]),
            limit: Some(MAX_FILES_COUNT),
            offset: None,
            ids: vec![],
            where_document: None,
            where_metadata: None,
        };

        let results = collection.get(get_options)?;

        Ok(RepositoryFilePaths {
            repo_id: repository.to_string(),
            file_paths: results.ids,
        })
    }

    fn is_indexed(&self, repository: &Repository) -> Result<bool> {
        match self.client.get_collection(&repository.to_string()) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}
