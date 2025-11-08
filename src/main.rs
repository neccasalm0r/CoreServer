mod protocol;
mod network;
mod game;
mod config;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting MMORPG Server...");
    
    let config = config::ServerConfig::load_config()?;
    let mut server = network::GameServer::new(config);
    server.start().await?;
    
    Ok(())
}