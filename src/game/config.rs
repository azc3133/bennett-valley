use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CropSeason {
    Spring,
    Summer,
    Fall,
    Winter,
}

impl CropSeason {
    pub fn name(&self) -> &'static str {
        match self {
            CropSeason::Spring => "spring",
            CropSeason::Summer => "summer",
            CropSeason::Fall   => "fall",
            CropSeason::Winter => "winter",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct GameConfig {
    pub crops: HashMap<String, CropConfig>,
    pub energy: EnergyConfig,
    pub shop: ShopConfig,
    pub npcs: Vec<NpcConfig>,
    pub forage: ForageConfig,
    pub fish: FishConfig,
    pub ore: OreConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OreConfig {
    pub sell_prices: HashMap<String, u32>,
    /// Number of hits to break a rock (default 3).
    pub rock_hp: u8,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ForageConfig {
    /// Sell price for each forage item by name.
    pub sell_prices: HashMap<String, u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FishConfig {
    /// Sell price for each fish by name.
    pub sell_prices: HashMap<String, u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CropConfig {
    pub grow_days: u8,
    pub seed_buy_price: u32,
    pub sell_price: u32,
    pub season: CropSeason,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EnergyConfig {
    pub start: i16,
    pub hoe_cost: i16,
    pub water_cost: i16,
    pub plant_cost: i16,
    pub harvest_cost: i16,
    pub forage_cost: i16,
    pub fish_cost: i16,
    pub mine_cost: i16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ShopConfig {
    pub buy_markup: f32,
    pub sell_markdown: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NpcConfig {
    pub id: u8,
    pub name: String,
    pub personality: String,
    pub schedule: Vec<ScheduleConfig>,
    pub gift_preferences: HashMap<String, i8>,
    #[serde(default)]
    pub marriageable: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScheduleConfig {
    pub start_hour: u8,
    pub end_hour: u8,
    pub tile: (usize, usize),
}

impl GameConfig {
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[cfg(test)]
pub fn test_config() -> GameConfig {
    let mut crops = HashMap::new();
    crops.insert(
        "parsnip".to_string(),
        CropConfig { grow_days: 1, seed_buy_price: 20, sell_price: 35, season: CropSeason::Spring },
    );
    crops.insert(
        "potato".to_string(),
        CropConfig { grow_days: 2, seed_buy_price: 50, sell_price: 80, season: CropSeason::Spring },
    );
    GameConfig {
        crops,
        energy: EnergyConfig {
            start: 270,
            hoe_cost: 2,
            water_cost: 2,
            plant_cost: 1,
            harvest_cost: 1,
            forage_cost: 1,
            fish_cost: 2,
            mine_cost: 4,
        },
        shop: ShopConfig { buy_markup: 1.0, sell_markdown: 1.0 },
        npcs: vec![],
        forage: ForageConfig {
            sell_prices: {
                let mut m = HashMap::new();
                m.insert("mushroom".to_string(), 40);
                m.insert("berry".to_string(), 20);
                m.insert("herb".to_string(), 30);
                m.insert("flower".to_string(), 25);
                m.insert("fern".to_string(), 15);
                m
            },
        },
        fish: FishConfig {
            sell_prices: {
                let mut m = HashMap::new();
                m.insert("bass".to_string(), 50);
                m.insert("catfish".to_string(), 75);
                m.insert("trout".to_string(), 65);
                m.insert("salmon".to_string(), 80);
                m
            },
        },
        ore: OreConfig {
            rock_hp: 3,
            sell_prices: {
                let mut m = HashMap::new();
                m.insert("copper".to_string(), 25);
                m.insert("iron".to_string(), 50);
                m.insert("gold".to_string(), 100);
                m
            },
        },
    }
}
