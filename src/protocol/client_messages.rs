use super::{Vector3, Transform};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::common::ChatChannel;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    // Аутентификация
    Login {
        username: String,
        auth_token: String,
    },
    
    // Движение
    PlayerMove {
        transform: Transform,
        velocity: Vector3,
        timestamp: u64,
    },
    
    // Действия
    PlayerAction {
        action_type: PlayerAction,
        target_id: Option<Uuid>,
        direction: Option<Vector3>,
    },
    
    ChatMessage {
        channel: ChatChannel,
        message: String,
        target_id: Option<Uuid>,
    },
    
    // Инвентарь
    UseItem {
        item_id: Uuid,
        target_id: Option<Uuid>,
    },
    
    // Combat
    Attack {
        target_id: Uuid,
        ability_id: u32,
    },
    
    // Keep-alive
    Heartbeat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlayerAction {
    Jump,
    Interact,
    Sit,
    Dance,
    Craft,
}
