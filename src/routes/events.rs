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
            $($key),*
        }

        impl Into<Data> for $name {
            fn into(self) -> Data {
                match self {
                    $(
                        $name::$key => Data::new("").event($value)
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
