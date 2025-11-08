use std::collections::HashMap;
use tokio::sync::RwLock;
use crate::protocol::ServerMessage;
use super::session::{GameSession, PlayerId};
use bincode;

#[derive(Debug)]
pub struct SessionManager {
    sessions: RwLock<HashMap<PlayerId, GameSession>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
        }
    }
    
    pub async fn add_session(&self, session: GameSession) -> Result<(), String> {
        let mut sessions = self.sessions.write().await;
        
        if sessions.contains_key(&session.player_id) {
            return Err(format!("Session already exists for player_id: {}", session.player_id));
        }
        
        println!("New session created for player: {} ({})", session.username, session.player_id);
        sessions.insert(session.player_id, session);
        
        Ok(())
    }
    
    pub async fn remove_session(&self, player_id: &PlayerId) -> Option<GameSession> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(player_id)
    }
    
    pub async fn get_session(&self, player_id: &PlayerId) -> Option<GameSession> {
        let sessions = self.sessions.read().await;
        sessions.get(player_id).cloned()
    }
    
    // ✅ ПРАВИЛЬНАЯ рассылка - сериализует один раз и отправляет всем
    pub async fn broadcast(&self, message: &ServerMessage) {
        let sessions = self.sessions.read().await;
        
        // СЕРИАЛИЗУЕМ СООБЩЕНИЕ ОДИН РАЗ
        let serialized_data = match bincode::serialize(message) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Failed to serialize message for broadcast: {}", e);
                return;
            }
        };
        
        let mut sent_count = 0;
        for session in sessions.values() {
            if session.send_serialized(serialized_data.clone()).is_ok() {
                sent_count += 1;
            }
        }
        
        if sent_count > 0 {
            println!("📢 Broadcasted to {} players", sent_count);
        }
    }
    
    // ✅ ПРАВИЛЬНАЯ рассылка кроме указанного игрока
    pub async fn broadcast_except(&self, exclude_player_id: &PlayerId, message: &ServerMessage) {
        let sessions = self.sessions.read().await;
        // СЕРИАЛИЗУЕМ СООБЩЕНИЕ ОДИН РАЗ
        let serialized_data = match bincode::serialize(message) {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Failed to serialize message for broadcast_except: {}", e);
                return;
            }
        };
        
        let mut sent_count = 0;
        for (player_id, session) in sessions.iter() {
            if player_id != exclude_player_id {
                if session.send_serialized(serialized_data.clone()).is_ok() {
                    sent_count += 1;
                } 
            }
        }
        if sent_count > 0 {
            println!("📢 Broadcasted to {} players (except {})", sent_count, exclude_player_id);
        }
    }
    
    // ✅ ПРАВИЛЬНАЯ отправка конкретному игроку
    pub async fn send_to_player(&self, player_id: &PlayerId, message: ServerMessage) -> Result<(), String> {
    
    let sessions = self.sessions.read().await;
    
    if let Some(session) = sessions.get(player_id) {
        let serialized_data = bincode::serialize(&message)
            .map_err(|e| format!("Failed to serialize message: {}", e))?;
            
        session.send_serialized(serialized_data)
            .map_err(|e| format!("Failed to send message to player {}: {}", player_id, e))
    } else {
        Err(format!("Player not found: {}", player_id))
    }
}
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}