use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{StreamExt, SinkExt};
use bincode;
use std::sync::Arc;
use tokio::net::TcpListener;
use chrono;

use crate::config::ServerConfig;
use crate::game::{SessionManager, GameWorld};
use crate::protocol::{ClientMessage, ServerMessage, ChatChannel, Transform};

pub struct GameServer {
    pub config: ServerConfig,
    session_manager: Arc<SessionManager>,
    game_world: Arc<GameWorld>,
}

impl GameServer {
    pub fn new(config: ServerConfig) -> Self {
        Self {
            config,
            session_manager: Arc::new(SessionManager::new()),
            game_world: Arc::new(GameWorld::new()),
        }
    }
    
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = format!("{}:{}", self.config.server.host, self.config.server.port);
        let listener = TcpListener::bind(&addr).await?;
        
        println!("🚀 GameServer started on {}", addr);
        println!("🌐 WebSocket server listening on ws://{}", addr);
        
        while let Ok((stream, _)) = listener.accept().await {
            let peer_addr = stream.peer_addr().unwrap();
            println!("New connection from: {}", peer_addr);
            
            let session_manager = self.session_manager.clone();
            let game_world = self.game_world.clone();
            
            tokio::spawn(async move {
                match accept_async(stream).await {
                    Ok(ws_stream) => {
                        println!("WebSocket connection established from: {}", peer_addr);
                        handle_connection(ws_stream, session_manager, game_world).await;
                    }
                    Err(e) => {
                        eprintln!("Failed to establish WebSocket connection from {}: {}", peer_addr, e);
                    }
                }
            });
        }
        
        Ok(())
    }
}

async fn handle_connection(
    ws_stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    session_manager: std::sync::Arc<SessionManager>,
    game_world: std::sync::Arc<GameWorld>,
) {
    use uuid::Uuid;
    
    println!("🆕 NEW CONNECTION - handle_connection started");
    
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let (message_tx, mut message_rx) = tokio::sync::mpsc::unbounded_channel::<ServerMessage>();
    let (serialized_tx, mut serialized_rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();
    
    let mut current_player_id: Option<Uuid> = None;
    
    // ✅ ОДНА задача отправки - обрабатывает оба типа сообщений
    let send_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                Some(message) = message_rx.recv() => {
                    if let Ok(serialized) = bincode::serialize(&message) {
                        if let Err(e) = ws_sender.send(Message::Binary(serialized.into())).await {
                            eprintln!("Failed to send message: {}", e);
                            break;
                        } 
                    }
                }
                Some(data) = serialized_rx.recv() => {
                    if let Err(e) = ws_sender.send(Message::Binary(data.into())).await {
                        eprintln!("Failed to send serialized data: {}", e);
                        break;
                    } 
                }
                else => break,
            }
        }
    });
    
    // Обработка входящих сообщений
    while let Some(message) = ws_receiver.next().await {
        match message {
            Ok(Message::Binary(data)) => {               
                if let Ok(client_message) = bincode::deserialize::<ClientMessage>(&data) {
                    match client_message {
                        ClientMessage::Login { username, auth_token } => {
                            // ВАЛИДАЦИЯ
                            if username.len() > 32 || username.is_empty() || !username.is_ascii() {
                                let response = ServerMessage::LoginError { 
                                    reason: "Invalid username".to_string() 
                                };
                                message_tx.send(response).ok();
                                continue;
                            }
                            
                            println!("[{}] 🔐 Login attempt: {}",
                                chrono::Local::now().format("%H:%M:%S"),
                                username
                            );
                            
                            let player_id = Uuid::new_v4();
                            current_player_id = Some(player_id);
                            
                            // Добавляем игрока в мир
                            let initial_transform = Transform::default();
                            game_world.add_player(player_id, username.clone(), initial_transform.clone()).await;
                            
                            // СОЗДАЕМ СЕССИЮ с каналом для сериализованных данных
                            let session = crate::game::GameSession::new(
                                player_id, 
                                username.clone(), 
                                serialized_tx.clone(),
                            );
                            
                            if let Err(e) = session_manager.add_session(session).await {
                                eprintln!("Failed to create session: {}", e);
                                continue;
                            }
                            
                            let response = ServerMessage::LoginSuccess { 
                                player_id,
                                username: username.clone(),
                            };
                            
                            if let Err(e) = message_tx.send(response) {
                                eprintln!("Failed to send login response: {}", e);
                            }
                            
                            println!("[{}] ✅ Player {} logged in (ID: {})",
                                chrono::Local::now().format("%H:%M:%S"),
                                username, player_id
                            );
                        }
                        ClientMessage::PlayerMove { transform, velocity, timestamp } => {
                            if let Some(player_id) = current_player_id {
                                if let Some(_) = game_world.update_player_position(player_id, transform.clone()).await {
                                    let update_message = ServerMessage::PlayerTransformUpdate {
                                        player_id,
                                        transform: transform.clone(),
                                        velocity,
                                    };
                                    session_manager.broadcast_except(&player_id, &update_message).await;
                                }
                            }
                        }
                        ClientMessage::ChatMessage { channel, message, target_id } => {
                            
                            if let Some(player_id) = current_player_id {
                                println!("[{}] 💬 {:?} chat from {}: {} (target: {:?})",
                                    chrono::Local::now().format("%H:%M:%S"),
                                    channel, player_id, message, target_id
                                );
                                
                                if let Some(player_state) = game_world.get_player_state(&player_id).await {
                                    let username = player_state.username.clone();
                                    
                                    let chat_message = ServerMessage::ChatMessage {
                                        from_player: username.clone(),
                                        channel: channel.clone(),
                                        message: message.clone(),
                                    };
                                    
                                    println!("📨 Created ServerMessage: {:?}", chat_message);
                                    
                                    match channel {
                                        ChatChannel::Global | ChatChannel::Local | ChatChannel::Party | ChatChannel::Guild => {
                                            session_manager.broadcast_except(&player_id, &chat_message).await;
                                        }
                                        ChatChannel::Whisper => {
                                            if let Some(target_id) = target_id {
                                                println!("🤫 Sending whisper to {}", target_id);
                                                if session_manager.send_to_player(&target_id, chat_message.clone()).await.is_ok() {
                                                    message_tx.send(chat_message.clone()).ok();
                                                } else {
                                                    println!("❌ Whisper target not found");
                                                    let error_msg = ServerMessage::ChatError {
                                                        reason: "Player not found".to_string()
                                                    };
                                                    message_tx.send(error_msg).ok();
                                                }
                                            } else {
                                                let error_msg = ServerMessage::ChatError {
                                                    reason: "Whisper requires target player ID".to_string()
                                                };
                                                message_tx.send(error_msg).ok();
                                            }
                                        }
                                    }
                                } else {
                                    println!("❌ Player state not found for {}", player_id);
                                }
                            } else {
                                println!("❌ No player_id for chat message");
                            }
                        }
                        
                        _ => {
                            println!("[{}] ❓ Unhandled message type: {:?}",
                                chrono::Local::now().format("%H:%M:%S"),
                                client_message
                            );
                        }
                    }
                } else {
                    println!("❌ DESERIALIZE FAILED");
                    println!("🔍 Full message hex: {:02x?}", &data);
                    
                    if data.len() >= 4 {
                        let variant = data[0];
                        println!("📊 Enum variant: {}", variant);
                        
                        // 🔥 РУЧНОЙ ПАРСИНГ ДЛЯ ChatMessage
                        if variant == 1 {
                            println!("🔄 Manually parsing as ChatMessage...");
                            
                            let mut offset = 4; // Пропускаем variant (4 байта для u32)
                            
                            // Парсим channel (u32)
                            if offset + 4 <= data.len() {
                                let channel_bytes = &data[offset..offset+4];
                                let channel_val = u32::from_le_bytes([channel_bytes[0], channel_bytes[1], channel_bytes[2], channel_bytes[3]]);
                                offset += 4;
                                println!("📊 Channel value: {}", channel_val);
                                
                                // Парсим длину message (u64)
                                if offset + 8 <= data.len() {
                                    let len_bytes = &data[offset..offset+8];
                                    let msg_len = u64::from_le_bytes([
                                        len_bytes[0], len_bytes[1], len_bytes[2], len_bytes[3],
                                        len_bytes[4], len_bytes[5], len_bytes[6], len_bytes[7]
                                    ]);
                                    offset += 8;
                                    println!("📊 Message length: {}", msg_len);
                                    
                                    // Парсим message (String)
                                    if offset + msg_len as usize <= data.len() {
                                        let message_bytes = &data[offset..offset + msg_len as usize];
                                        let message = String::from_utf8_lossy(message_bytes).to_string();
                                        offset += msg_len as usize;
                                        println!("📊 Message: '{}'", message);
                                        
                                        // Парсим target_id (Option<Uuid>)
                                        let target_id = if offset < data.len() {
                                            // Option serialization: 0 for None, 1 for Some
                                            if data[offset] == 0 {
                                                offset += 1;
                                                None
                                            } else if data[offset] == 1 && offset + 16 <= data.len() {
                                                offset += 1;
                                                match Uuid::from_slice(&data[offset..offset+16]) {
                                                    Ok(uuid) => {
                                                        println!("📊 Target ID: {}", uuid);
                                                        Some(uuid)
                                                    }
                                                    Err(_) => {
                                                        println!("❌ Invalid UUID format");
                                                        None
                                                    }
                                                }
                                            } else {
                                                println!("❌ Invalid target_id format");
                                                None
                                            }
                                        } else {
                                            None
                                        };
                                        
                                        println!("🎯 MANUALLY PARSED ChatMessage: channel={}, message='{}', target_id={:?}", 
                                            channel_val, message, target_id);
                                        
                                        // 🔥 ОБРАБОТКА КАК ОБЫЧНОГО СООБЩЕНИЯ
                                        if let Some(player_id) = current_player_id {
                                            if let Some(player_state) = game_world.get_player_state(&player_id).await {
                                                let channel = match channel_val {
                                                    0 => ChatChannel::Global,
                                                    1 => ChatChannel::Local,
                                                    2 => ChatChannel::Party,
                                                    3 => ChatChannel::Guild,
                                                    4 => ChatChannel::Whisper,
                                                    _ => ChatChannel::Local,
                                                };
                                                
                                                let chat_message = ServerMessage::ChatMessage {
                                                    from_player: player_state.username.clone(),
                                                    channel: channel.clone(),
                                                    message: message.clone(),
                                                };
                                                
                                                println!("📨 Created ServerMessage: {:?}", chat_message);
                                                
                                                match channel {
                                                    ChatChannel::Global | ChatChannel::Local | ChatChannel::Party | ChatChannel::Guild => {
                                                        println!("📢 Broadcasting to all except {}", player_id);
                                                        session_manager.broadcast_except(&player_id, &chat_message).await;
                                                        println!("✅ Broadcast completed");
                                                    }
                                                    ChatChannel::Whisper => {
                                                        if let Some(target_id) = target_id {
                                                            println!("🤫 Sending whisper to {}", target_id);
                                                            if session_manager.send_to_player(&target_id, chat_message.clone()).await.is_ok() {
                                                                // Также отправляем копию отправителю
                                                                message_tx.send(chat_message.clone()).ok();
                                                                println!("✅ Whisper sent successfully");
                                                            } else {
                                                                println!("❌ Whisper target not found: {}", target_id);
                                                                let error_msg = ServerMessage::ChatError {
                                                                    reason: format!("Player not found: {}", target_id)
                                                                };
                                                                message_tx.send(error_msg).ok();
                                                            }
                                                        } else {
                                                            println!("❌ No target_id for whisper");
                                                            let error_msg = ServerMessage::ChatError {
                                                                reason: "Whisper requires target player ID".to_string()
                                                            };
                                                            message_tx.send(error_msg).ok();
                                                        }
                                                    }
                                                }
                                            } else {
                                                println!("❌ Player state not found for {}", player_id);
                                            }
                                        } else {
                                            println!("❌ No player_id for chat message");
                                        }
                                    } else {
                                        println!("❌ Invalid message length: {} > {}", msg_len, data.len() - offset);
                                    }
                                } else {
                                    println!("❌ Not enough bytes for message length");
                                }
                            } else {
                                println!("❌ Not enough bytes for channel");
                            }
                        }
                    }
                }
                println!("=== END SERVER DEBUG ===\n");
            }
            Ok(Message::Close(_)) => {
                println!("[DEBUG SERVER] 🔌 Close frame received");
                break;
            }
            Err(e) => {
                println!("[DEBUG SERVER] ❌ WebSocket error: {}", e);
                break;
            }
            _ => {
                println!("[DEBUG SERVER] 📨 Other WebSocket message type");
            }
        }
    }
    
    if let Some(player_id) = current_player_id {
        session_manager.remove_session(&player_id).await;
        game_world.remove_player(&player_id).await;
        
        println!("[{}] 🚪 Player {} disconnected",
            chrono::Local::now().format("%H:%M:%S"),
            player_id
        );
    }
    
    let _ = send_task.await;
    println!("🔚 CONNECTION ENDED - handle_connection finished");
}

// Вспомогательная функция для форматирования канала
fn format_channel(channel: &ChatChannel) -> String {
    match channel {
        ChatChannel::Global => "🌍 Global",
        ChatChannel::Local => "📍 Local", 
        ChatChannel::Party => "👥 Party",
        ChatChannel::Guild => "⚔️ Guild",
        ChatChannel::Whisper => "🤫 Whisper",
    }.to_string()
}