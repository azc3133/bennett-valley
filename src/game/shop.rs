use std::collections::HashMap;
use crate::game::inventory::{CropSeed, ItemKind};

#[derive(Debug, Clone)]
pub struct ShopItem {
    pub buy_price: u32,
    pub sell_price: u32,
}

#[derive(Debug, Clone)]
pub struct ShopInventory {
    pub items: HashMap<String, ShopItem>,
}

#[derive(Debug, PartialEq)]
pub enum ShopError {
    NotEnoughGold,
    ItemNotAvailable,
    NotInInventory,
}

impl ShopInventory {
    pub fn new() -> Self {
        Self { items: HashMap::new() }
    }

    pub fn add_item(&mut self, name: &str, buy_price: u32, sell_price: u32) {
        self.items.insert(name.to_string(), ShopItem { buy_price, sell_price });
    }

    pub fn buy_seeds(
        &self,
        player_gold: &mut u32,
        player_inv: &mut crate::game::inventory::PlayerInventory,
        seed: CropSeed,
        qty: u32,
        item_name: &str,
    ) -> Result<(), ShopError> {
        let item = self.items.get(item_name).ok_or(ShopError::ItemNotAvailable)?;
        let total = item.buy_price * qty;
        if *player_gold < total {
            return Err(ShopError::NotEnoughGold);
        }
        *player_gold -= total;
        player_inv.add(ItemKind::Seed(seed), qty);
        Ok(())
    }

    pub fn sell_price_for(&self, item_name: &str) -> Option<u32> {
        self.items.get(item_name).map(|i| i.sell_price)
    }
}

impl Default for ShopInventory {
    fn default() -> Self {
        Self::new()
    }
}

pub fn make_default_shop(config: &crate::game::config::GameConfig) -> ShopInventory {
    let mut shop = ShopInventory::new();
    for (name, crop) in &config.crops {
        shop.add_item(name, crop.seed_buy_price, crop.sell_price);
    }
    // Pendant: used to propose marriage. Not sellable (sell_price 0).
    shop.add_item("pendant", 500, 0);
    // Tool upgrades (gold cost; not resellable).
    shop.add_item("copper_hoe", 200, 0);
    shop.add_item("iron_hoe",   600, 0);
    shop.add_item("gold_hoe",  1500, 0);
    shop.add_item("copper_can", 250, 0);
    shop.add_item("iron_can",   750, 0);
    shop.add_item("gold_can",  2000, 0);
    // House extension
    shop.add_item("house_extension", 10000, 0);
    shop
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::config::test_config;
    use crate::game::inventory::PlayerInventory;

    fn test_shop() -> ShopInventory {
        let config = test_config();
        make_default_shop(&config)
    }

    #[test]
    fn buy_seeds_deducts_gold() {
        let shop = test_shop();
        let mut gold = 100u32;
        let mut inv = PlayerInventory::new();
        shop.buy_seeds(&mut gold, &mut inv, CropSeed::Parsnip, 2, "parsnip").unwrap();
        assert_eq!(gold, 60); // 100 - 2*20
    }

    #[test]
    fn buy_seeds_adds_to_inventory() {
        let shop = test_shop();
        let mut gold = 100u32;
        let mut inv = PlayerInventory::new();
        shop.buy_seeds(&mut gold, &mut inv, CropSeed::Parsnip, 3, "parsnip").unwrap();
        assert_eq!(inv.count(&ItemKind::Seed(CropSeed::Parsnip)), 3);
    }

    #[test]
    fn buy_more_than_affordable_returns_error() {
        let shop = test_shop();
        let mut gold = 10u32;
        let mut inv = PlayerInventory::new();
        let result = shop.buy_seeds(&mut gold, &mut inv, CropSeed::Parsnip, 1, "parsnip");
        assert_eq!(result.unwrap_err(), ShopError::NotEnoughGold);
    }
}
