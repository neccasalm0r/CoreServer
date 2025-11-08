use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use bincode;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use tokio::io::{self, AsyncBufReadExt};
use rand::Rng;

#[derive(Serialize, Debug)]
enum ClientMessage {
    Login { 
        username: String, 
        auth_token: String 
    },
    ChatMessage { 
        channel: ChatChannel, 
        message: String, 
        target_id: Option<Uuid> 
    },
}

#[derive(Serialize, Deserialize, Debug)]
enum ServerMessage {
    LoginSuccess { 
        player_id: Uuid, 
        username: String 
    },
    LoginError { 
        reason: String 
    },
    PlayerUpdate { 
        player_id: Uuid, 
        transform: Transform 
    },
    // 🔥 ДОБАВЬТЕ ЭТИ СТРУКТУРЫ:
    WorldState {
        players: Vec<PlayerUpdate>,
        npcs: Vec<NpcUpdate>,
        objects: Vec<ObjectUpdate>,
        timestamp: u64,
    },
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
    // 🔥 ЭТО ОСНОВНАЯ ПРОБЛЕМА - ChatMessage должен быть ПОСЛЕДНИМ из известных вариантов
    ChatMessage {
        channel: ChatChannel,
        message: String,
        from_player: String,
    },
    ChatError {
        reason: String,
    },
    CombatEvent {
        source_id: Uuid,
        target_id: Uuid,
        damage: i32,
        ability_id: u32,
    },
    HeartbeatResponse {
        server_time: u64,
    },
    // Обработка неизвестных вариантов
    #[serde(other)]
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum ChatChannel {
    Global,
    Local,
    Party,
    Guild,
    Whisper,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Transform {
    position: Vector3,
    rotation: Quaternion,
    scale: Vector3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
            rotation: Quaternion { x: 0.0, y: 0.0, z: 0.0, w: 1.0 },
            scale: Vector3 { x: 1.0, y: 1.0, z: 1.0 },
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Vector3 {
    x: f32,
    y: f32,
    z: f32,
}

impl Default for Vector3 {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0 }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Quaternion {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}

impl Default for Quaternion {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0, z: 0.0, w: 1.0 }
    }
}

// Вспомогательные структуры для полного соответствия с сервером
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

#[tokio::main]
async fn main() {
    println!("🚀 Advanced Chat Client - Connecting...");
    
    let mut rng = rand::thread_rng();
    let username = format!("user_{}", rng.gen_range(1000..9999));
    
    match connect_async("ws://127.0.0.1:8080").await {
        Ok((mut ws, _)) => {
            println!("✅ Connected successfully!");
            
            // Логин
            let login_msg = ClientMessage::Login {
                username: username.clone(),
                auth_token: "chat_token".to_string(),
            };
            
            if let Ok(encoded) = bincode::serialize(&login_msg) {
                println!("📤 Logging in as {}...", username);
                ws.send(Message::Binary(encoded.into())).await.unwrap();
            }
            
            // Клонируем WebSocket для чтения
            let (mut ws_sender, mut ws_receiver) = ws.split();
            
            // Задача чтения сообщений от сервера
            let read_handle = tokio::spawn(async move {
                // В read_handle ЗАМЕНИТЕ весь сложный блок на простой:
            while let Some(message) = ws_receiver.next().await {
                match message {
                    Ok(Message::Binary(data)) => {
                        match bincode::deserialize::<ServerMessage>(&data) {
                            Ok(server_msg) => {
                                match server_msg {
                                    ServerMessage::ChatMessage { channel, message, from_player } => {
                                        let channel_icon = match channel {
                                            ChatChannel::Global => "🌍",
                                            ChatChannel::Local => "📍",
                                            ChatChannel::Party => "👥", 
                                            ChatChannel::Guild => "⚔️",
                                            ChatChannel::Whisper => "🤫",
                                        };
                                        println!("\n💬 {} [{}]: {}", channel_icon, from_player, message);
                                    }
                                    ServerMessage::LoginSuccess { player_id, username } => {
                                        println!("✨ Welcome, {}! (ID: {})", username, player_id);
                                    }
                                    ServerMessage::LoginError { reason } => {
                                        println!("❌ Login failed: {}", reason);
                                    }
                                    ServerMessage::ChatError { reason } => {
                                        println!("❌ Chat error: {}", reason);
                                    }
                                    _ => {} // Игнорируем другие типы сообщений
                                }
                            }
                            Err(e) => {
                                println!("❌ Failed to decode message: {}", e);
                            }
                        }
                        print!("💬 Your message: ");
                    }
                    Ok(Message::Text(text)) => {
                        println!("📨 Received text: {}", text);
                    }
                    Ok(Message::Close(_)) => {
                        println!("🔌 Connection closed by server");
                        break;
                    }
                    Err(e) => {
                        println!("❌ WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        });
            
            // Чтение stdin и отправка сообщений
            let mut stdin = io::BufReader::new(io::stdin()).lines();
            
            println!("💬 Type your messages (or /help for commands):");
            print!("💬 Your message: ");
            
            while let Ok(Some(line)) = stdin.next_line().await {
                if line.is_empty() {
                    print!("💬 Your message: ");
                    continue;
                }
                
                let chat_msg = if line.starts_with("/g ") {
                    ClientMessage::ChatMessage {
                        channel: ChatChannel::Global,
                        message: line[3..].to_string(),
                        target_id: None,
                    }
                } else if line.starts_with("/l ") {
                    ClientMessage::ChatMessage {
                        channel: ChatChannel::Local,
                        message: line[3..].to_string(),
                        target_id: None,
                    }
                } else if line.starts_with("/w ") {
                    let parts: Vec<&str> = line[3..].splitn(2, ' ').collect();
                    if parts.len() == 2 {
                        if let Ok(target_id) = Uuid::parse_str(parts[0]) {
                            println!("🔍 Sending whisper to: {}", target_id);
                            ClientMessage::ChatMessage {
                                channel: ChatChannel::Whisper,
                                message: parts[1].to_string(),
                                target_id: Some(target_id),
                            }
                        } else {
                            println!("❌ Invalid UUID. Example valid UUID: 905ddeed-4110-4c70-9387-377d8c661697");
                            continue;
                        }
                    } else {
                        println!("❌ Usage: /w <player_id> <message>");
                        continue;
                    }
                } else if line == "/help" {
                    println!("💡 Chat commands:");
                    println!("  /g <message> - Global chat");
                    println!("  /l <message> - Local chat"); 
                    println!("  /w <player_id> <message> - Whisper");
                    println!("  <message> - Local chat (default)");
                    println!("  /help - Show this help");
                    print!("💬 Your message: ");
                    continue;
                } else if line == "/quit" || line == "/exit" {
                    println!("👋 Goodbye!");
                    break;
                } else {
                    ClientMessage::ChatMessage {
                        channel: ChatChannel::Local,
                        message: line,
                        target_id: None,
                    }
                };
                
                if let Ok(encoded) = bincode::serialize(&chat_msg) {
                    if ws_sender.send(Message::Binary(encoded.into())).await.is_err() {
                        println!("❌ Connection lost");
                        break;
                    }
                } else {
                    println!("❌ Failed to serialize message");
                }
                
                print!("💬 Your message: ");
            }
            
            read_handle.await.ok();
        }
        Err(e) => println!("❌ Failed to connect: {}", e),
    }
}

// Минимальная структура только для чата (резервная)
#[derive(Serialize, Deserialize, Debug)]
struct ServerMessageChatOnly {
    channel: ChatChannel,
    message: String,
    from_player: String,
}