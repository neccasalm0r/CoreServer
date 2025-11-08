use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use bincode;
use serde::{Serialize, Deserialize};
use bytes::Bytes;
use uuid::Uuid;

#[derive(Serialize)]
enum ClientMessage {
    Login { username: String, auth_token: String },
    PlayerMove { transform: Transform, velocity: Vector3, timestamp: u64 },
}

#[derive(Serialize, Deserialize, Debug)]
enum ServerMessage {
    LoginSuccess { player_id: Uuid, username: String },
    LoginError { reason: String },
    PlayerUpdate { player_id: Uuid, transform: Transform },
    LoginResponse { success: bool, player_data: Option<PlayerData>, error: Option<String> },
    WorldState { players: Vec<PlayerUpdate>, npcs: Vec<NpcUpdate>, objects: Vec<ObjectUpdate>, timestamp: u64 },
    PlayerJoined { player_data: PlayerData },
    PlayerLeft { player_id: Uuid },
    ChatMessage { from_player: String, channel: ChatChannel, message: String },
    CombatEvent { source_id: Uuid, target_id: Uuid, damage: i32, ability_id: u32 },
    HeartbeatResponse { server_time: u64 },
}

#[derive(Serialize, Deserialize, Debug)]
struct Transform {
    position: Vector3,
    rotation: Quaternion,
    scale: Vector3,
}

#[derive(Serialize, Deserialize, Debug)]
struct Vector3 {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Quaternion {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}

#[derive(Serialize, Deserialize, Debug)]
struct PlayerData {
    id: Uuid,
    name: String,
    level: u32,
    class: PlayerClass,
    transform: Transform,
    stats: PlayerStats,
}

#[derive(Serialize, Deserialize, Debug)]
struct PlayerUpdate {
    player_id: Uuid,
    transform: Transform,
    velocity: Vector3,
    animation: Option<String>,
    health: i32,
}

#[derive(Serialize, Deserialize, Debug)]
struct NpcUpdate {
    npc_id: Uuid,
    transform: Transform,
    health: i32,
}

#[derive(Serialize, Deserialize, Debug)]
struct ObjectUpdate {
    object_id: Uuid,
    transform: Transform,
    state: String,
}

#[derive(Serialize, Deserialize, Debug)]
enum PlayerClass {
    Warrior,
    Mage,
    Archer,
    Rogue,
}

#[derive(Serialize, Deserialize, Debug)]
struct PlayerStats {
    health: i32,
    max_health: i32,
    mana: i32,
    max_mana: i32,
    strength: i32,
    agility: i32,
    intelligence: i32,
}

#[derive(Serialize, Deserialize, Debug)]
enum ChatChannel {
    Global,
    Local,
    Party,
    Guild,
    Whisper,
}

#[tokio::main]
async fn main() {
    println!("🚀 Connecting to ws://127.0.0.1:8080...");
    
    match connect_async("ws://127.0.0.1:8080").await {
        Ok((ws_stream, _)) => {
            println!("✅ Connected successfully!");
            let (mut write, mut read) = ws_stream.split();
            
            // Отправляем логин
            let login_msg = ClientMessage::Login {
                username: "rust_client".to_string(),
                auth_token: "test_token".to_string(),
            };
            
            if let Ok(encoded) = bincode::serialize(&login_msg) {
                println!("📤 Sending login message...");
                write.send(Message::Binary(encoded.into())).await.unwrap();
            }
            
            // Ждем ответ на логин и затем отправляем движение
            let mut logged_in = false;
            
            while let Some(message) = read.next().await {
                match message {
                    Ok(Message::Binary(data)) => {
                        println!("📦 Received binary data: {} bytes", data.len());
                        
                        // ПРОБУЕМ РАЗНЫЕ ВАРИАНТЫ ДЕСЕРИАЛИЗАЦИИ
                        match bincode::deserialize::<ServerMessage>(&data) {
                            Ok(server_msg) => {
                                println!("🎯 Successfully decoded: {:?}", server_msg);
                                
                                match server_msg {
                                    ServerMessage::LoginSuccess { player_id, username } => {
                                        println!("✨ Login successful! Player: {} (ID: {})", username, player_id);
                                        logged_in = true;
                                        
                                        // После успешного логина отправляем движение
                                        println!("🎮 Sending movement...");
                                        let move_msg = ClientMessage::PlayerMove {
                                            transform: Transform {
                                                position: Vector3 { x: 10.0, y: 0.0, z: 5.0 },
                                                rotation: Quaternion { x: 0.0, y: 0.0, z: 0.0, w: 1.0 },
                                                scale: Vector3 { x: 1.0, y: 1.0, z: 1.0 },
                                            },
                                            velocity: Vector3 { x: 1.0, y: 0.0, z: 0.0 },
                                            timestamp: std::time::SystemTime::now()
                                                .duration_since(std::time::UNIX_EPOCH)
                                                .unwrap()
                                                .as_millis() as u64,
                                        };
                                        
                                        if let Ok(encoded) = bincode::serialize(&move_msg) {
                                            write.send(Message::Binary(encoded.into())).await.unwrap();
                                            println!("📤 Movement message sent!");
                                        }
                                    }
                                    ServerMessage::PlayerUpdate { player_id, transform } => {
                                        println!("🎮 Player position update: {} at ({:.1}, {:.1}, {:.1})", 
                                            player_id, transform.position.x, transform.position.y, transform.position.z);
                                    }
                                    ServerMessage::LoginError { reason } => {
                                        println!("❌ Login failed: {}", reason);
                                    }
                                    _ => println!("📨 Other message: {:?}", server_msg),
                                }
                            }
                            Err(e) => {
                                println!("❌ Failed to decode ServerMessage: {}", e);
                                // Выводим сырые байты для отладки
                                println!("🔍 Raw bytes (first 20): {:?}", &data[..std::cmp::min(20, data.len())]);
                            }
                        }
                    }
                    Ok(Message::Text(text)) => {
                        println!("📨 Received text: {}", text);
                    }
                    Ok(Message::Close(_)) => {
                        println!("🔌 Connection closed");
                        break;
                    }
                    Err(e) => {
                        println!("❌ Error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        }
        Err(e) => println!("❌ Failed to connect: {}", e),
    }
}