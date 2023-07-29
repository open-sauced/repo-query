use actix_web_lab::sse::{Data, SendError, Sender};

use crate::sse_events;

pub async fn emit<T: Into<Data>>(sender: &Sender, event: T) -> Result<(), SendError> {
    sender.send(event.into()).await?;
    //Empty message to force send the above message to receiver
    //Else, will stay in the buffer when using actix_rt::spawn
    //TODO: Investigate further to avoid this workaround
    sender.send(Data::new("")).await?;
    Ok(())
}

sse_events! {
    EmbedEvent,
    (FetchRepo, "FETCH_REPO"),
    (EmbedRepo, "EMBED_REPO"),
    (SaveEmbeddings, "SAVE_EMBEDDINGS"),
    (Done, "DONE"),
    (Error, "ERROR"),
}

sse_events! {
    QueryEvent,
    (ProcessQuery, "PROCESS_QUERY"),
    (SearchCodebase, "SEARCH_CODEBASE"),
    (SearchFile, "SEARCH_FILE"),
    (SearchPath, "SEARCH_PATH"),
    (GenerateResponse, "GENERATE_RESPONSE"),
    (Done, "DONE"),
    (Error, "ERROR"),
}
