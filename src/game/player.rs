use crate::game::inventory::{CropSeed, ItemKind, PlayerInventory};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Tool {
    Hoe,
    WateringCan,
    Seeds,
    None,
}

#[derive(Debug, Clone)]
pub struct Player {
    pub tile: (usize, usize),
    pub energy: i16,
    pub max_energy: i16,
    pub gold: u32,
    pub inventory: PlayerInventory,
    pub facing: Direction,
    pub active_tool: Tool,
    pub selected_seed: Option<CropSeed>,
    /// Accumulated charisma XP. 10 XP per level, capped at level 10 (100 XP).
    pub charisma_xp: u32,
    /// Hoe upgrade level: 0=basic, 1=copper (cheaper), 2=iron (3-tile line), 3=gold (5-tile line).
    pub hoe_level: u8,
    /// Watering can upgrade level: 0=basic, 1=copper (3-tile row), 2=iron (cross), 3=gold (3×3).
    pub can_level: u8,
    /// Currently equipped outfit index.
    pub outfit: u8,
    /// Player gender: 0 = Male, 1 = Female.
    pub gender: u8,
    /// Hairstyle index (mainly for female characters, 0-5).
    pub hairstyle: u8,
    /// Hair color index (0-5).
    pub hair_color: u8,
}

impl Player {
    pub fn new(start_energy: i16, start_gold: u32) -> Self {
        Self {
            tile: (10, 10),
            energy: start_energy,
            max_energy: start_energy,
            gold: start_gold,
            inventory: PlayerInventory::new(),
            facing: Direction::Down,
            active_tool: Tool::None,
            selected_seed: None,
            charisma_xp: 0,
            hoe_level: 0,
            can_level: 0,
            outfit: 0,
            gender: 0,
            hairstyle: 0,
            hair_color: 0,
        }
    }

    /// Charisma level 0–10. Each level requires 10 XP (total 100 XP to max).
    pub fn charisma_level(&self) -> u8 {
        (self.charisma_xp / 10).min(10) as u8
    }

    /// Called after successfully completing an NPC conversation.
    pub fn gain_charisma_xp(&mut self, amount: u32) {
        self.charisma_xp = (self.charisma_xp + amount).min(100);
    }

    pub fn spend_energy(&mut self, amount: i16) -> bool {
        if self.energy >= amount {
            self.energy -= amount;
            true
        } else {
            false
        }
    }

    pub fn restore_energy(&mut self) {
        self.energy = self.max_energy;
    }

    pub fn facing_tile(&self) -> (usize, usize) {
        let (col, row) = self.tile;
        match self.facing {
            Direction::Up => (col, row.saturating_sub(1)),
            Direction::Down => (col, row + 1),
            Direction::Left => (col.saturating_sub(1), row),
            Direction::Right => (col + 1, row),
        }
    }

    pub fn try_move(&mut self, dir: Direction, map_width: usize, map_height: usize) {
        let (col, row) = self.tile;
        let new_tile = match dir {
            Direction::Up => (col, row.saturating_sub(1)),
            Direction::Down => (col, (row + 1).min(map_height - 1)),
            Direction::Left => (col.saturating_sub(1), row),
            Direction::Right => ((col + 1).min(map_width - 1), row),
        };
        self.facing = dir;
        self.tile = new_tile;
    }

    pub fn add_seed_to_inventory(&mut self, item: ItemKind, qty: u32) {
        self.inventory.add(item, qty);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_player() -> Player {
        Player::new(270, 100)
    }

    #[test]
    fn spend_energy_succeeds_when_enough() {
        let mut p = test_player();
        assert!(p.spend_energy(10));
        assert_eq!(p.energy, 260);
    }

    #[test]
    fn spend_energy_fails_when_not_enough() {
        let mut p = test_player();
        p.energy = 1;
        assert!(!p.spend_energy(10));
        assert_eq!(p.energy, 1);
    }

    #[test]
    fn restore_energy_fills_to_max() {
        let mut p = test_player();
        p.energy = 10;
        p.restore_energy();
        assert_eq!(p.energy, p.max_energy);
    }

    #[test]
    fn facing_tile_up() {
        let mut p = test_player();
        p.tile = (5, 5);
        p.facing = Direction::Up;
        assert_eq!(p.facing_tile(), (5, 4));
    }

    #[test]
    fn facing_tile_down() {
        let mut p = test_player();
        p.tile = (5, 5);
        p.facing = Direction::Down;
        assert_eq!(p.facing_tile(), (5, 6));
    }
}
