use crate::game::config::GameConfig;

pub struct Assets {
    pub config: GameConfig,
}

pub async fn load_all() -> Assets {
    let config = load_config().await;
    Assets { config }
}

async fn load_config() -> GameConfig {
    // Try loading from file; fall back to embedded default
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Ok(bytes) = macroquad::file::load_file("static/assets/config.json").await {
            if let Ok(json) = std::str::from_utf8(&bytes) {
                if let Ok(config) = GameConfig::from_json(json) {
                    return config;
                }
            }
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        if let Ok(bytes) = macroquad::file::load_file("assets/config.json").await {
            if let Ok(json) = std::str::from_utf8(&bytes) {
                if let Ok(config) = GameConfig::from_json(json) {
                    return config;
                }
            }
        }
    }
    default_config()
}

fn default_config() -> GameConfig {
    let json = include_str!("../../static/assets/config.json");
    GameConfig::from_json(json).expect("Embedded config.json must be valid")
}
