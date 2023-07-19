use openai_api_rs::v1::chat_completion::{
    ChatCompletionMessage, ChatCompletionRequest, Function, FunctionParameters, JSONSchemaDefine,
    JSONSchemaType, GPT3_5_TURBO,
};
use std::collections::HashMap;

pub fn generate_completion_request(messages: Vec<ChatCompletionMessage>) -> ChatCompletionRequest {
    ChatCompletionRequest {
        model: GPT3_5_TURBO.into(),
        messages,
        functions: Some(functions()),
        function_call: Some("auto".to_string()),
        temperature: None,
        top_p: None,
        n: None,
        stream: None,
        stop: None,
        max_tokens: None,
        presence_penalty: None,
        frequency_penalty: None,
        logit_bias: None,
        user: None,
    }
}

pub fn functions() -> Vec<Function> {
    vec![
        Function {
            name: "done".into(),
            description: Some("This is the final step, and signals that you have enough information to respond to the user's query.".into()),
            parameters: Some(FunctionParameters {
                schema_type: JSONSchemaType::Object,
                properties: Some(HashMap::new()),
                required: None,
            }.into()),
        },
        Function {
            name: "search_codebase".into(),
            description: Some("Search the contents of files in a repository semantically. Results will not necessarily match search terms exactly, but should be related.".into()),
            parameters: Some(FunctionParameters {
                schema_type: JSONSchemaType::Object,
                properties: Some(HashMap::from([
                    ("query".into(), Box::new(JSONSchemaDefine {
                        schema_type: Some(JSONSchemaType::String),
                        description: Some("The query with which to search. This should consist of keywords that might match something in the repository, e.g. 'project dependencies'".to_string()),
                        enum_values: None,
                        properties: None,
                        required: None,
                        items: None,
                    }))
                ])),
                required: Some(vec!["query".into()]),
            }.into())
        },
        Function {
            name: "search_path".into(),
            description: Some("Search the pathnames in a repository. Results may not be exact matches, but will be similar by some edit-distance. Use when you want to find a specific file".into()),
            parameters: Some(FunctionParameters {
                schema_type: JSONSchemaType::Object,
                properties: Some(HashMap::from([
                    ("query".into(), Box::new(JSONSchemaDefine {
                        schema_type: Some(JSONSchemaType::String),
                        description: Some("The query with which to search. This should consist of keywords that might match a file path, e.g. 'src/components/Footer'.".to_string()),
                        enum_values: None,
                        properties: None,
                        required: None,
                        items: None,
                    }))
                ])),
                required: Some(vec!["query".into()]),
            }.into())
        },
        Function {
            name: "search_file".into(),
            description: Some("Search a file semantically. Results will not necessarily match search terms exactly, but should be related.".into()),
            parameters: Some(FunctionParameters {
                schema_type: JSONSchemaType::Object,
                properties: Some(HashMap::from([
                    ("query".into(), Box::new(JSONSchemaDefine {
                        schema_type: Some(JSONSchemaType::String),
                        description: Some("The query with which to search the file.".to_string()),
                        enum_values: None,
                        properties: None,
                        required: None,
                        items: None,
                    })),
                    ("path".into(), Box::new(JSONSchemaDefine {
                        schema_type: Some(JSONSchemaType::Number),
                        description: Some("The file path to search".to_string()),
                        enum_values: None,
                        properties: None,
                        required: None,
                        items: None,
                    }))
                ])),
                required: Some(vec!["query".into(), "path".into()]),
            }.into())
        }
    ]
}

pub fn system_message() -> String {
    let mut s = String::new();
    s.push_str(
        r#"Your job is to choose a function that will help you answer a query about a repository
Follow these rules at all times:
- If the output of a function is empty, try the same function again with different arguments or try using a different function
- If there have been 5 function calls, respond with functions.none
- In most cases respond with functions.search_codebase or functions.search_path functions before responding with functions.none
- Do not assume the structure of the codebase, or the existence of files or folders
- Do NOT respond with a function that you've used before with the same arguments
- When you have enough information to answer the  user's query respond with functions.none
- If after making a path search the query can be answered by the existance of the paths, use the functions.none function
- Only refer to paths that are returned by the functions.search_path function
- Respond with functions to find information related to the query, until all relevant information has been found.
- If after attempting to gather information you are still unsure how to answer the query, respond with the functions.none function
- Always respond with a function call. Do NOT answer the question directly"#
);
    s
}
