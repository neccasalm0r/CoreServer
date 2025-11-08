use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub server: NetworkConfig,
    pub game: GameConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NetworkConfig {
    pub host: String,
    pub port: u16,
    pub max_players: u32,
    pub tick_rate: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GameConfig {
    pub world: WorldConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WorldConfig {
    pub name: String,
    pub max_players_per_zone: u32,
    pub save_interval: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            server: NetworkConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                max_players: 1000,
                tick_rate: 60,
            },
            game: GameConfig {
                world: WorldConfig {
                    name: "Aethelgard".to_string(),
                    max_players_per_zone: 100,
                    save_interval: 300,
                },
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
            },
        }
    }
}

impl ServerConfig {
    pub fn load_config() -> Result<Self, Box<dyn std::error::Error>> {
        // Пробуем загрузить из файла, если не получается - используем значения по умолчанию
        match fs::read_to_string("src/config/default.toml") {
            Ok(config_content) => {
                let config: ServerConfig = toml::from_str(&config_content)?;
                Ok(config)
            }
            Err(_) => {
                println!("Config file not found, using default configuration");
                Ok(ServerConfig::default())
            }
        }
    }
}