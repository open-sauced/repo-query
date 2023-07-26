use crate::prelude::*;
use ndarray::{Array, Axis, CowArray};
use ort::{
    execution_providers::CPUExecutionProviderOptions, Environment, ExecutionProvider,
    GraphOptimizationLevel, SessionBuilder, Value,
};
use std::{path::Path, sync::Arc, thread::available_parallelism};

use super::{Embeddings, EmbeddingsModel};

#[derive(Clone, Debug)]
pub struct Onnx {
    tokenizer: Arc<tokenizers::Tokenizer>,
    session: Arc<ort::Session>,
}

impl Onnx {
    pub fn new(model_dir: &Path) -> Result<Self> {
        let environment = Arc::new(
            Environment::builder()
                .with_name("Embeddings")
                .with_execution_providers([ExecutionProvider::CPU(
                    CPUExecutionProviderOptions::default(),
                )])
                .build()?,
        );

        let threads = available_parallelism().unwrap().get() as i16;

        Ok(Self {
            tokenizer: tokenizers::Tokenizer::from_file(model_dir.join("tokenizer.json"))
                .unwrap()
                .into(),
            session: SessionBuilder::new(&environment)?
                .with_optimization_level(GraphOptimizationLevel::Level3)?
                .with_intra_threads(threads)?
                .with_model_from_file(model_dir.join("model_quantized.onnx"))?
                .into(),
        })
    }
}

impl EmbeddingsModel for Onnx {
    fn embed(&self, sequence: &str) -> Result<Embeddings> {
        let tokenizer_output = self.tokenizer.encode(sequence, true).unwrap();

        let input_ids = tokenizer_output.get_ids();
        let attention_mask = tokenizer_output.get_attention_mask();
        let token_type_ids = tokenizer_output.get_type_ids();
        let length = input_ids.len();

        let inputs_ids_array = CowArray::from(Array::from_shape_vec(
            (1, length),
            input_ids.iter().map(|&x| x as i64).collect(),
        )?)
        .into_dyn();

        let attention_mask_array = CowArray::from(Array::from_shape_vec(
            (1, length),
            attention_mask.iter().map(|&x| x as i64).collect(),
        )?)
        .into_dyn();

        let token_type_ids_array = CowArray::from(Array::from_shape_vec(
            (1, length),
            token_type_ids.iter().map(|&x| x as i64).collect(),
        )?)
        .into_dyn();

        let outputs = self.session.run(vec![
            Value::from_array(self.session.allocator(), &inputs_ids_array)?,
            Value::from_array(self.session.allocator(), &attention_mask_array)?,
            Value::from_array(self.session.allocator(), &token_type_ids_array)?,
        ])?;

        let output_tensor = outputs[0].try_extract().unwrap();
        let sequence_embedding = &*output_tensor.view();
        let pooled = sequence_embedding.mean_axis(Axis(1)).unwrap();
        Ok(pooled.to_owned().as_slice().unwrap().to_vec())
    }
}
