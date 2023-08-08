use std::ops::RangeInclusive;

//Actix-web
pub const SSE_CHANNEL_BUFFER_SIZE: usize = 1;
pub const HOME_ROUTE_REDIRECT_URL: &str = "https://opensauced.pizza";

//OpenAI
pub const CHAT_COMPLETION_TEMPERATURE: f64 = 0.7;
pub const CHAT_COMPLETION_MODEL: &str = "gpt-3.5-turbo";

//Semantic search
pub const MAX_FILES_COUNT: usize = 1000;
pub const FILE_CHUNKER_CAPACITY_RANGE: RangeInclusive<usize> = 300..=400;
pub const RELEVANT_FILES_LIMIT: usize = 3;
pub const RELEVANT_CHUNKS_LIMIT: usize = 2;
