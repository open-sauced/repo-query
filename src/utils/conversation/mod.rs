mod prompts;

use crate::db::RepositoryEmbeddingsDB;
use crate::prelude::*;
use crate::{embeddings::EmbeddingsModel, github::Repository};
use openai_api_rs::v1::chat_completion::{FinishReason, FunctionCall};
use openai_api_rs::v1::{
    api::Client,
    chat_completion::{
        ChatCompletionMessage, ChatCompletionRequest, ChatCompletionResponse, MessageRole,
    },
};

use serde::Deserialize;
use std::env;
use std::str::FromStr;
use std::sync::Arc;

use prompts::{generate_completion_request, system_message};

use super::functions::{
    paths_to_completion_message, relevant_chunks_to_completion_message, search_codebase,
    search_file, search_path, Function,
};

#[derive(Deserialize)]
pub struct Query {
    pub repository: Repository,
    pub query: String,
}

impl ToString for Query {
    fn to_string(&self) -> String {
        let Query {
            repository:
                Repository {
                    owner,
                    name,
                    branch,
                },
            query,
        } = self;
        format!(
            "##Repository Info##\nOwner:{}\nName:{}\nBranch:{}\n##User Query##\nQuery:{}",
            owner, name, branch, query
        )
    }
}

#[derive(Debug)]
pub struct RelevantChunk {
    pub path: String,
    pub query: String,
    pub content: String,
}

impl ToString for RelevantChunk {
    fn to_string(&self) -> String {
        format!(
            "##Relevant file chunk##\nPath argument:{}\nQuery argument:{}\nRelevant content: {}",
            self.path,
            self.query,
            self.content.trim()
        )
    }
}

pub struct Conversation<D: RepositoryEmbeddingsDB, M: EmbeddingsModel> {
    query: Query,
    client: Client,
    messages: Vec<ChatCompletionMessage>,
    db: Arc<D>,
    model: Arc<M>,
}

impl<D: RepositoryEmbeddingsDB, M: EmbeddingsModel> Conversation<D, M> {
    pub fn new(query: Query, db: Arc<D>, model: Arc<M>) -> Self {
        Self {
            client: Client::new(env::var("OPENAI_API_KEY").unwrap().to_string()),
            messages: vec![
                ChatCompletionMessage {
                    name: None,
                    function_call: None,
                    role: MessageRole::system,
                    content: Some(system_message()),
                },
                ChatCompletionMessage {
                    name: None,
                    function_call: None,
                    role: MessageRole::user,
                    content: Some(query.to_string()),
                },
            ],
            query,
            db,
            model,
        }
    }

    fn append_message(&mut self, message: ChatCompletionMessage) {
        self.messages.push(message);
    }

    async fn send_request(&self, request: ChatCompletionRequest) -> Result<ChatCompletionResponse> {
        Ok(self.client.chat_completion(request).await?)
    }

    pub async fn generate_answer(&mut self) -> Result<()> {
        'conversation: loop {
            let request = generate_completion_request(self.messages.clone());
            let response = self.send_request(request).await.unwrap();

            if let FinishReason::function_call = response.choices[0].finish_reason {
                if let Some(function_call) = response.choices[0].message.function_call.clone() {
                    let parsed_function_call = parse_function_call(function_call)?;
                    dbg!(parsed_function_call.clone());
                    match parsed_function_call.name {
                        Function::SearchCodebase => {
                            let query: &str = parsed_function_call.args["query"]
                                .as_str()
                                .unwrap_or_default();
                            let relevant_chunks = search_codebase(
                                query,
                                &self.query.repository,
                                self.model.as_ref(),
                                self.db.as_ref(),
                                3,
                                2,
                            )
                            .await?;
                            let completion_message = relevant_chunks_to_completion_message(
                                parsed_function_call.name.to_string(),
                                relevant_chunks,
                            );
                            self.append_message(completion_message);
                        }
                        Function::SearchFile => {
                            let query: &str = parsed_function_call.args["query"]
                                .as_str()
                                .unwrap_or_default();
                            let path: &str = parsed_function_call.args["path"]
                                .as_str()
                                .unwrap_or_default();
                            let relevant_chunks = search_file(
                                path,
                                query,
                                &self.query.repository,
                                self.model.as_ref(),
                                2,
                            )
                            .await?;
                            let completion_message = relevant_chunks_to_completion_message(
                                parsed_function_call.name.to_string(),
                                relevant_chunks,
                            );
                            self.append_message(completion_message);
                        }
                        Function::SearchPath => {
                            let path: &str = parsed_function_call.args["path"]
                                .as_str()
                                .unwrap_or_default();
                            let fuzzy_matched_paths =
                                search_path(path, &self.query.repository, self.db.as_ref(), 1)
                                    .await?;
                            let completion_message = paths_to_completion_message(
                                parsed_function_call.name.to_string(),
                                fuzzy_matched_paths,
                            );
                            self.append_message(completion_message);
                        }
                        Function::None => {
                            break 'conversation;
                        }
                    }
                };
            }
        }
        Ok(())
    }
}

fn parse_function_call(mut func: FunctionCall) -> Result<ParsedFunctionCall> {
    let function_name = Function::from_str(&func.name.get_or_insert("none".into())).unwrap();
    let function_args = func.arguments.get_or_insert("{}".to_string());
    let function_args = serde_json::from_str::<serde_json::Value>(function_args)?;
    Ok(ParsedFunctionCall {
        name: function_name,
        args: function_args,
    })
}

#[derive(Debug, Clone)]
struct ParsedFunctionCall {
    name: Function,
    args: serde_json::Value,
}
