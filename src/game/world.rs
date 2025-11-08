use crate::protocol::{PlayerId, Transform};
use std::collections::HashMap;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct PlayerState {
    pub username: String,
    pub transform: Transform,
    pub zone_id: u32, // Простая система зон
}

#[derive(Debug)]
pub struct GameWorld {
    players: RwLock<HashMap<PlayerId, PlayerState>>,
    next_zone_id: u32,
}

impl GameWorld {
    pub fn new() -> Self {
        Self {
            players: RwLock::new(HashMap::new()),
            next_zone_id: 1,
        }
    }
    
    // Добавляем игрока в мир
    pub async fn add_player(&self, player_id: PlayerId, username: String, transform: Transform) {
        let mut players = self.players.write().await;
        
        let player_state = PlayerState {
            username,
            transform,
            zone_id: self.next_zone_id, // Пока все в одной зоне
        };
        
        players.insert(player_id, player_state);
        println!("Player {} added to world (zone: {})", player_id, self.next_zone_id);
    }
    
    // Обновляем позицию игрока
    pub async fn update_player_position(&self, player_id: PlayerId, transform: Transform) -> Option<Transform> {
        let mut players = self.players.write().await;
        
        if let Some(player_state) = players.get_mut(&player_id) {
            let old_transform = player_state.transform.clone();
            player_state.transform = transform;
            Some(old_transform)
        } else {
            None
        }
    }
    
    // Получаем состояние игрока
    pub async fn get_player_state(&self, player_id: &PlayerId) -> Option<PlayerState> {
        let players = self.players.read().await;
        players.get(player_id).cloned()
    }
    
    // Получаем всех игроков в зоне (пока все в одной зоне)
    pub async fn get_players_in_zone(&self, zone_id: u32) -> Vec<(PlayerId, PlayerState)> {
        let players = self.players.read().await;
        players
            .iter()
            .filter(|(_, state)| state.zone_id == zone_id)
            .map(|(id, state)| (*id, state.clone())) // Используем clone() для PlayerState
            .collect()
    }
    
    // Удаляем игрока из мира
    pub async fn remove_player(&self, player_id: &PlayerId) -> Option<PlayerState> {
        let mut players = self.players.write().await;
        let removed = players.remove(player_id);
        
        if removed.is_some() {
            println!("Player {} removed from world", player_id);
        }
        
        removed
    }
    
    // Получаем количество игроков в мире
    pub async fn player_count(&self) -> usize {
        let players = self.players.read().await;
        players.len()
    }
}

impl Default for GameWorld {
    fn default() -> Self {
        Self::new()
    }
}