use std::str::FromStr;

use crate::{
    constants::FILE_CHUNKER_CAPACITY_RANGE,
    conversation::RelevantChunk,
    db::RepositoryEmbeddingsDB,
    embeddings::{cosine_similarity, Embeddings, EmbeddingsModel},
    functions_enum,
    github::{fetch_file_content, Repository},
    prelude::*,
};
use ndarray::ArrayView1;
use openai_api_rs::v1::chat_completion::{ChatCompletionMessage, MessageRole};
use rayon::prelude::*;

functions_enum! {
    Function,
    (SearchCodebase, "search_codebase"),
    (SearchFile, "search_file"),
    (SearchPath, "search_path"),
    (Done, "done"),
}

pub async fn search_codebase<M: EmbeddingsModel, D: RepositoryEmbeddingsDB>(
    query: &str,
    repository: &Repository,
    model: &M,
    db: &D,
    files_limit: usize,
    chunks_limit: usize,
) -> Result<Vec<RelevantChunk>> {
    let query_embeddings = model.embed(query)?;
    let relevant_files = db
        .get_relevant_file_paths(repository, query_embeddings, files_limit)?
        .file_paths;
    let mut relevant_chunks: Vec<RelevantChunk> = Vec::new();
    for path in relevant_files {
        let chunks = search_file(&path, query, repository, model, chunks_limit).await?;
        relevant_chunks.extend(chunks);
    }

    Ok(relevant_chunks)
}

pub async fn search_file<M: EmbeddingsModel>(
    path: &str,
    query: &str,
    repository: &Repository,
    model: &M,
    chunks_limit: usize,
) -> Result<Vec<RelevantChunk>> {
    let file_content = fetch_file_content(repository, path)
        .await
        .unwrap_or_default();

    let splitter = text_splitter::TextSplitter::default().with_trim_chunks(true);

    let chunks: Vec<&str> = splitter
        .chunks(&file_content, FILE_CHUNKER_CAPACITY_RANGE)
        .collect();

    let cleaned_chunks: Vec<String> = clean_chunks(chunks);
    let chunks_embeddings: Vec<Embeddings> = cleaned_chunks
        .iter()
        .map(|chunk| model.embed(chunk).unwrap())
        .collect();

    let query_embeddings = model.embed(query)?;

    let similarities: Vec<f32> = similarity_score(chunks_embeddings, query_embeddings);

    let indices = get_top_n_indices(similarities, chunks_limit);

    let relevant_chunks: Vec<RelevantChunk> = indices
        .iter()
        .map(|index| RelevantChunk {
            path: path.to_string(),
            content: cleaned_chunks[*index].to_string(),
        })
        .collect();
    Ok(relevant_chunks)
}

pub async fn search_path<D: RepositoryEmbeddingsDB>(
    path: &str,
    repository: &Repository,
    db: &D,
    limit: usize,
) -> Result<Vec<String>> {
    let list = db.get_file_paths(repository)?;
    let file_paths: Vec<&str> = list.file_paths.iter().map(String::as_ref).collect();
    let response: Vec<(&str, f32)> =
        rust_fuzzy_search::fuzzy_search_best_n(path, &file_paths, limit);
    let file_paths = response
        .iter()
        .map(|(path, _)| path.to_string())
        .collect::<Vec<String>>();
    Ok(file_paths)
}

pub fn paths_to_completion_message(
    function_name: Function,
    paths: Vec<String>,
) -> ChatCompletionMessage {
    let paths = paths.join(", ");

    ChatCompletionMessage {
        name: Some(function_name.to_string()),
        role: MessageRole::function,
        content: paths,
        function_call: None,
    }
}

pub fn relevant_chunks_to_completion_message(
    function_name: Function,
    relevant_chunks: Vec<RelevantChunk>,
) -> ChatCompletionMessage {
    let chunks = relevant_chunks
        .iter()
        .map(|chunk| chunk.to_string())
        .collect::<Vec<String>>()
        .join("\n\n");

    ChatCompletionMessage {
        name: Some(function_name.to_string()),
        role: MessageRole::function,
        content: chunks,
        function_call: None,
    }
}

//Remove extra whitespaces from chunks
fn clean_chunks(chunks: Vec<&str>) -> Vec<String> {
    chunks
        .iter()
        .map(|s| s.split_whitespace().collect::<Vec<&str>>().join(" "))
        .collect()
}

//Compute cosine similarity between query and file content chunks
fn similarity_score(files_embeddings: Vec<Embeddings>, query_embeddings: Embeddings) -> Vec<f32> {
    files_embeddings
        .par_iter()
        .map(|embedding| {
            cosine_similarity(
                ArrayView1::from(&query_embeddings),
                ArrayView1::from(embedding),
            )
        })
        .collect()
}

//Get n indices with highest similarity scores
fn get_top_n_indices(similarity_scores: Vec<f32>, n: usize) -> Vec<usize> {
    let mut indexed_vec: Vec<(usize, &f32)> = similarity_scores.par_iter().enumerate().collect();
    indexed_vec.par_sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
    indexed_vec.iter().map(|x| x.0).take(n).collect()
}
