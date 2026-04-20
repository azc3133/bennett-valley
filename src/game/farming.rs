use crate::game::crop::{CropKind, CropState};
use crate::game::inventory::{FishKind, ForageKind, HarvestedCrop, ItemKind, OreKind};
use crate::game::player::Player;
use crate::game::time::Season;
use crate::game::world::{FarmMap, TileKind};

#[derive(Debug, PartialEq)]
pub enum ActionError {
    NotEnoughEnergy,
    InvalidTile,
    NoSeedSelected,
    NothingToHarvest,
    NotMatureYet,
    NotInInventory,
    NothingToForage,
    NotAFishingSpot,
    NotARock,
}

#[derive(Debug)]
pub enum ActionResult {
    Hoed,
    Planted,
    Watered,
    Harvested(HarvestedCrop),
    Shipped { gold: u32 },
}

pub fn hoe_tile(
    map: &mut FarmMap,
    player: &mut Player,
    col: usize,
    row: usize,
    hoe_cost: i16,
) -> Result<ActionResult, ActionError> {
    let tile = map.get(col, row).ok_or(ActionError::InvalidTile)?;
    if !tile.kind.is_tillable() {
        return Err(ActionError::InvalidTile);
    }
    if !player.spend_energy(hoe_cost) {
        return Err(ActionError::NotEnoughEnergy);
    }
    map.get_mut(col, row).unwrap().kind = TileKind::Tilled;
    Ok(ActionResult::Hoed)
}

pub fn plant_seed(
    map: &mut FarmMap,
    player: &mut Player,
    col: usize,
    row: usize,
    kind: CropKind,
    plant_cost: i16,
) -> Result<ActionResult, ActionError> {
    let tile = map.get(col, row).ok_or(ActionError::InvalidTile)?;
    if !tile.kind.is_plantable() {
        return Err(ActionError::InvalidTile);
    }
    if tile.crop.is_some() {
        return Err(ActionError::InvalidTile);
    }
    if !player.spend_energy(plant_cost) {
        return Err(ActionError::NotEnoughEnergy);
    }
    let tile = map.get_mut(col, row).unwrap();
    tile.crop = Some(CropState::new(kind));
    Ok(ActionResult::Planted)
}

pub fn water_tile(
    map: &mut FarmMap,
    player: &mut Player,
    col: usize,
    row: usize,
    water_cost: i16,
) -> Result<ActionResult, ActionError> {
    let tile = map.get(col, row).ok_or(ActionError::InvalidTile)?;
    if !tile.kind.is_waterable() && tile.crop.is_none() {
        return Err(ActionError::InvalidTile);
    }
    if !player.spend_energy(water_cost) {
        return Err(ActionError::NotEnoughEnergy);
    }
    let tile = map.get_mut(col, row).unwrap();
    tile.kind = TileKind::Watered;
    if let Some(crop) = &mut tile.crop {
        crop.watered_today = true;
    }
    Ok(ActionResult::Watered)
}

pub fn harvest_tile(
    map: &mut FarmMap,
    player: &mut Player,
    col: usize,
    row: usize,
    grow_days: u8,
    harvest_cost: i16,
) -> Result<ActionResult, ActionError> {
    let tile = map.get(col, row).ok_or(ActionError::InvalidTile)?;
    let crop = tile.crop.as_ref().ok_or(ActionError::NothingToHarvest)?;
    if !crop.is_mature(grow_days) {
        return Err(ActionError::NotMatureYet);
    }
    if !player.spend_energy(harvest_cost) {
        return Err(ActionError::NotEnoughEnergy);
    }
    let tile = map.get_mut(col, row).unwrap();
    let crop = tile.crop.take().unwrap();
    tile.kind = TileKind::Tilled;
    let harvested = crop.kind.to_harvested();
    player.inventory.add(ItemKind::Crop(harvested.clone()), 1);
    Ok(ActionResult::Harvested(harvested))
}

/// Forage the patch at (col, row). Converts ForagePatch → ForagePatchEmpty and
/// adds a season-appropriate item to the player's inventory.
pub fn forage_tile(
    map: &mut FarmMap,
    player: &mut Player,
    col: usize,
    row: usize,
    forage_cost: i16,
    season: &Season,
) -> Result<ForageKind, ActionError> {
    let tile = map.get(col, row).ok_or(ActionError::InvalidTile)?;
    if tile.kind != TileKind::ForagePatch {
        return Err(ActionError::InvalidTile);
    }
    if !player.spend_energy(forage_cost) {
        return Err(ActionError::NotEnoughEnergy);
    }
    let tile = map.get_mut(col, row).unwrap();
    tile.kind = TileKind::ForagePatchEmpty;

    // Season-specific drop weights (simple deterministic cycle based on tile position)
    let item = season_forage_item(season, col, row);
    player.inventory.add(ItemKind::Forage(item.clone()), 1);
    Ok(item)
}

fn season_forage_item(season: &Season, col: usize, row: usize) -> ForageKind {
    // Use tile position as a cheap pseudo-random selector
    let idx = (col * 3 + row * 7) % 3;
    match season {
        Season::Spring => [ForageKind::Flower, ForageKind::Fern, ForageKind::Herb][idx].clone(),
        Season::Summer => [ForageKind::Berry,  ForageKind::Berry, ForageKind::Herb][idx].clone(),
        Season::Fall   => [ForageKind::Mushroom, ForageKind::Mushroom, ForageKind::Berry][idx].clone(),
        Season::Winter => ForageKind::Fern, // sparse winter forage
    }
}

/// Forage an oak tree at (col, row). Only yields acorns in Fall.
pub fn forage_oak(
    map: &mut FarmMap,
    player: &mut Player,
    col: usize,
    row: usize,
    forage_cost: i16,
    season: &Season,
) -> Result<ForageKind, ActionError> {
    let tile = map.get(col, row).ok_or(ActionError::InvalidTile)?;
    match tile.kind {
        TileKind::OakTree => {}
        TileKind::OakTreeEmpty => return Err(ActionError::NothingToForage),
        _ => return Err(ActionError::NothingToForage),
    }
    if *season != Season::Fall {
        return Err(ActionError::NothingToForage);
    }
    if !player.spend_energy(forage_cost) {
        return Err(ActionError::NotEnoughEnergy);
    }
    let tile = map.get_mut(col, row).unwrap();
    tile.kind = TileKind::OakTreeEmpty;
    let item = ForageKind::Acorn;
    player.inventory.add(ItemKind::Forage(item.clone()), 1);
    Ok(item)
}

/// Strike the rock at (col, row). Reduces HP by 1; at 0 the rock shatters and
/// drops an ore item into the player's inventory. Returns the ore if broken.
pub fn mine_tile(
    map: &mut FarmMap,
    player: &mut Player,
    col: usize,
    row: usize,
    mine_cost: i16,
    rock_hp: u8,
) -> Result<Option<OreKind>, ActionError> {
    let tile = map.get(col, row).ok_or(ActionError::InvalidTile)?;
    let hp = match tile.kind {
        TileKind::Rock(hp) => hp,
        _ => return Err(ActionError::NotARock),
    };
    if !player.spend_energy(mine_cost) {
        return Err(ActionError::NotEnoughEnergy);
    }
    let tile = map.get_mut(col, row).unwrap();
    if hp <= 1 {
        let ore = rock_ore_drop(col, row, rock_hp);
        tile.kind = TileKind::Grass;
        player.inventory.add(ItemKind::Ore(ore.clone()), 1);
        Ok(Some(ore))
    } else {
        tile.kind = TileKind::Rock(hp - 1);
        Ok(None)
    }
}

/// Deterministic ore drop based on tile position.
/// ~60% Copper, ~30% Iron, ~10% Gold.
fn rock_ore_drop(col: usize, row: usize, _rock_hp: u8) -> OreKind {
    let idx = (col * 7 + row * 13) % 10;
    match idx {
        0..=5 => OreKind::Copper,
        6..=8 => OreKind::Iron,
        _     => OreKind::Gold,
    }
}

/// Fish at the spot at (col, row). Player must be standing on a FishingSpot tile.
pub fn fish_tile(
    map: &FarmMap,
    player: &mut Player,
    col: usize,
    row: usize,
    fish_cost: i16,
    season: &Season,
) -> Result<FishKind, ActionError> {
    let tile = map.get(col, row).ok_or(ActionError::InvalidTile)?;
    if tile.kind != TileKind::FishingSpot {
        return Err(ActionError::NotAFishingSpot);
    }
    if !player.spend_energy(fish_cost) {
        return Err(ActionError::NotEnoughEnergy);
    }
    let fish = season_fish(season, col, row);
    player.inventory.add(ItemKind::Fish(fish.clone()), 1);
    Ok(fish)
}

pub fn season_fish_pub(season: &Season, col: usize, row: usize) -> FishKind {
    season_fish(season, col, row)
}

fn season_fish(season: &Season, col: usize, row: usize) -> FishKind {
    let idx = (col * 5 + row * 3) % 2;
    match season {
        Season::Spring => [FishKind::Bass,   FishKind::Catfish][idx].clone(),
        Season::Summer => [FishKind::Trout,  FishKind::Bass][idx].clone(),
        Season::Fall   => [FishKind::Salmon, FishKind::Trout][idx].clone(),
        Season::Winter => [FishKind::Catfish, FishKind::Bass][idx].clone(),
    }
}

pub fn ship_item(
    player: &mut Player,
    item: &ItemKind,
    qty: u32,
    sell_price: u32,
) -> Result<ActionResult, ActionError> {
    if !player.inventory.remove(item, qty) {
        return Err(ActionError::NotInInventory);
    }
    let gold = sell_price * qty;
    Ok(ActionResult::Shipped { gold })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::config::test_config;
    use crate::game::world::test_farm_map;
    use crate::game::player::Player;

    fn test_player() -> Player {
        Player::new(270, 100)
    }

    #[test]
    fn hoe_converts_grass_to_tilled() {
        let mut map = test_farm_map();
        let mut player = test_player();
        // tile (3,3) is Grass
        map.tiles[3][3].kind = TileKind::Grass;
        hoe_tile(&mut map, &mut player, 3, 3, 2).unwrap();
        assert_eq!(map.get(3, 3).unwrap().kind, TileKind::Tilled);
    }

    #[test]
    fn hoe_deducts_energy() {
        let mut map = test_farm_map();
        let mut player = test_player();
        map.tiles[3][3].kind = TileKind::Grass;
        hoe_tile(&mut map, &mut player, 3, 3, 2).unwrap();
        assert_eq!(player.energy, 268);
    }

    #[test]
    fn hoe_on_tilled_returns_error() {
        let mut map = test_farm_map();
        let mut player = test_player();
        let result = hoe_tile(&mut map, &mut player, 5, 5, 2); // pre-tilled
        assert_eq!(result.unwrap_err(), ActionError::InvalidTile);
    }

    #[test]
    fn hoe_with_insufficient_energy_returns_error() {
        let mut map = test_farm_map();
        let mut player = test_player();
        player.energy = 1;
        map.tiles[3][3].kind = TileKind::Grass;
        let result = hoe_tile(&mut map, &mut player, 3, 3, 2);
        assert_eq!(result.unwrap_err(), ActionError::NotEnoughEnergy);
    }

    #[test]
    fn plant_seed_on_tilled_tile() {
        let mut map = test_farm_map();
        let mut player = test_player();
        plant_seed(&mut map, &mut player, 5, 5, CropKind::Parsnip, 1).unwrap();
        assert!(map.get(5, 5).unwrap().crop.is_some());
    }

    #[test]
    fn plant_on_non_tilled_returns_error() {
        let mut map = test_farm_map();
        let mut player = test_player();
        let result = plant_seed(&mut map, &mut player, 0, 0, CropKind::Parsnip, 1);
        assert_eq!(result.unwrap_err(), ActionError::InvalidTile);
    }

    #[test]
    fn water_sets_watered_flag() {
        let mut map = test_farm_map();
        let mut player = test_player();
        plant_seed(&mut map, &mut player, 5, 5, CropKind::Parsnip, 1).unwrap();
        water_tile(&mut map, &mut player, 5, 5, 2).unwrap();
        assert!(map.get(5, 5).unwrap().crop.as_ref().unwrap().watered_today);
    }

    #[test]
    fn harvest_mature_adds_to_inventory() {
        let config = test_config();
        let crop_cfg = config.crops.get("parsnip").unwrap();
        let mut map = test_farm_map();
        let mut player = test_player();
        plant_seed(&mut map, &mut player, 5, 5, CropKind::Parsnip, 1).unwrap();
        // Manually mature the crop
        map.get_mut(5, 5).unwrap().crop.as_mut().unwrap().days_grown = crop_cfg.grow_days;
        harvest_tile(&mut map, &mut player, 5, 5, crop_cfg.grow_days, 1).unwrap();
        assert_eq!(
            player.inventory.count(&ItemKind::Crop(HarvestedCrop::Parsnip)),
            1
        );
    }

    #[test]
    fn harvest_immature_returns_error() {
        let mut map = test_farm_map();
        let mut player = test_player();
        plant_seed(&mut map, &mut player, 5, 5, CropKind::Parsnip, 1).unwrap();
        let result = harvest_tile(&mut map, &mut player, 5, 5, 4, 1);
        assert_eq!(result.unwrap_err(), ActionError::NotMatureYet);
    }

    #[test]
    fn ship_item_returns_gold() {
        let mut player = test_player();
        player.inventory.add(ItemKind::Crop(HarvestedCrop::Parsnip), 3);
        let result = ship_item(
            &mut player,
            &ItemKind::Crop(HarvestedCrop::Parsnip),
            3,
            35,
        ).unwrap();
        match result {
            ActionResult::Shipped { gold } => assert_eq!(gold, 105),
            _ => panic!("Expected Shipped"),
        }
    }

    #[test]
    fn ship_item_not_in_inventory_returns_error() {
        let mut player = test_player();
        let result = ship_item(
            &mut player,
            &ItemKind::Crop(HarvestedCrop::Parsnip),
            1,
            35,
        );
        assert_eq!(result.unwrap_err(), ActionError::NotInInventory);
    }

    #[test]
    fn forage_adds_item_to_inventory() {
        let mut map = test_farm_map();
        let mut player = test_player();
        map.tiles[3][3].kind = TileKind::ForagePatch;
        let result = forage_tile(&mut map, &mut player, 3, 3, 1, &Season::Spring);
        assert!(result.is_ok());
        // Inventory should have exactly one forage item
        let total: u32 = [
            ForageKind::Mushroom, ForageKind::Berry, ForageKind::Herb,
            ForageKind::Flower, ForageKind::Fern,
        ]
        .iter()
        .map(|k| player.inventory.count(&ItemKind::Forage(k.clone())))
        .sum();
        assert_eq!(total, 1);
    }

    #[test]
    fn forage_converts_patch_to_empty() {
        let mut map = test_farm_map();
        let mut player = test_player();
        map.tiles[3][3].kind = TileKind::ForagePatch;
        forage_tile(&mut map, &mut player, 3, 3, 1, &Season::Spring).unwrap();
        assert_eq!(map.get(3, 3).unwrap().kind, TileKind::ForagePatchEmpty);
    }

    #[test]
    fn forage_empty_patch_returns_error() {
        let mut map = test_farm_map();
        let mut player = test_player();
        map.tiles[3][3].kind = TileKind::ForagePatchEmpty;
        let result = forage_tile(&mut map, &mut player, 3, 3, 1, &Season::Spring);
        assert_eq!(result.unwrap_err(), ActionError::InvalidTile);
    }

    #[test]
    fn mine_reduces_rock_hp() {
        let mut map = test_farm_map();
        let mut player = test_player();
        map.tiles[3][3].kind = TileKind::Rock(3);
        mine_tile(&mut map, &mut player, 3, 3, 4, 3).unwrap();
        assert_eq!(map.get(3, 3).unwrap().kind, TileKind::Rock(2));
    }

    #[test]
    fn mine_breaks_rock_on_last_hit() {
        let mut map = test_farm_map();
        let mut player = test_player();
        map.tiles[3][3].kind = TileKind::Rock(1);
        let result = mine_tile(&mut map, &mut player, 3, 3, 4, 3).unwrap();
        assert!(result.is_some());
        assert_eq!(map.get(3, 3).unwrap().kind, TileKind::Grass);
        // Inventory should contain one ore
        let total: u32 = [OreKind::Copper, OreKind::Iron, OreKind::Gold]
            .iter()
            .map(|k| player.inventory.count(&ItemKind::Ore(k.clone())))
            .sum();
        assert_eq!(total, 1);
    }

    #[test]
    fn mine_on_non_rock_returns_error() {
        let mut map = test_farm_map();
        let mut player = test_player();
        // tile (3,3) is Grass
        let result = mine_tile(&mut map, &mut player, 3, 3, 4, 3);
        assert_eq!(result.unwrap_err(), ActionError::NotARock);
    }

    #[test]
    fn mine_deducts_energy() {
        let mut map = test_farm_map();
        let mut player = test_player();
        map.tiles[3][3].kind = TileKind::Rock(3);
        mine_tile(&mut map, &mut player, 3, 3, 4, 3).unwrap();
        assert_eq!(player.energy, 266); // 270 - 4
    }

    #[test]
    fn mine_insufficient_energy_returns_error() {
        let mut map = test_farm_map();
        let mut player = test_player();
        player.energy = 2;
        map.tiles[3][3].kind = TileKind::Rock(3);
        let result = mine_tile(&mut map, &mut player, 3, 3, 4, 3);
        assert_eq!(result.unwrap_err(), ActionError::NotEnoughEnergy);
    }

    #[test]
    fn fish_on_fishing_spot_adds_to_inventory() {
        let mut map = test_farm_map();
        let mut player = test_player();
        map.tiles[3][3].kind = TileKind::FishingSpot;
        let result = fish_tile(&map, &mut player, 3, 3, 2, &Season::Spring);
        assert!(result.is_ok());
        let fish = result.unwrap();
        assert!(matches!(fish, FishKind::Bass | FishKind::Catfish));
        let total: u32 = [FishKind::Bass, FishKind::Catfish, FishKind::Trout, FishKind::Salmon]
            .iter()
            .map(|k| player.inventory.count(&ItemKind::Fish(k.clone())))
            .sum();
        assert_eq!(total, 1);
    }

    #[test]
    fn fish_on_non_fishing_spot_returns_error() {
        let mut map = test_farm_map();
        let mut player = test_player();
        // tile (3,3) is Grass by default
        let result = fish_tile(&map, &mut player, 3, 3, 2, &Season::Spring);
        assert_eq!(result.unwrap_err(), ActionError::NotAFishingSpot);
    }

    #[test]
    fn fish_deducts_energy() {
        let mut map = test_farm_map();
        let mut player = test_player();
        map.tiles[3][3].kind = TileKind::FishingSpot;
        fish_tile(&map, &mut player, 3, 3, 2, &Season::Spring).unwrap();
        assert_eq!(player.energy, 268);
    }

    #[test]
    fn fish_summer_yields_trout_or_bass() {
        let mut map = test_farm_map();
        let mut player = test_player();
        map.tiles[3][3].kind = TileKind::FishingSpot;
        let fish = fish_tile(&map, &mut player, 3, 3, 2, &Season::Summer).unwrap();
        assert!(matches!(fish, FishKind::Trout | FishKind::Bass));
    }

    #[test]
    fn fish_fall_yields_salmon_or_trout() {
        let mut map = test_farm_map();
        let mut player = test_player();
        map.tiles[3][3].kind = TileKind::FishingSpot;
        let fish = fish_tile(&map, &mut player, 3, 3, 2, &Season::Fall).unwrap();
        assert!(matches!(fish, FishKind::Salmon | FishKind::Trout));
    }

    #[test]
    fn forage_fall_yields_mushroom_or_berry() {
        let mut map = test_farm_map();
        let mut player = test_player();
        map.tiles[3][3].kind = TileKind::ForagePatch;
        let item = forage_tile(&mut map, &mut player, 3, 3, 1, &Season::Fall).unwrap();
        assert!(matches!(item, ForageKind::Mushroom | ForageKind::Berry));
    }
}
