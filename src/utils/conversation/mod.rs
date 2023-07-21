#![allow(unused_must_use)]
mod prompts;

use crate::{
    prelude::*,
    constants::{RELEVANT_CHUNKS_LIMIT, RELEVANT_FILES_LIMIT},
    db::RepositoryEmbeddingsDB,
    embeddings::EmbeddingsModel,
    github::Repository,
    routes::events::{emit, QueryEvent},
};
use actix_web_lab::sse::Sender;
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

use self::prompts::answer_generation_prompt;

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
    pub content: String,
}

impl ToString for RelevantChunk {
    fn to_string(&self) -> String {
        format!(
            "##Relevant file chunk##\nPath argument:{}\nRelevant content: {}",
            self.path,
            self.content.trim()
        )
    }
}

#[derive(Debug, Clone)]
struct ParsedFunctionCall {
    name: Function,
    args: serde_json::Value,
}

pub struct Conversation<D: RepositoryEmbeddingsDB, M: EmbeddingsModel> {
    query: Query,
    client: Client,
    messages: Vec<ChatCompletionMessage>,
    db: Arc<D>,
    model: Arc<M>,
    sender: Sender,
}

impl<D: RepositoryEmbeddingsDB, M: EmbeddingsModel> Conversation<D, M> {
    pub fn new(query: Query, db: Arc<D>, model: Arc<M>, sender: Sender) -> Self {
        Self {
            client: Client::new(env::var("OPENAI_API_KEY").unwrap()),
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
            sender,
        }
    }

    fn append_message(&mut self, message: ChatCompletionMessage) {
        self.messages.push(message);
    }

    fn prepare_final_explanation_message(&mut self) {
        //Update the system prompt using answer_generation_prompt()
        self.messages[0] = ChatCompletionMessage {
            name: None,
            function_call: None,
            role: MessageRole::system,
            content: Some(answer_generation_prompt()),
        }
    }

    async fn send_request(&self, request: ChatCompletionRequest) -> Result<ChatCompletionResponse> {
        Ok(self.client.chat_completion(request).await?)
    }

    pub async fn generate(&mut self) -> Result<()> {
        #[allow(unused_labels)]
        'conversation: loop {
            //Generate a request with the message history and functions
            let request = generate_completion_request(self.messages.clone(), true);

            match self.send_request(request).await {
                Ok(response) => {
                    if let FinishReason::function_call = response.choices[0].finish_reason {
                        if let Some(function_call) =
                            response.choices[0].message.function_call.clone()
                        {
                            let parsed_function_call = parse_function_call(&function_call)?;
                            let function_call_message = ChatCompletionMessage {
                                name: None,
                                function_call: Some(function_call),
                                role: MessageRole::assistant,
                                content: Some(String::new()),
                            };
                            self.append_message(function_call_message);
                            dbg!(parsed_function_call.clone());
                            match parsed_function_call.name {
                                Function::SearchCodebase => {
                                    let query: &str = parsed_function_call.args["query"]
                                        .as_str()
                                        .unwrap_or_default();
                                    emit(
                                        &self.sender,
                                        QueryEvent::SearchCodebase(Some(
                                            parsed_function_call.clone().args,
                                        )),
                                    )
                                    .await;
                                    let relevant_chunks = search_codebase(
                                        query,
                                        &self.query.repository,
                                        self.model.as_ref(),
                                        self.db.as_ref(),
                                        RELEVANT_FILES_LIMIT,
                                        RELEVANT_CHUNKS_LIMIT,
                                    )
                                    .await?;
                                    let completion_message = relevant_chunks_to_completion_message(
                                        parsed_function_call.name,
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
                                    emit(
                                        &self.sender,
                                        QueryEvent::SearchFile(Some(
                                            parsed_function_call.clone().args,
                                        )),
                                    )
                                    .await;
                                    let relevant_chunks = search_file(
                                        path,
                                        query,
                                        &self.query.repository,
                                        self.model.as_ref(),
                                        RELEVANT_CHUNKS_LIMIT,
                                    )
                                    .await?;
                                    let completion_message = relevant_chunks_to_completion_message(
                                        parsed_function_call.name,
                                        relevant_chunks,
                                    );
                                    self.append_message(completion_message);
                                }
                                Function::SearchPath => {
                                    let path: &str = parsed_function_call.args["path"]
                                        .as_str()
                                        .unwrap_or_default();
                                    emit(
                                        &self.sender,
                                        QueryEvent::SearchPath(Some(
                                            parsed_function_call.clone().args,
                                        )),
                                    )
                                    .await;
                                    let fuzzy_matched_paths = search_path(
                                        path,
                                        &self.query.repository,
                                        self.db.as_ref(),
                                        1,
                                    )
                                    .await?;
                                    let completion_message = paths_to_completion_message(
                                        parsed_function_call.name,
                                        fuzzy_matched_paths,
                                    );
                                    self.append_message(completion_message);
                                }
                                Function::None => {
                                    self.prepare_final_explanation_message();

                                    //Generate a request with the message history and no functions
                                    let request =
                                        generate_completion_request(self.messages.clone(), false);
                                    emit(
                                        &self.sender,
                                        QueryEvent::GenerateResponse(Some(
                                            parsed_function_call.args,
                                        )),
                                    )
                                    .await;
                                    let response = match self.send_request(request).await {
                                        Ok(response) => response,
                                        Err(e) => {
                                            return Err(e);
                                        }
                                    };
                                    let response = response.choices[0]
                                        .message
                                        .content
                                        .clone()
                                        .unwrap_or_default();
                                    emit(&self.sender, QueryEvent::Done(Some(response.into())))
                                        .await;
                                    return Ok(());
                                }
                            }
                        };
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            };
        }
    }
}

fn parse_function_call(func: &FunctionCall) -> Result<ParsedFunctionCall> {
    let func = func.clone();
    let function_name = Function::from_str(&func.name.unwrap_or("none".into()))?;
    let function_args = func.arguments.unwrap_or("{}".to_string());
    let function_args = serde_json::from_str::<serde_json::Value>(&function_args)?;
    Ok(ParsedFunctionCall {
        name: function_name,
        args: function_args,
    })
}
