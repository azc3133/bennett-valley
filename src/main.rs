use macroquad::prelude::*;
use serde_json;

use bennett_valley::{
    assets::loader,
    game::{
        dialogue::DialogueState,
        farming::{self, ActionError},
        inventory::CropSeed,
        player::Direction,
        save as game_save,
        state::{GamePhase, GameState},
    },
    input::handler::{collect_action_events, InputEvent},
    render::{camera::Camera, renderer},
};

/// Parse the JSON response from /api/chat into (npc_line, options).
/// Returns None if parsing fails (falls back to plain text).
#[cfg(target_arch = "wasm32")]
fn parse_npc_json(text: &str) -> Option<(String, Vec<String>)> {
    // Minimal JSON parse: look for "npc_line" and "options" keys.
    // We avoid pulling in serde_json for WASM size; do a simple string scan.
    let npc_line = extract_json_string(text, "npc_line")?;
    let options = extract_json_array(text, "options")?;
    if options.len() >= 2 {
        Some((npc_line, options))
    } else {
        None
    }
}

#[cfg(target_arch = "wasm32")]
fn extract_json_string(json: &str, key: &str) -> Option<String> {
    let needle = format!("\"{}\"", key);
    let pos = json.find(&needle)?;
    let after_key = &json[pos + needle.len()..];
    let colon = after_key.find(':')? + 1;
    let after_colon = after_key[colon..].trim_start();
    if !after_colon.starts_with('"') { return None; }
    let inner = &after_colon[1..];
    let mut result = String::new();
    let mut escaped = false;
    for ch in inner.chars() {
        if escaped {
            result.push(match ch { 'n' => '\n', 't' => '\t', _ => ch });
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == '"' {
            break;
        } else {
            result.push(ch);
        }
    }
    Some(result)
}

#[cfg(target_arch = "wasm32")]
fn extract_json_array(json: &str, key: &str) -> Option<Vec<String>> {
    let needle = format!("\"{}\"", key);
    let pos = json.find(&needle)?;
    let after_key = &json[pos + needle.len()..];
    let colon = after_key.find(':')? + 1;
    let after_colon = after_key[colon..].trim_start();
    if !after_colon.starts_with('[') { return None; }
    let inner = &after_colon[1..];
    // Find the closing ']' of the array, skipping ']' inside quoted strings.
    let bracket_end = {
        let mut in_q = false;
        let mut esc = false;
        let mut found = None;
        for (i, ch) in inner.char_indices() {
            if esc { esc = false; continue; }
            if ch == '\\' && in_q { esc = true; continue; }
            if ch == '"' { in_q = !in_q; continue; }
            if ch == ']' && !in_q { found = Some(i); break; }
        }
        found?
    };
    let array_str = &inner[..bracket_end];
    // Split on commas that are outside quotes
    let mut items = Vec::new();
    let mut in_str = false;
    let mut escaped = false;
    let mut current = String::new();
    for ch in array_str.chars() {
        if escaped { current.push(ch); escaped = false; continue; }
        if ch == '\\' && in_str { escaped = true; continue; }
        if ch == '"' { in_str = !in_str; continue; }
        if ch == ',' && !in_str {
            let s = current.trim().to_string();
            if !s.is_empty() { items.push(s); }
            current.clear();
        } else if in_str {
            current.push(ch);
        }
    }
    let s = current.trim().to_string();
    if !s.is_empty() { items.push(s); }
    Some(items)
}

/// Percent-encodes a string for use in query parameters.
fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9'
            | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            b' ' => out.push('+'),
            _ => { out.push('%'); out.push_str(&format!("{:02X}", b)); }
        }
    }
    out
}

// Channel for the NPC LLM coroutine.
thread_local! {
    static LLM_RESULT: std::cell::RefCell<Option<(usize, String)>> =
        std::cell::RefCell::new(None);
}

// Separate channel for old-friend letter coroutine.
thread_local! {
    static LLM_LETTER_RESULT: std::cell::RefCell<Option<String>> =
        std::cell::RefCell::new(None);
}


fn action_error_msg(e: &ActionError) -> &'static str {
    match e {
        ActionError::NotEnoughEnergy  => "Not enough energy!",
        ActionError::InvalidTile      => "Can't do that here.",
        ActionError::NoSeedSelected   => "Select a seed first (1/2/3).",
        ActionError::NothingToHarvest => "Nothing to harvest.",
        ActionError::NotMatureYet     => "Not ready to harvest yet.",
        ActionError::NotInInventory   => "No seeds in inventory.",
        ActionError::NothingToForage  => "Nothing to forage here.",
        ActionError::NotAFishingSpot  => "Stand on a fishing spot to fish.",
        ActionError::NotARock         => "No rock to mine here.",
    }
}

const MOVE_REPEAT_INTERVAL: f32 = 0.15; // seconds between tiles when held (animation takes 0.125s)

#[macroquad::main("Bennett Valley")]
async fn main() {
    let assets = loader::load_all().await;
    let config = assets.config.clone();
    let mut state = GameState::new(assets.config);

    // Try to restore a previous save from platform storage.
    if let Some(json) = game_save::platform_load() {
        if let Ok(data) = serde_json::from_str(&json) {
            state.apply_save(data);
        } else {
            state.snap_npcs();
        }
    } else {
        state.snap_npcs(); // place NPCs at their correct starting tiles
    }
    let mut camera = Camera::default();
    let mut camera2 = Camera::default(); // P2 camera for split-screen
    let mut tick_acc = 0.0f32;
    const TICK_SECS: f32 = 7.0;
    let mut move_cooldown: f32 = 0.0;
    let mut horse_autopilot_timer: f32 = 0.0;

    loop {
        let dt = get_frame_time();
        let sw = screen_width();
        let sh = screen_height();

        // ── Always keep camera target current (handles first-frame snap) ─────
        camera.set_target(state.player.tile.0, state.player.tile.1, sw, sh);

        // ── Movement ─────────────────────────────────────────────────────────
        // P1 movement: WASD (+ Arrow keys when co-op is off)
        let coop = state.coop_active;
        let move_dir = if is_key_down(KeyCode::W) || (!coop && is_key_down(KeyCode::Up)) {
            Some(Direction::Up)
        } else if is_key_down(KeyCode::S) || (!coop && is_key_down(KeyCode::Down)) {
            Some(Direction::Down)
        } else if is_key_down(KeyCode::A) || (!coop && is_key_down(KeyCode::Left)) {
            Some(Direction::Left)
        } else if is_key_down(KeyCode::D) || (!coop && is_key_down(KeyCode::Right)) {
            Some(Direction::Right)
        } else {
            move_cooldown = 0.0;
            None
        };

        // P2 movement: Arrow keys (only when co-op active)
        if coop && state.phase == GamePhase::Playing {
            static mut P2_COOLDOWN: f32 = 0.0;
            let p2_dir = if is_key_down(KeyCode::Up) || is_key_down(KeyCode::I) {
                Some(Direction::Up)
            } else if is_key_down(KeyCode::Down) || is_key_down(KeyCode::K) {
                Some(Direction::Down)
            } else if is_key_down(KeyCode::Left) || is_key_down(KeyCode::J) {
                Some(Direction::Left)
            } else if is_key_down(KeyCode::Right) || is_key_down(KeyCode::L) {
                Some(Direction::Right)
            } else {
                unsafe { P2_COOLDOWN = 0.0; }
                None
            };
            if let Some(d) = p2_dir {
                unsafe {
                    if P2_COOLDOWN <= 0.0 {
                        if state.mp_role == "guest" {
                            // Guest sends input to host
                            let dir_str = match d {
                                Direction::Up => "up", Direction::Down => "down",
                                Direction::Left => "left", Direction::Right => "right",
                            };
                            state.mp_send_input(dir_str);
                        } else {
                            state.move_player2(d);
                        }
                        P2_COOLDOWN = if state.riding_horse_p2 { MOVE_REPEAT_INTERVAL * 0.5 } else { MOVE_REPEAT_INTERVAL };
                    } else {
                        P2_COOLDOWN -= dt;
                    }
                }
            }
        }

        // P2 relationships toggle (/ or RightShift key)
        if coop && (is_key_pressed(KeyCode::Slash) || is_key_pressed(KeyCode::RightShift)) {
            state.show_relationships_p2 = !state.show_relationships_p2;
            state.show_relationships = false; // close P1's if P2 opens
        }

        // P2 actions (Enter = context-sensitive, covers all actions)
        if coop && state.phase == GamePhase::Playing
            && (is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::RightControl)
                || is_key_pressed(KeyCode::KpEnter) || is_key_pressed(KeyCode::Period)
                || is_key_pressed(KeyCode::N))
        {
            let facing = state.player2.facing_tile();
            let (p2col, p2row) = state.player2.tile;
            let season = state.clock.season.clone();
            let mut acted = false;

            // Check standing tile first
            // Bench — sit and rest
            if !acted {
                if let Some(tile) = state.map.get(p2col, p2row) {
                    if tile.kind == bennett_valley::game::world::TileKind::Bench {
                        state.player2.energy = (state.player2.energy + 20).min(state.player2.max_energy);
                        state.notify("P2 rests on the bench...");
                        acted = true;
                    }
                }
            }
            // Forage on standing tile
            if !acted {
                if let Some(tile) = state.map.get(p2col, p2row) {
                    if tile.kind == bennett_valley::game::world::TileKind::ForagePatch {
                        let cost = state.config.energy.forage_cost;
                        if farming::forage_tile(&mut state.map, &mut state.player2, p2col, p2row, cost, &season).is_ok() {
                            game_save::play_sound("harvest");
                            acted = true;
                        }
                    }
                }
            }
            // Fish on standing tile
            if !acted {
                if let Some(tile) = state.map.get(p2col, p2row) {
                    if tile.kind == bennett_valley::game::world::TileKind::FishingSpot {
                        if state.player2.spend_energy(state.config.energy.fish_cost) {
                            let fish = farming::season_fish_pub(&season, p2col, p2row);
                            state.fish_target = Some(fish.clone());
                            state.fish_bar = 0.5; state.fish_bar_vel = 0.0;
                            state.fish_pos = 0.5; state.fish_vel = 0.3;
                            state.fish_progress = 0.3;
                            state.fish_difficulty = match fish {
                                bennett_valley::game::inventory::FishKind::Bass => 0.6,
                                bennett_valley::game::inventory::FishKind::Catfish => 0.9,
                                bennett_valley::game::inventory::FishKind::Trout => 1.2,
                                bennett_valley::game::inventory::FishKind::Salmon => 1.5,
                            };
                            state.fish_player2 = true;
                            state.phase = GamePhase::FishingMinigame;
                            game_save::play_sound("fishCast");
                            acted = true;
                        }
                    }
                }
            }

            // Check facing tile
            if !acted {
                if let Some(tile) = state.map.get(facing.0, facing.1) {
                    let kind = tile.kind.clone();
                    match kind {
                        bennett_valley::game::world::TileKind::OakTree => {
                            // P2 forage oak tree for acorns
                            let cost = state.config.energy.forage_cost;
                            if farming::forage_oak(&mut state.map, &mut state.player2, facing.0, facing.1, cost, &season).is_ok() {
                                game_save::play_sound("harvest");
                                acted = true;
                            }
                        }
                        bennett_valley::game::world::TileKind::ShipBox => {
                            state.open_ship_select();
                            acted = true;
                        }
                        bennett_valley::game::world::TileKind::Shop => {
                            state.phase = GamePhase::ShopOpen;
                            acted = true;
                        }
                        bennett_valley::game::world::TileKind::Farmhouse => {
                            if let Some(bk) = bennett_valley::game::state::door_at(facing.0, facing.1) {
                                if bk == bennett_valley::game::state::BuildingKind::FurnitureShop {
                                    state.furniture_cursor = 0;
                                    state.phase = GamePhase::FurnitureShopOpen;
                                } else if bk == bennett_valley::game::state::BuildingKind::AnimalShop {
                                    state.animal_cursor = 0;
                                    state.phase = GamePhase::AnimalShopOpen;
                                } else if bk == bennett_valley::game::state::BuildingKind::Arcade {
                                    state.start_arcade();
                                } else if bk == bennett_valley::game::state::BuildingKind::Restaurant {
                                    state.open_restaurant();
                                } else if bk == bennett_valley::game::state::BuildingKind::IceCreamShop {
                                    state.open_icecream();
                                } else {
                                    state.current_building = bk;
                                    state.phase = GamePhase::FarmhouseInterior;
                                    state.farmhouse_tile = (5, 6);
                                }
                                acted = true;
                            }
                        }
                        _ => {}
                    }
                }
            }

            // Try farming actions on facing tile: harvest, mine, hoe, water, plant
            if !acted {
                // Harvest
                let crop_name = state.map.get(facing.0, facing.1)
                    .and_then(|t| t.crop.as_ref())
                    .map(|c| c.kind.name().to_string());
                if let Some(cn) = crop_name {
                    let grow_days = state.config.crops.get(&cn).map(|c| c.grow_days).unwrap_or(4);
                    let cost = state.config.energy.harvest_cost;
                    if farming::harvest_tile(&mut state.map, &mut state.player2, facing.0, facing.1, grow_days, cost).is_ok() {
                        game_save::play_sound("harvest");
                        acted = true;
                    }
                }
            }
            if !acted {
                // Mine — check facing tile for rock
                let is_rock = state.map.get(facing.0, facing.1)
                    .map(|t| t.kind.is_rock())
                    .unwrap_or(false);
                if is_rock {
                    let cost = state.config.energy.mine_cost;
                    let hp = state.config.ore.rock_hp;
                    match farming::mine_tile(&mut state.map, &mut state.player2, facing.0, facing.1, cost, hp) {
                        Ok(ore) => {
                            game_save::play_sound("mine");
                            if ore.is_some() { game_save::play_sound("rockBreak"); }
                            acted = true;
                        }
                        Err(e) => {
                            state.notify(action_error_msg(&e));
                            acted = true;
                        }
                    }
                }
            }
            if !acted {
                // Hoe
                let cost = state.config.energy.hoe_cost;
                if farming::hoe_tile(&mut state.map, &mut state.player2, facing.0, facing.1, cost).is_ok() {
                    game_save::play_sound("hoe");
                    acted = true;
                }
            }
            if !acted {
                // Water
                let cost = state.config.energy.water_cost;
                if farming::water_tile(&mut state.map, &mut state.player2, facing.0, facing.1, cost).is_ok() {
                    game_save::play_sound("water");
                    acted = true;
                }
            }
            if !acted {
                // Plant (use P1's selected seed)
                if let Some(seed) = state.player.selected_seed.clone() {
                    let kind = bennett_valley::game::state::seed_to_crop_kind_pub(&seed);
                    let cost = state.config.energy.plant_cost;
                    if farming::plant_seed(&mut state.map, &mut state.player2, facing.0, facing.1, kind, cost).is_ok() {
                        game_save::play_sound("plant");
                        acted = true;
                    }
                }
            }

            // If nothing worked and P2 has no energy, notify
            if !acted && state.player2.energy <= 0 {
                state.notify("P2 is out of energy!");
            }

            // P2 proposal — if P2 has pendant and facing marriageable NPC with 8+ hearts
            if !acted {
                let npc_idx = state.npcs.iter().position(|n| n.tile == facing);
                if let Some(idx) = npc_idx {
                    let npc = &state.npcs[idx];
                    let p2_friendship = *state.p2_friendships.get(&npc.id).unwrap_or(&0);
                    let p2_hearts = p2_friendship / 25;
                    let p2_gender = state.player2.gender;
                    let npc_g = bennett_valley::game::state::npc_gender(&npc.name);
                    if npc.marriageable
                        && p2_hearts >= 8
                        && npc_g != p2_gender
                        && state.married_npc_id_p2.is_none()
                        && state.player2.inventory.count(&bennett_valley::game::inventory::ItemKind::Pendant) > 0
                    {
                        state.player2.inventory.remove(&bennett_valley::game::inventory::ItemKind::Pendant, 1);
                        state.married_npc_id_p2 = Some(npc.id);
                        state.notify(&format!("P2 married {}!", npc.name));
                        game_save::play_sound("buy");
                        acted = true;
                    }
                }
            }

            // NPC interaction — P2 gains their own friendship
            if !acted {
                let npc_idx = state.npcs.iter().position(|n| n.tile == facing);
                if let Some(idx) = npc_idx {
                    let npc_id = state.npcs[idx].id;

                    // Try to gift best item from P2's inventory
                    let mut gifted = false;
                    if !state.npcs[idx].gifted_today {
                        let prefs: Vec<(String, i8)> = state.npcs[idx].gift_preferences.iter()
                            .map(|(k, &v)| (k.clone(), v)).collect();
                        // Find best item P2 has
                        let mut best: Option<(String, i8)> = None;
                        for (name, score) in &prefs {
                            if *score > 0 {
                                // Check if P2 has this item
                                let has = state.player2.inventory.items().iter().any(|(k, &qty)| {
                                    qty > 0 && bennett_valley::game::state::item_kind_to_name_pub(k) == name.as_str()
                                });
                                if has {
                                    if best.is_none() || score > &best.as_ref().unwrap().1 {
                                        best = Some((name.clone(), *score));
                                    }
                                }
                            }
                        }
                        if let Some((gift_name, _)) = best {
                            let pref = state.npcs[idx].give_gift_by_name(&gift_name);
                            let gain = match pref { p if p >= 2 => 30u8, 1 => 15, _ => 5 };
                            let current = *state.p2_friendships.get(&npc_id).unwrap_or(&0);
                            let actual = gain.min(250u8.saturating_sub(current));
                            *state.p2_friendships.entry(npc_id).or_insert(0) += actual;
                            state.notify(&format!("P2 gifted {} to {}!", gift_name, state.npcs[idx].name));
                            gifted = true;
                        }
                    }

                    if !gifted {
                        // Just talk
                        let current = *state.p2_friendships.get(&npc_id).unwrap_or(&0);
                        let gain = 5u8.min(250u8.saturating_sub(current));
                        if gain > 0 {
                            *state.p2_friendships.entry(npc_id).or_insert(0) += gain;
                        }
                        state.notify(&format!("P2 talked to {}!", state.npcs[idx].name));
                    }

                    // Check if P2 triggered Victor win
                    let p2_victor_f = *state.p2_friendships.get(&bennett_valley::game::npc::VICTOR_ID).unwrap_or(&0);
                    if npc_id == bennett_valley::game::npc::VICTOR_ID
                        && p2_victor_f / 25 >= bennett_valley::game::npc::VICTOR_FINAL_HEARTS
                    {
                        let victor_name = state.npcs[idx].name.clone();
                        let url = format!(
                            "/api/victor_final?npc={}&season={}&day={}",
                            victor_name, state.clock.season.name(), state.clock.day
                        );
                        state.waiting_npc_name = Some(victor_name);
                        state.waiting_npc_idx = Some(idx);
                        state.is_victor_final = true;
                        state.pending_llm = Some(bennett_valley::game::state::LlmRequest { npc_idx: idx, url });
                        state.phase = GamePhase::LlmWaiting;
                    }

                    acted = true;
                }
            }
        }

        if let Some(d) = move_dir {
            // Manual movement cancels horse autopilot
            if state.horse_target.is_some() && state.phase == GamePhase::Playing {
                state.horse_target = None;
                state.horse_path.clear();
                state.notify("Autopilot cancelled.");
            }
            if move_cooldown <= 0.0 {
                if state.phase == GamePhase::Playing {
                    state.move_player(d);
                    camera.set_target(state.player.tile.0, state.player.tile.1, sw, sh);
                } else if state.phase == GamePhase::FarmhouseInterior {
                    state.move_in_farmhouse(d);
                } else if state.phase == GamePhase::ArenaEditor {
                    let (dx, dy) = match d {
                        Direction::Up    => (0, -1),
                        Direction::Down  => (0, 1),
                        Direction::Left  => (-1, 0),
                        Direction::Right => (1, 0),
                    };
                    state.arena_move_cursor(dx, dy);
                } else if state.phase == GamePhase::FestivalPlaying {
                    let (dx, dy) = match d {
                        Direction::Up    => (0, -1),
                        Direction::Down  => (0, 1),
                        Direction::Left  => (-1, 0),
                        Direction::Right => (1, 0),
                    };
                    state.festival_move(dx, dy);
                }
                move_cooldown = if state.riding_horse { MOVE_REPEAT_INTERVAL * 0.5 } else { MOVE_REPEAT_INTERVAL };
            } else {
                move_cooldown -= dt;
            }
        }

        // ── Horse destination: A/D for column switching ──────────────────────
        if state.phase == GamePhase::HorseDestination {
            let len = GameState::horse_destinations().len();
            let rpc = (len + 1) / 2;
            if is_key_pressed(KeyCode::A) || is_key_pressed(KeyCode::Left) {
                let col_idx = state.horse_dest_cursor / rpc;
                let row_idx = state.horse_dest_cursor % rpc;
                if col_idx > 0 {
                    state.horse_dest_cursor = row_idx; // jump to left column, same row
                }
            }
            if is_key_pressed(KeyCode::D) || is_key_pressed(KeyCode::Right) {
                let col_idx = state.horse_dest_cursor / rpc;
                let row_idx = state.horse_dest_cursor % rpc;
                if col_idx == 0 {
                    let new_idx = rpc + row_idx;
                    if new_idx < len {
                        state.horse_dest_cursor = new_idx;
                    }
                }
            }
        }

        // ── Action inputs ─────────────────────────────────────────────────────
        for event in collect_action_events() {
            match event {
                InputEvent::Hoe => {
                    if state.phase == GamePhase::OutfitShopOpen && state.player.gender == 1 {
                        // H = cycle hairstyle when female in wardrobe
                        let total = bennett_valley::game::state::HAIRSTYLES.len() as u8;
                        state.player.hairstyle = (state.player.hairstyle + 1) % total;
                        let name = bennett_valley::game::state::HAIRSTYLES[state.player.hairstyle as usize];
                        state.notify(&format!("Hairstyle: {}", name));
                    } else {
                        match state.try_hoe() {
                            Ok(_) => game_save::play_sound("hoe"),
                            Err(e) => state.notify(action_error_msg(&e)),
                        }
                    }
                }
                InputEvent::Water => {
                    match state.try_water() {
                        Ok(_) => game_save::play_sound("water"),
                        Err(e) => state.notify(action_error_msg(&e)),
                    }
                }
                InputEvent::Plant => {
                    match state.try_plant() {
                        Ok(_) => game_save::play_sound("plant"),
                        Err(e) => state.notify(action_error_msg(&e)),
                    }
                }
                InputEvent::Harvest => {
                    match state.try_harvest() {
                        Ok(_) => game_save::play_sound("harvest"),
                        Err(e) => state.notify(action_error_msg(&e)),
                    }
                }
                InputEvent::Forage => {
                    if state.phase == GamePhase::OutfitShopOpen {
                        state.toggle_gender();
                    } else {
                        match state.try_forage() {
                            Ok(_) => game_save::play_sound("harvest"),
                            Err(e) => state.notify(action_error_msg(&e)),
                        }
                    }
                }
                InputEvent::Fish => {
                    if state.phase == GamePhase::OutfitShopOpen && state.player.gender == 1 {
                        // C = cycle hair color when female in wardrobe
                        let total = bennett_valley::game::state::HAIR_COLORS.len() as u8;
                        state.player.hair_color = (state.player.hair_color + 1) % total;
                        let name = bennett_valley::game::state::HAIR_COLORS[state.player.hair_color as usize].3;
                        state.notify(&format!("Hair color: {}", name));
                    } else {
                        match state.try_fish() {
                            Ok(_) => game_save::play_sound("fishCast"),
                            Err(e) => state.notify(action_error_msg(&e)),
                        }
                    }
                }
                InputEvent::Mine => {
                    match state.try_mine() {
                        Ok(_) => game_save::play_sound("mine"),
                        Err(e) => state.notify(action_error_msg(&e)),
                    }
                }
                InputEvent::Gift => {
                    if state.phase == GamePhase::Playing {
                        state.try_gift();
                    }
                }
                InputEvent::ToggleRelationships => {
                    if state.phase == GamePhase::Playing {
                        state.show_relationships = !state.show_relationships;
                        state.show_relationships_p2 = false; // close P2's if P1 opens
                    }
                }
                InputEvent::Sleep => {
                    let can_sleep = state.phase == GamePhase::Playing
                        || (state.phase == GamePhase::FarmhouseInterior
                            && state.current_building == bennett_valley::game::state::BuildingKind::Farmhouse);
                    if can_sleep {
                        state.advance_day();

                        // Wake up inside the farmhouse next to the bed
                        state.current_building = bennett_valley::game::state::BuildingKind::Farmhouse;
                        state.phase = GamePhase::FarmhouseInterior;
                        state.farmhouse_tile = (8, 3); // next to P1 bed
                        state.player.facing = bennett_valley::game::player::Direction::Left;
                        // Place P1 on the map at the farmhouse door for when they exit
                        state.player.tile = (2, 4);
                        if state.coop_active {
                            state.player2.tile = (3, 4);
                        }
                    }
                }
                InputEvent::ResetGame => {
                    // Only open confirm from states where it makes sense to interrupt.
                    match state.phase {
                        GamePhase::Playing | GamePhase::FarmhouseInterior
                        | GamePhase::DaySummary | GamePhase::Won => {
                            state.phase = GamePhase::ResetConfirm;
                        }
                        _ => {}
                    }
                }
                InputEvent::Reply => {
                    if state.phase == GamePhase::LetterOpen {
                        state.start_letter_reply();
                    }
                }
                InputEvent::Interact => match state.phase {
                    GamePhase::Playing        => {
                        if state.riding_horse {
                            state.open_horse_destinations();
                        } else {
                            state.try_interact();
                        }
                    }
                    GamePhase::DialogueOpen   => state.advance_dialogue(),
                    GamePhase::DialogueChoice => { state.confirm_choice(); }
                    GamePhase::DaySummary     => state.dismiss_summary(),
                    GamePhase::LlmWaiting     => {} // wait for NPC response
                    GamePhase::LetterWaiting  => {} // wait for letter response
                    GamePhase::LetterOpen     => state.dismiss_letter(),
                    GamePhase::LetterReply    => state.confirm_letter_reply(),
                    GamePhase::ShopOpen       => {
                        if state.shop_try_buy().is_ok() {
                            game_save::play_sound("buy");
                        } else {
                            state.notify("Can't afford that!");
                        }
                    }
                    GamePhase::FarmhouseInterior => {
                        if state.current_building == bennett_valley::game::state::BuildingKind::Farmhouse {
                            state.advance_day();
    
                        }
                    }
                    GamePhase::Won => {
                        // Win screen — press E to return to free play.
                        state.phase = GamePhase::Playing;
                    }
                    GamePhase::ResetConfirm => {
                        // E confirms reset: wipe save, reinitialize state.
                        game_save::platform_clear();
                        state = GameState::new(config.clone());
                        state.snap_npcs();
                    }
                    GamePhase::ShipSelect => {
                        if state.ship_cursor >= state.ship_manifest.len() {
                            state.ship_confirm();
                        } else {
                            state.ship_toggle();
                        }
                    }
                    GamePhase::FurnitureShopOpen => {
                        state.furniture_try_buy();
                    }
                    GamePhase::AnimalShopOpen => {
                        state.animal_try_buy();
                    }
                    GamePhase::OutfitShopOpen => {
                        state.outfit_try_buy();
                    }
                    // (G key for gender toggle is handled below)
                    GamePhase::RestaurantOpen => {
                        state.restaurant_order();
                    }
                    GamePhase::IceCreamShopOpen => {
                        state.icecream_order();
                    }
                    GamePhase::ArcadePlaying => {
                        if state.arcade_phase == 2 {
                            // Play again
                            state.start_arcade();
                        } else {
                            state.arcade_react();
                        }
                    }
                    GamePhase::ArenaEditor => {
                        state.arena_toggle_jump();
                    }
                    GamePhase::FishingMinigame => {} // handled by tick_fishing
                    GamePhase::FestivalAnnounce => {
                        state.festival_start();
                    }
                    GamePhase::FestivalPlaying => {
                        state.festival_search();
                    }
                    GamePhase::FestivalResults => {
                        state.festival_dismiss();
                    }
                    GamePhase::HorseDestination => {
                        state.confirm_horse_destination();
                    }
                },
                InputEvent::CloseMenu => {
                    if state.phase == GamePhase::HorseDestination {
                        state.phase = GamePhase::Playing;
                    } else if state.phase == GamePhase::ResetConfirm {
                        state.phase = GamePhase::Playing;
                    } else if state.show_relationships { state.show_relationships = false; }
                    else if state.show_relationships_p2 { state.show_relationships_p2 = false; }
                    else if state.phase == GamePhase::LetterReply { state.phase = GamePhase::LetterOpen; }
                    else if state.phase == GamePhase::RestaurantOpen { state.restaurant_close(); }
                    else if state.phase == GamePhase::IceCreamShopOpen { state.icecream_close(); }
                    else if state.phase == GamePhase::ArcadePlaying { state.arcade_close(); }
                    else if state.phase == GamePhase::ArenaEditor { state.close_arena_editor(); }
                    else if state.phase == GamePhase::FishingMinigame {
                        state.fish_target = None;
                        state.phase = GamePhase::Playing;
                        state.notify("You stopped fishing.");
                    }
                    else if state.phase == GamePhase::OutfitShopOpen { state.outfit_close(); }
                    else if state.phase == GamePhase::AnimalShopOpen { state.animal_close(); }
                    else if state.phase == GamePhase::FestivalPlaying { state.festival_end(); }
                    else if state.phase == GamePhase::FurnitureShopOpen { state.furniture_close(); }
                    else if state.phase == GamePhase::ShipSelect { state.ship_cancel(); }
                    else if state.phase == GamePhase::ShopOpen { state.close_shop(); }
                    else if state.phase == GamePhase::FarmhouseInterior { state.exit_farmhouse(); }
                }
                InputEvent::ShopUp => {
                    if state.phase == GamePhase::ShopOpen {
                        state.shop_move_cursor(-1);
                    } else if state.phase == GamePhase::ShipSelect {
                        state.ship_move_cursor(-1);
                    } else if state.phase == GamePhase::FurnitureShopOpen {
                        state.furniture_move_cursor(-1);
                    } else if state.phase == GamePhase::AnimalShopOpen {
                        state.animal_move_cursor(-1);
                    } else if state.phase == GamePhase::OutfitShopOpen {
                        state.outfit_move_cursor(-1);
                    } else if state.phase == GamePhase::RestaurantOpen {
                        state.restaurant_move_cursor(-1);
                    } else if state.phase == GamePhase::IceCreamShopOpen {
                        state.icecream_move_cursor(-1);
                    } else if state.phase == GamePhase::FestivalPlaying {
                        state.festival_move(0, -1);
                    } else if state.phase == GamePhase::DialogueChoice {
                        state.move_choice(-1);
                    } else if state.phase == GamePhase::LetterReply {
                        state.move_reply_cursor(-1);
                    } else if state.phase == GamePhase::HorseDestination {
                        let len = GameState::horse_destinations().len();
                        let rpc = (len + 1) / 2; // rows per column
                        let col_idx = state.horse_dest_cursor / rpc;
                        let row_idx = state.horse_dest_cursor % rpc;
                        let new_row = if row_idx == 0 { rpc - 1 } else { row_idx - 1 };
                        let new_idx = col_idx * rpc + new_row;
                        if new_idx < len { state.horse_dest_cursor = new_idx; }
                    }
                }
                InputEvent::ShopDown => {
                    if state.phase == GamePhase::ShopOpen {
                        state.shop_move_cursor(1);
                    } else if state.phase == GamePhase::ShipSelect {
                        state.ship_move_cursor(1);
                    } else if state.phase == GamePhase::FurnitureShopOpen {
                        state.furniture_move_cursor(1);
                    } else if state.phase == GamePhase::AnimalShopOpen {
                        state.animal_move_cursor(1);
                    } else if state.phase == GamePhase::OutfitShopOpen {
                        state.outfit_move_cursor(1);
                    } else if state.phase == GamePhase::RestaurantOpen {
                        state.restaurant_move_cursor(1);
                    } else if state.phase == GamePhase::IceCreamShopOpen {
                        state.icecream_move_cursor(1);
                    } else if state.phase == GamePhase::FestivalPlaying {
                        state.festival_move(0, 1);
                    } else if state.phase == GamePhase::DialogueChoice {
                        state.move_choice(1);
                    } else if state.phase == GamePhase::LetterReply {
                        state.move_reply_cursor(1);
                    } else if state.phase == GamePhase::HorseDestination {
                        let len = GameState::horse_destinations().len();
                        let rpc = (len + 1) / 2;
                        let col_idx = state.horse_dest_cursor / rpc;
                        let row_idx = state.horse_dest_cursor % rpc;
                        let new_row = (row_idx + 1) % rpc;
                        let new_idx = col_idx * rpc + new_row;
                        if new_idx < len { state.horse_dest_cursor = new_idx; }
                    }
                }
                InputEvent::SelectSeedParsnip     => { state.player.selected_seed = Some(CropSeed::Parsnip); }
                InputEvent::SelectSeedPotato      => { state.player.selected_seed = Some(CropSeed::Potato); }
                InputEvent::SelectSeedCauliflower => { state.player.selected_seed = Some(CropSeed::Cauliflower); }
                InputEvent::P2Action => {} // handled in P2 action block above via is_key_pressed(N)
                InputEvent::ToggleHorse => {
                    if state.phase == GamePhase::Playing {
                        let was_riding = state.riding_horse;
                        state.toggle_horse();
                        if state.riding_horse != was_riding {
                            game_save::play_sound("horse");
                        }
                    }
                }
                InputEvent::HorseLeap => {
                    if state.phase == GamePhase::Playing {
                        state.horse_leap();
                        if state.horse_leap_timer > 0.0 {
                            game_save::play_sound("leap");
                        }
                        camera.set_target(state.player.tile.0, state.player.tile.1, sw, sh);
                    }
                }
                InputEvent::MpHost => {
                    if state.phase == GamePhase::Playing && state.mp_role == "none" {
                        state.mp_host();
                    }
                }
                InputEvent::MpJoin => {
                    if state.phase == GamePhase::Playing && state.mp_role == "none" {
                        // For now, prompt is hardcoded. In a real game you'd have a text input.
                        // The user needs to type the code. Let's use a simple approach:
                        // join the room that was most recently created (for testing).
                        state.notify("Enter room code in browser console: bvJoin('CODE')");
                    }
                }
                InputEvent::ToggleCoop => {
                    if state.phase == GamePhase::Playing {
                        state.coop_active = !state.coop_active;
                        if state.coop_active {
                            state.player2.tile = (state.player.tile.0 + 1, state.player.tile.1);
                            state.player2.energy = state.player.max_energy;
                            state.notify("Co-op ON! P2: Arrow Keys");
                        } else {
                            state.notify("Co-op OFF");
                        }
                    }
                }
                InputEvent::OpenOutfits => {
                    if state.phase == GamePhase::Playing {
                        state.outfit_cursor = state.player.outfit as usize;
                        state.phase = GamePhase::OutfitShopOpen;
                    }
                }
                InputEvent::Propose => {
                    if state.phase == GamePhase::Playing {
                        state.try_propose();
                    }
                }
                InputEvent::Scythe => {
                    if state.phase == GamePhase::Playing {
                        match state.try_scythe() {
                            Ok(_) => game_save::play_sound("hoe"),
                            Err(e) => state.notify(action_error_msg(&e)),
                        }
                    }
                }
                InputEvent::ToggleMusic => {
                    state.music_track = game_save::toggle_music();
                    let name = match state.music_track {
                        1 => "Guqin Garden",
                        2 => "Valley Pop",
                        _ => "Off",
                    };
                    state.notify(&format!("Music: {}", name));
                }
                InputEvent::Move(_) => {}
            }
        }

        // ── LLM request: launch coroutine when state signals one ──────────────
        if let Some(req) = state.pending_llm.take() {
            let npc_idx = req.npc_idx;
            let url = req.url;
            #[cfg(target_arch = "wasm32")]
            {
                use macroquad::experimental::coroutines::start_coroutine;
                start_coroutine(async move {
                    let text = load_string(&url).await
                        .unwrap_or_else(|_| "*smiles at you*".to_string());
                    LLM_RESULT.with(|r| *r.borrow_mut() = Some((npc_idx, text)));
                });
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                // Native build: inject placeholder immediately (no HTTP available)
                LLM_RESULT.with(|r| *r.borrow_mut() = Some((npc_idx,
                    "Hello there, neighbour!".to_string())));
            }
        }

        // ── LLM result: convert to dialogue when coroutine finishes ───────────
        let llm_done = LLM_RESULT.with(|r| r.borrow_mut().take());
        if let Some((npc_idx, text)) = llm_done {
            if state.phase == GamePhase::LlmWaiting {
                let npc_name = state.npcs.get(npc_idx)
                    .map(|n| n.name.clone())
                    .unwrap_or_default();

                if state.is_followup_dialogue {
                    // Follow-up reaction: plain text, no choice options
                    state.response_options.clear();
                    state.dialogue = Some(DialogueState::new(npc_name, vec![text]));
                    state.phase = GamePhase::DialogueOpen;
                } else {
                    // Initial greeting: try to parse JSON with npc_line + options
                    #[cfg(target_arch = "wasm32")]
                    let parsed = parse_npc_json(&text);
                    #[cfg(not(target_arch = "wasm32"))]
                    let parsed = None::<(String, Vec<String>)>;

                    if let Some((npc_line, options)) = parsed {
                        state.response_options = options;
                        state.dialogue = Some(DialogueState::new(npc_name, vec![npc_line]));
                    } else {
                        state.response_options.clear();
                        state.dialogue = Some(DialogueState::new(npc_name, vec![text]));
                    }
                    state.phase = GamePhase::DialogueOpen;
                }
            }
        }

        // ── Letter request: launch coroutine when state signals one ──────────
        if let Some(req) = state.pending_letter.take() {
            let url = req.url.clone();
            let friend_name = req.friend_name.clone();
            #[cfg(target_arch = "wasm32")]
            {
                use macroquad::experimental::coroutines::start_coroutine;
                start_coroutine(async move {
                    let text = load_string(&url).await
                        .unwrap_or_else(|_| "Hope you're doing well out there.".to_string());
                    LLM_LETTER_RESULT.with(|r| *r.borrow_mut() = Some(text));
                });
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                let _ = friend_name;
                LLM_LETTER_RESULT.with(|r| *r.borrow_mut() = Some(
                    "Life in the city is dull without you around. Hope the farm treats you well!".to_string()
                ));
            }
            // Store friend name so we can display it when the result arrives.
            state.current_letter = Some((req.friend_name, String::new()));
        }

        // ── Letter result: show overlay when coroutine finishes ───────────────
        let letter_done = LLM_LETTER_RESULT.with(|r| r.borrow_mut().take());
        if let Some(text) = letter_done {
            if state.phase == GamePhase::LetterWaiting {
                // current_letter already has (friend_name, ""); fill in text.
                if let Some((name, body)) = &mut state.current_letter {
                    *body = text;
                    let _ = name; // already set
                }
                state.phase = GamePhase::LetterOpen;
            }
        }

        // ── Memory save: fire-and-forget after dialogue closes ───────────────
        if let Some(mem) = state.pending_memory.take() {
            let url = format!("/api/memory?npc_id={}&text={}", mem.npc_id, url_encode(&mem.text));
            #[cfg(target_arch = "wasm32")]
            {
                use macroquad::experimental::coroutines::start_coroutine;
                start_coroutine(async move { let _ = load_string(&url).await; });
            }
        }

        // ── Notification timer ────────────────────────────────────────────────
        state.tick_notification(dt);

        // ── Camera: split-screen when co-op, normal otherwise ─────────────────
        if state.coop_active && state.phase == GamePhase::Playing {
            camera.set_target(state.player.tile.0, state.player.tile.1, sw, sh);
            camera2.set_target(state.player2.tile.0, state.player2.tile.1, sw, sh);
        }
        camera.update(dt);
        camera2.update(dt);

        // ── Time tick ─────────────────────────────────────────────────────────
        tick_acc += dt;
        if tick_acc >= TICK_SECS {
            tick_acc -= TICK_SECS;
            state.tick_time();
        }

        state.tick_npc_movement(dt);
        state.tick_birds(dt);
        state.tick_squirrels(dt);
        state.tick_horse_leap(dt);
        state.tick_arcade(dt);
        state.mp_tick(dt);

        // Check for ride_to command from JS console
        {
            let cmd = game_save::read_ride_cmd();
            if !cmd.is_empty() {
                state.ride_to(&cmd);
            }
        }

        // Horse autopilot — move toward target every 0.12s
        if state.horse_target.is_some() && state.phase == GamePhase::Playing {
            horse_autopilot_timer += dt;
            if horse_autopilot_timer >= 0.12 {
                horse_autopilot_timer = 0.0;
                if state.horse_autopilot_step() {
                    game_save::play_sound("step");
                    camera.set_target(state.player.tile.0, state.player.tile.1, sw, sh);
                }
            }
        } else {
            horse_autopilot_timer = 0.0;
        }

        // Fishing minigame — reel key depends on who is fishing
        if state.phase == GamePhase::FishingMinigame {
            let reel = if state.fish_player2 {
                is_key_down(KeyCode::Up) || is_key_down(KeyCode::I) // P2 uses Up arrow or I
            } else {
                is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) // P1 uses W or Up
            };
            state.tick_fishing(dt, reel);
        }

        // ── Auto-save ─────────────────────────────────────────────────────────
        if state.pending_save {
            state.pending_save = false;
            if let Ok(json) = serde_json::to_string(&state.to_save()) {
                game_save::platform_save(&json);
            }
        }

        // ── Render ────────────────────────────────────────────────────────────
        clear_background(Color::from_hex(0x2d5a27));
        if state.coop_active {
            renderer::draw_coop(&state, &camera, &camera2);
        } else {
            renderer::draw(&state, &camera);
        }

        next_frame().await;
    }
}
