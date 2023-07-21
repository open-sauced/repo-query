use actix_web_lab::sse::{Data, SendError, Sender};

pub async fn emit(sender: &Sender, event: Data) -> Result<(), SendError> {
    sender.send(event).await?;
    //Empty message to force send the above message to receiver
    //Else, the above message will stay in the buffer
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
            $($key(String)),*
        }

        impl From<$name> for Data {
            fn from(event: $name) -> Data {
                match event {
                    $(
                        $name::$key(data) => Data::new(data).event($value)
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
}

sse_events! {
    QueryEvent,
    (SearchCodebase, "SEARCH_CODEBASE"),
    (SearchFile, "SEARCH_FILE"),
    (SearchPath, "SEARCH_PATH"),
    (GenerateResponse, "GENERATE_RESPONSE"),
    (Done, "DONE"),
}
