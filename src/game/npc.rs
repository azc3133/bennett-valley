use std::collections::{HashMap, HashSet};

/// Returns true if a tile should be off-limits to NPCs.
/// NPCs avoid the player's farmhouse area, crop fields, and animal pens.
pub fn is_npc_excluded(col: usize, row: usize, map: &crate::game::world::FarmMap) -> bool {
    // Farmhouse area (cols 1-3, rows 1-4) — player's home
    if col >= 1 && col <= 3 && row >= 1 && row <= 4 { return true; }
    // Animal pen area (cols 5-18, rows 14-20) — pens/barns
    if col >= 5 && col <= 18 && row >= 14 && row <= 20 { return true; }
    // Crop fields — avoid any tilled/watered tile or tile with a crop
    if let Some(tile) = map.get(col, row) {
        use crate::game::world::TileKind;
        if matches!(tile.kind, TileKind::Tilled | TileKind::Watered) { return true; }
        if tile.crop.is_some() { return true; }
    }
    false
}
use crate::game::inventory::ItemKind;

#[derive(Debug, Clone)]
pub struct ScheduleEntry {
    pub start_hour: u8,
    pub end_hour: u8,
    pub tile: (usize, usize),
}

pub const VICTOR_ID: u8 = 15;
pub const VICTOR_FINAL_HEARTS: u8 = 5; // friendship / 25 >= 5 → 125+

/// Friendship cap before a loved gift is given (4 hearts = 100).
const FRIENDSHIP_CAP_UNGIFTED: u8 = 100;
/// Friendship cap after loved gift (8 hearts = 200).
const FRIENDSHIP_CAP_GIFTED: u8 = 200;

#[derive(Debug, Clone)]
pub struct NPC {
    pub id: u8,
    pub name: String,
    pub personality: String,
    pub tile: (usize, usize),
    pub schedule: Vec<ScheduleEntry>,
    pub friendship: u8, // 0–250; hearts = friendship / 25
    pub gifted_today: bool,
    pub talked_today: bool,
    pub marriageable: bool,
    /// Item preferences: item_name → score (2=loved, 1=liked, 0=neutral, -1=disliked).
    pub gift_preferences: HashMap<String, i8>,
    /// True once the player has gifted this NPC a loved item (pref >= 2).
    /// Unlocks friendship past the 4-heart cap.
    pub loved_gift_given: bool,
}

impl NPC {
    pub fn new(
        id: u8,
        name: String,
        personality: String,
        schedule: Vec<ScheduleEntry>,
        marriageable: bool,
        gift_preferences: HashMap<String, i8>,
    ) -> Self {
        let start_tile = schedule.first().map(|s| s.tile).unwrap_or((0, 0));
        Self {
            id,
            name,
            personality,
            tile: start_tile,
            schedule,
            friendship: 0,
            gifted_today: false,
            talked_today: false,
            marriageable,
            gift_preferences,
            loved_gift_given: false,
        }
    }

    pub fn hearts(&self) -> u8 {
        self.friendship / 25
    }

    /// Returns true when conversation friendship is capped and a loved gift is needed.
    pub fn friendship_capped(&self) -> bool {
        let cap = if self.loved_gift_given { FRIENDSHIP_CAP_GIFTED } else { FRIENDSHIP_CAP_UNGIFTED };
        self.friendship >= cap
    }

    /// Returns the tile this NPC should be heading toward at the given hour.
    pub fn schedule_target(&self, hour: u8) -> (usize, usize) {
        for entry in &self.schedule {
            if hour >= entry.start_hour && hour < entry.end_hour {
                return entry.tile;
            }
        }
        self.schedule.last().map(|s| s.tile).unwrap_or(self.tile)
    }

    /// Teleport to the correct schedule tile (used on game init / day start).
    pub fn snap_to_schedule(&mut self, hour: u8) {
        self.tile = self.schedule_target(hour);
    }

    /// Move one tile toward `target`, respecting passability and the set of
    /// tiles already claimed by other NPCs this step.
    pub fn step_toward(
        &mut self,
        target: (usize, usize),
        map: &crate::game::world::FarmMap,
        occupied: &HashSet<(usize, usize)>,
    ) {
        if self.tile == target {
            return;
        }
        let (cx, cy) = (self.tile.0 as i32, self.tile.1 as i32);
        let (tx, ty) = (target.0 as i32, target.1 as i32);
        let dx = (tx - cx).signum();
        let dy = (ty - cy).signum();

        let prefer_horiz = (tx - cx).abs() >= (ty - cy).abs();
        let candidates: [(i32, i32); 2] = if prefer_horiz {
            [(cx + dx, cy), (cx, cy + dy)]
        } else {
            [(cx, cy + dy), (cx + dx, cy)]
        };

        for (nx, ny) in candidates {
            if nx < 0 || ny < 0 { continue; }
            let (nx, ny) = (nx as usize, ny as usize);
            if occupied.contains(&(nx, ny)) { continue; }
            if is_npc_excluded(nx, ny, map) { continue; }
            if let Some(tile) = map.get(nx, ny) {
                if tile.kind.is_passable() {
                    self.tile = (nx, ny);
                    return;
                }
            }
        }
    }

    /// Legacy gift method (used by tests). Prefer `give_gift_by_name` for gameplay.
    pub fn give_gift(&mut self, _item: &ItemKind, preference: i8) -> i8 {
        if self.gifted_today { return 0; }
        self.gifted_today = true;
        let change = match preference {
            p if p > 0 => (p as u8 * 20).min(250 - self.friendship),
            p if p < 0 => 0,
            _ => 10u8.min(250 - self.friendship),
        };
        self.friendship = self.friendship.saturating_add(change);
        preference
    }

    /// Give a gift by item name. Looks up preference from this NPC's preferences.
    /// Returns the preference score (or 0 if neutral/unknown, or -99 if already gifted today).
    /// Loved gifts (pref >= 2) also set `loved_gift_given = true`.
    pub fn give_gift_by_name(&mut self, item_name: &str) -> i8 {
        if self.gifted_today { return -99; }
        self.gifted_today = true;
        let preference = self.gift_preferences.get(item_name).copied().unwrap_or(0);
        if preference < 0 {
            // disliked — friendship unchanged
            return preference;
        }
        if preference >= 2 {
            self.loved_gift_given = true;
        }
        let gain: u8 = match preference {
            p if p >= 2 => 30,
            1            => 15,
            _            => 5,
        };
        let cap = if self.loved_gift_given { FRIENDSHIP_CAP_GIFTED } else { FRIENDSHIP_CAP_UNGIFTED };
        let actual_gain = gain.min(cap.saturating_sub(self.friendship));
        self.friendship = self.friendship.saturating_add(actual_gain);
        preference
    }

    /// Gain friendship from a conversation (once per NPC per day).
    /// Capped at 4 hearts until a loved gift is given, then capped at 8 hearts.
    /// Returns friendship gained (0 if already talked or capped).
    pub fn gain_conversation_friendship(&mut self, amount: u8) -> u8 {
        if self.talked_today { return 0; }
        self.talked_today = true;
        let cap = if self.loved_gift_given { FRIENDSHIP_CAP_GIFTED } else { FRIENDSHIP_CAP_UNGIFTED };
        if self.friendship >= cap { return 0; }
        let gain = amount.min(cap - self.friendship);
        self.friendship = self.friendship.saturating_add(gain);
        gain
    }

    pub fn reset_daily(&mut self) {
        self.gifted_today = false;
        self.talked_today = false;
        // loved_gift_given and friendship persist across days
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::inventory::{HarvestedCrop, ItemKind};

    fn test_npc() -> NPC {
        let mut prefs = HashMap::new();
        prefs.insert("parsnip".to_string(), 1i8);
        prefs.insert("salmon".to_string(), 2i8);
        NPC::new(
            0,
            "Elara".to_string(),
            "curious".to_string(),
            vec![
                ScheduleEntry { start_hour: 6, end_hour: 12, tile: (10, 10) },
                ScheduleEntry { start_hour: 12, end_hour: 20, tile: (20, 15) },
            ],
            false,
            prefs,
        )
    }

    #[test]
    fn npc_at_correct_location_for_morning() {
        let mut npc = test_npc();
        npc.snap_to_schedule(8);
        assert_eq!(npc.tile, (10, 10));
    }

    #[test]
    fn npc_at_correct_location_for_afternoon() {
        let mut npc = test_npc();
        npc.snap_to_schedule(14);
        assert_eq!(npc.tile, (20, 15));
    }

    #[test]
    fn gift_increases_friendship() {
        let mut npc = test_npc();
        npc.give_gift(&ItemKind::Crop(HarvestedCrop::Parsnip), 1);
        assert!(npc.friendship > 0);
    }

    #[test]
    fn gift_twice_same_day_no_extra() {
        let mut npc = test_npc();
        npc.give_gift(&ItemKind::Crop(HarvestedCrop::Parsnip), 1);
        let after_first = npc.friendship;
        npc.give_gift(&ItemKind::Crop(HarvestedCrop::Parsnip), 1);
        assert_eq!(npc.friendship, after_first);
    }

    #[test]
    fn reset_daily_allows_gift_again() {
        let mut npc = test_npc();
        npc.give_gift(&ItemKind::Crop(HarvestedCrop::Parsnip), 1);
        npc.reset_daily();
        let after_reset = npc.friendship;
        npc.give_gift(&ItemKind::Crop(HarvestedCrop::Parsnip), 1);
        assert!(npc.friendship > after_reset);
    }

    #[test]
    fn hearts_computed_from_friendship() {
        let mut npc = test_npc();
        npc.friendship = 75;
        assert_eq!(npc.hearts(), 3);
    }

    #[test]
    fn give_gift_by_name_loved_unlocks_cap() {
        let mut npc = test_npc();
        // Gift loved item (salmon, pref=2)
        let pref = npc.give_gift_by_name("salmon");
        assert_eq!(pref, 2);
        assert!(npc.loved_gift_given);
        assert!(npc.friendship > 0);
    }

    #[test]
    fn friendship_capped_at_4_hearts_without_loved_gift() {
        let mut npc = test_npc();
        npc.friendship = 100; // exactly 4 hearts
        // Conversation should give 0 (capped)
        let gained = npc.gain_conversation_friendship(5);
        assert_eq!(gained, 0);
    }

    #[test]
    fn friendship_grows_past_4_hearts_after_loved_gift() {
        let mut npc = test_npc();
        npc.friendship = 100;
        npc.loved_gift_given = true;
        let gained = npc.gain_conversation_friendship(5);
        assert!(gained > 0);
    }
}
