use crate::{
    db::RepositoryEmbeddingsDB,
    embeddings::{cosine_similarity, Embeddings, EmbeddingsModel, RelevantChunk},
    github::{fetch_file_content, Repository, RepositoryFilePaths},
    prelude::*,
};
use ndarray::ArrayView1;
use rayon::prelude::*;

pub fn search_codebase<M: EmbeddingsModel, D: RepositoryEmbeddingsDB>(
    query: &str,
    repository: Repository,
    model: &M,
    db: &D,
) -> Result<()> {
    let query_embeddings = model.embed(query)?;
    Ok(())
}

pub async fn search_file<M: EmbeddingsModel>(
    path: &str,
    query: &str,
    repository: Repository,
    model: &M,
    limit: usize,
) -> Result<Vec<RelevantChunk>> {
    let file_content = fetch_file_content(repository, path).await?;
    let splitter = text_splitter::TextSplitter::default();
    let chunks: Vec<&str> = splitter.chunks(&file_content, 300).collect();
    let chunks_embeddings: Vec<Embeddings> = chunks
        .iter()
        .map(|chunk| model.embed(chunk).unwrap())
        .collect();
    let query_embeddings = model.embed(query)?;
    let similarities: Vec<f32> = chunks_embeddings
        .par_iter()
        .map(|embedding| {
            cosine_similarity(
                ArrayView1::from(&query_embeddings),
                ArrayView1::from(embedding),
            )
        })
        .collect();
    let mut indexed_vec: Vec<(usize, &f32)> = similarities.par_iter().enumerate().collect();
    indexed_vec.par_sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
    let indices: Vec<usize> = indexed_vec.iter().map(|x| x.0).take(limit).collect();

    let relevant_chunks: Vec<RelevantChunk> = indices
        .iter()
        .map(|index| RelevantChunk {
            path: path.to_string(),
            content: chunks[*index].to_string(),
        })
        .collect();
    Ok(relevant_chunks)
}

pub fn search_path(path: &str, list: RepositoryFilePaths) -> Vec<String> {
    let file_paths = list.file_paths;
    let response: Vec<(&str, f32)> = rust_fuzzy_search::fuzzy_search_best_n(
        path,
        &file_paths.iter().map(String::as_ref).collect::<Vec<&str>>(),
        3,
    );
    let file_paths = response
        .iter()
        .map(|(path, _)| path.to_string())
        .collect::<Vec<String>>();
    file_paths
}
