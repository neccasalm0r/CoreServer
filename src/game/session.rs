use tokio::sync::mpsc;
use uuid::Uuid;

pub type PlayerId = Uuid;

#[derive(Debug, Clone)]
pub struct GameSession {
    pub player_id: PlayerId,
    pub username: String,
    pub serialized_tx: mpsc::UnboundedSender<Vec<u8>>, // ✅ Канал для сериализованных данных
    pub udp_addr: Option<std::net::SocketAddr>,
}

impl GameSession {
    pub fn new(
        player_id: PlayerId, 
        username: String, 
        serialized_tx: mpsc::UnboundedSender<Vec<u8>>,
    ) -> Self {
        Self {
            player_id,
            username,
            serialized_tx,
            udp_addr: None,
        }
    }
    
    // ✅ ОСНОВНОЙ метод - отправляет уже сериализованные данные
    pub fn send_serialized(&self, data: Vec<u8>) -> Result<(), mpsc::error::SendError<Vec<u8>>> {
        let result = self.serialized_tx.send(data);
        result
    }
    
    pub fn set_udp_addr(&mut self, addr: std::net::SocketAddr) {
        self.udp_addr = Some(addr);
    }
    
    pub fn get_udp_addr(&self) -> Option<std::net::SocketAddr> {
        self.udp_addr
    }
}