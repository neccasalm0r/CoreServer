use super::{Vector3, Transform};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::common::ChatChannel;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
      // Аутентификация
    LoginSuccess {
        player_id: Uuid,
        username: String,
    },
    
    LoginError {
        reason: String,
    },
    
    // Движение
    PlayerUpdate {
        player_id: Uuid,
        transform: Transform,
    },
    
    // Обновления мира
    WorldState {
        players: Vec<PlayerUpdate>,
        npcs: Vec<NpcUpdate>,
        objects: Vec<ObjectUpdate>,
        timestamp: u64,
    },
    
    // События игроков
    PlayerJoined {
        player_data: PlayerData,
    },
    
    PlayerLeft {
        player_id: Uuid,
    },
    
    PlayerTransformUpdate {
        player_id: Uuid,
        transform: Transform,
        velocity: Vector3,
    },
    // Чат
    ChatMessage {
        channel: ChatChannel,
        message: String,
        from_player: String,  // ✅ Оставляем как есть
    },
    
    ChatError {
        reason: String,
    },
    
    // Combat
    CombatEvent {
        source_id: Uuid,
        target_id: Uuid,
        damage: i32,
        ability_id: u32,
    },
    
    // Heartbeat response
    HeartbeatResponse {
        server_time: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerData {
    pub id: Uuid,
    pub name: String,
    pub level: u32,
    pub class: PlayerClass,
    pub transform: Transform,
    pub stats: PlayerStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerUpdate {
    pub player_id: Uuid,
    pub transform: Transform,
    pub velocity: Vector3,
    pub animation: Option<String>,
    pub health: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcUpdate {
    pub npc_id: Uuid,
    pub transform: Transform,
    pub health: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectUpdate {
    pub object_id: Uuid,
    pub transform: Transform,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlayerClass {
    Warrior,
    Mage,
    Archer,
    Rogue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerStats {
    pub health: i32,
    pub max_health: i32,
    pub mana: i32,
    pub max_mana: i32,
    pub strength: i32,
    pub agility: i32,
    pub intelligence: i32,
}
