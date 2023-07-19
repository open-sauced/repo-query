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

use super::functions::Function;

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
            "##Relevant file chunk##\nFile path:{}\nChunk content: {}",
            self.path,
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

    pub async fn generate_answer(&self) {
        let request = generate_completion_request(self.messages.clone());
        let response = self.send_request(request).await.unwrap();
        match response.choices[0].finish_reason {
            FinishReason::function_call => {
                match response.choices[0].message.function_call.clone() {
                    Some(function_call) => {
                        let parsed_function_call = parse_function_call(function_call).unwrap();
                    }
                    None => {}
                }
            }
            _ => {}
        }
        {}
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

struct ParsedFunctionCall {
    name: Function,
    args: serde_json::Value,
}
