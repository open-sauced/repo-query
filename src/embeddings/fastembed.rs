use crate::prelude::*;
use fastembed::{InitOptions, TextEmbedding};

use super::{Embeddings, EmbeddingsModel};

pub struct Fastembed {
    model: TextEmbedding,
}

impl Fastembed {
    pub fn try_new() -> Result<Self> {
        let model = TextEmbedding::try_new(InitOptions {
            model_name: fastembed::EmbeddingModel::AllMiniLML6V2,
            ..Default::default()
        })?;
        Ok(Self { model })
    }
}

impl EmbeddingsModel for Fastembed {
    fn embed<S: AsRef<str> + Send + Sync>(&self, texts: Vec<S>) -> Result<Vec<Embeddings>> {
        self.model.embed(texts, None)
    }

    fn query_embed<S: AsRef<str> + Send + Sync>(&self, query: S) -> Result<Embeddings> {
        let query = format!("query: {}.", query.as_ref());
        Ok(self.model.embed(vec![query], None)?[0].clone())
    }
}
