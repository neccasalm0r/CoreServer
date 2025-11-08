use crate::protocol::ServerMessage;
use uuid::Uuid;

pub type PlayerId = Uuid; // Создаем алиас

pub enum TransportMessage {
    Reliable(ServerMessage),
    Unreliable(Vec<u8>), 
}

pub trait GameTransport: Send + Sync {
    async fn send(&self, player_id: &PlayerId, message: TransportMessage); // Используем наш PlayerId
    async fn broadcast(&self, message: TransportMessage);
}