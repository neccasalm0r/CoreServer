use tokio::net::UdpSocket;
use std::sync::Arc;

pub struct UdpTransport {
    socket: Arc<UdpSocket>,
}

impl UdpTransport {
    pub async fn new(bind_addr: &str) -> Result<Self, std::io::Error> {
        let socket = UdpSocket::bind(bind_addr).await?;
        println!("🎯 UDP server listening on {}", bind_addr);
        Ok(Self {
            socket: Arc::new(socket),
        })
    }
    
    pub async fn send_to(&self, data: &[u8], addr: std::net::SocketAddr) -> Result<(), std::io::Error> {
        self.socket.send_to(data, addr).await?;
        Ok(())  
    }
    pub async fn broadcast(&self, data: &[u8], addresses: &[std::net::SocketAddr]) {
        for &addr in addresses {
            let _ = self.send_to(data, addr).await;
        }
    }
}