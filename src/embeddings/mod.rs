mod onnx;
use crate::prelude::*;

use ndarray::ArrayView1;
pub use onnx::*;
pub type Embeddings = Vec<f32>;

pub trait EmbeddingsModel {
    fn embed(&self, string: &str) -> Result<Embeddings>;
}

pub struct RelevantChunk {
    pub path: String,
    pub content: String
}

impl ToString for RelevantChunk {
    fn to_string(&self) -> String {
        format!("##Relevant file chunk##\nFile path:{}\nChunk content: {}", self.path, self.content)
    }
}

pub fn cosine_similarity(a: ArrayView1<f32>, b: ArrayView1<f32>) -> f32 {
    let dot_product = a.dot(&b);
    let norm_a = a.dot(&a).sqrt();
    let norm_b = b.dot(&b).sqrt();
    dot_product / (norm_a * norm_b)
}