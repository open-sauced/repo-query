use actix_web_lab::sse::{Data, SendError, Sender};

pub async fn emit<T: Into<Data>>(sender: &Sender, event: T) -> Result<(), SendError> {
    sender.send(event.into()).await?;
    //Empty message to force send the above message to receiver
    //Else, will stay in the buffer when using actix_rt::spawn
    //TODO: Investigate further to avoid this workaround
    sender.send(Data::new("")).await?;
    Ok(())
}

//Custom implementation for SSE Events based on https://crates.io/crates/enum_str
macro_rules!  sse_events {
    ($name:ident, $(($key:ident, $value:expr),)*) => {
       #[derive(Debug, PartialEq)]
       pub enum $name
        {
            $($key(Option<serde_json::Value>)),*
        }

        impl From<$name> for Data {
            fn from(event: $name) -> Data {
                match event {
                    $(
                        $name::$key(data) => Data::new(data.unwrap_or_default().to_string()).event($value)
                    ),*
                }
            }
        }

    }
}

sse_events! {
    EmbedEvent,
    (FetchRepo, "FETCH_REPO"),
    (EmbedRepo, "EMBED_REPO"),
    (SaveEmbeddings, "SAVE_EMBEDDINGS"),
    (Done, "DONE"),
}

sse_events! {
    QueryEvent,
    (SearchCodebase, "SEARCH_CODEBASE"),
    (SearchFile, "SEARCH_FILE"),
    (SearchPath, "SEARCH_PATH"),
    (GenerateResponse, "GENERATE_RESPONSE"),
    (Done, "DONE"),
}
