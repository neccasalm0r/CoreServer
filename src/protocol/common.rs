use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChatChannel {
    Global,
    Local,
    Party,
    Guild,
    Whisper,
}