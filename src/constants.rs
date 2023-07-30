use std::ops::RangeInclusive;

//Embeddings
pub const EMBEDDINGS_DIMENSION: usize = 384;

//Actix-web
pub const SSE_CHANNEL_BUFFER_SIZE: usize = 1;
pub const ACTIX_WEB_SERVER_PORT: usize = 3000;

//OpenAI
pub const CHAT_COMPLETION_TEMPERATURE: f64 = 0.7;
pub const CHAT_COMPLETION_MODEL: &str = "gpt-3.5-turbo";

//Semantic search
pub const MAX_FILES_COUNT: usize = 1000;
pub const FILE_CHUNKER_CAPACITY_RANGE: RangeInclusive<usize> = 800..=1000;
pub const RELEVANT_CHUNKS_LIMIT: usize = 3;
