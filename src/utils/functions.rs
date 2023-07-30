use std::str::FromStr;

use crate::{
    conversation::RelevantChunk, db::RepositoryEmbeddingsDB, embeddings::EmbeddingsModel,
    functions_enum, github::Repository, prelude::*,
};
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
) -> Result<Vec<RelevantChunk>> {
    let query_embeddings = model.embed(query)?;
    let relevant_files = db
        .get_relevant_chunks_from_codebase(repository, query_embeddings, files_limit)
        .await?;
    let relevant_chunks = relevant_files
        .par_iter()
        .map(|file| RelevantChunk {
            content: file.content.clone(),
            path: file.path.clone(),
        })
        .collect();
    Ok(relevant_chunks)
}

pub async fn search_file<M: EmbeddingsModel, D: RepositoryEmbeddingsDB>(
    path: &str,
    query: &str,
    repository: &Repository,
    model: &M,
    db: &D,
    chunks_limit: usize,
) -> Result<Vec<RelevantChunk>> {
    let query_embeddings = model.embed(query)?;
    let file_chunks = db
        .get_relevant_chunks_from_path(repository, query_embeddings, path, chunks_limit)
        .await?;

    let relevant_chunks: Vec<RelevantChunk> = file_chunks
        .iter()
        .map(|file| RelevantChunk {
            path: path.to_string(),
            content: file.content.clone(),
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
    let list = db.get_paths(repository).await?;
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
    dbg!(&chunks);
    ChatCompletionMessage {
        name: Some(function_name.to_string()),
        role: MessageRole::function,
        content: chunks,
        function_call: None,
    }
}

//Remove extra whitespaces from chunks
pub fn clean_chunks(chunks: Vec<&str>) -> Vec<String> {
    chunks
        .iter()
        .map(|s| s.split_whitespace().collect::<Vec<&str>>().join(" "))
        .collect()
}
