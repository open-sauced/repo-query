mod prompts;

use crate::github::Repository;
use crate::prelude::*;
use openai_api_rs::v1::{
    api::Client,
    chat_completion::{
        ChatCompletionMessage, ChatCompletionRequest, ChatCompletionResponse, MessageRole,
    },
};

use serde::Deserialize;
use std::env;

use prompts::{generate_completion_request, system_message};

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

pub struct Conversation {
    query: Query,
    client: Client,
    messages: Vec<ChatCompletionMessage>,
}

impl Conversation {
    pub fn new(query: Query) -> Self {
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
        }
    }

    fn append_message(&mut self, message: ChatCompletionMessage) {
        self.messages.push(message);
    }

    async fn send_request(&self, request: ChatCompletionRequest) -> Result<ChatCompletionResponse> {
        Ok(self.client.chat_completion(request).await?)
    }

    pub async fn generate_answer(&self) {
        'conversation: loop {
            let _request = generate_completion_request(self.messages.clone());
        }
    }
}
