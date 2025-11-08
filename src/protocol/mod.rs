use serde::{Deserialize, Serialize};
pub mod common;
// Подмодули
pub mod client_messages;
pub mod server_messages;
pub type PlayerId = uuid::Uuid;

pub use common::*;
pub use client_messages::*;
pub use server_messages::*;
// Re-export everything from child modules
pub use client_messages::*;
pub use server_messages::*;



// Базовые типы данных, совместимые с UE5
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transform {
    pub position: Vector3,
    pub rotation: Quaternion,
    pub scale: Vector3,
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

