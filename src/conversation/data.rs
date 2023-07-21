use crate::prelude::*;
use crate::{github::Repository, utils::functions::Function};
use openai_api_rs::v1::chat_completion::FunctionCall;
use serde::Deserialize;
use std::str::FromStr;

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
pub struct ParsedFunctionCall {
    pub name: Function,
    pub args: serde_json::Value,
}

impl TryFrom<&FunctionCall> for ParsedFunctionCall {
    type Error = anyhow::Error;

    fn try_from(func: &FunctionCall) -> Result<Self> {
        let func = func.clone();
        let name = Function::from_str(&func.name.unwrap_or("done".into()))?;
        let args = func.arguments.unwrap_or("{}".to_string());
        let args = serde_json::from_str::<serde_json::Value>(&args)?;
        Ok(ParsedFunctionCall { name, args })
    }
}
