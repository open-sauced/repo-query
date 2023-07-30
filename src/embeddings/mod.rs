mod onnx;

use crate::prelude::*;

pub use onnx::*;
pub type Embeddings = Vec<f32>;

pub trait EmbeddingsModel {
    fn embed(&self, string: &str) -> Result<Embeddings>;
}
