use serde::{Deserialize, Serialize};
use crate::game::inventory::{CropSeed, FishKind, ForageKind, HarvestedCrop, ItemKind, OreKind};
use crate::game::time::Season;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NpcSave {
    pub id: u8,
    pub friendship: u8,
    pub loved_gift_given: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SaveData {
    // Clock
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub season: String,
    pub year: u16,
    // Player
    pub tile: [usize; 2],
    pub energy: i16,
    pub max_energy: i16,
    pub gold: u32,
    pub hoe_level: u8,
    pub can_level: u8,
    pub charisma_xp: u32,
    pub selected_seed: Option<String>,
    pub inventory: Vec<(String, u32)>,
    // NPCs
    pub npcs: Vec<NpcSave>,
    // Other
    pub pending_gold: u32,
    pub ships_today: u32,
    pub married_npc_id: Option<u8>,
    #[serde(default)]
    pub house_upgraded: bool,
    #[serde(default)]
    pub owned_furniture: Vec<String>,
    #[serde(default)]
    pub owned_animals: Vec<String>,
    #[serde(default)]
    pub has_equestrian_center: bool,
    #[serde(default)]
    pub arena_jumps: Vec<(u8, u8, u8)>,
    #[serde(default)]
    pub outfit: u8,
    #[serde(default)]
    pub gender: u8,
    #[serde(default)]
    pub hairstyle: u8,
    #[serde(default)]
    pub hair_color: u8,
    #[serde(default)]
    pub owned_outfits: Vec<u8>,
}

pub fn item_to_key(item: &ItemKind) -> String {
    match item {
        ItemKind::Seed(s)   => format!("seed:{}", s.name()),
        ItemKind::Crop(c)   => format!("crop:{}", c.name()),
        ItemKind::Forage(f) => format!("forage:{}", f.name()),
        ItemKind::Fish(f)   => format!("fish:{}", f.name()),
        ItemKind::Ore(o)    => format!("ore:{}", o.name()),
        ItemKind::Pendant   => "pendant".to_string(),
        ItemKind::Egg       => "egg".to_string(),
        ItemKind::Milk      => "milk".to_string(),
        ItemKind::Fiber     => "fiber".to_string(),
    }
}

pub fn item_from_key(key: &str) -> Option<ItemKind> {
    if key == "pendant" { return Some(ItemKind::Pendant); }
    if key == "egg" { return Some(ItemKind::Egg); }
    if key == "milk" { return Some(ItemKind::Milk); }
    if key == "fiber" { return Some(ItemKind::Fiber); }
    let (prefix, name) = key.split_once(':')?;
    match prefix {
        "seed" => Some(ItemKind::Seed(match name {
            "parsnip"     => CropSeed::Parsnip,
            "potato"      => CropSeed::Potato,
            "cauliflower" => CropSeed::Cauliflower,
            "melon"       => CropSeed::Melon,
            "blueberry"   => CropSeed::Blueberry,
            "tomato"      => CropSeed::Tomato,
            "pumpkin"     => CropSeed::Pumpkin,
            "yam"         => CropSeed::Yam,
            "cranberry"   => CropSeed::Cranberry,
            _ => return None,
        })),
        "crop" => Some(ItemKind::Crop(match name {
            "parsnip"     => HarvestedCrop::Parsnip,
            "potato"      => HarvestedCrop::Potato,
            "cauliflower" => HarvestedCrop::Cauliflower,
            "melon"       => HarvestedCrop::Melon,
            "blueberry"   => HarvestedCrop::Blueberry,
            "tomato"      => HarvestedCrop::Tomato,
            "pumpkin"     => HarvestedCrop::Pumpkin,
            "yam"         => HarvestedCrop::Yam,
            "cranberry"   => HarvestedCrop::Cranberry,
            _ => return None,
        })),
        "forage" => Some(ItemKind::Forage(match name {
            "mushroom" => ForageKind::Mushroom,
            "berry"    => ForageKind::Berry,
            "herb"     => ForageKind::Herb,
            "flower"   => ForageKind::Flower,
            "fern"     => ForageKind::Fern,
            "acorn"    => ForageKind::Acorn,
            _ => return None,
        })),
        "fish" => Some(ItemKind::Fish(match name {
            "bass"    => FishKind::Bass,
            "catfish" => FishKind::Catfish,
            "trout"   => FishKind::Trout,
            "salmon"  => FishKind::Salmon,
            _ => return None,
        })),
        "ore" => Some(ItemKind::Ore(match name {
            "copper" => OreKind::Copper,
            "iron"   => OreKind::Iron,
            "gold"   => OreKind::Gold,
            _ => return None,
        })),
        _ => None,
    }
}

pub fn seed_from_name(name: &str) -> Option<CropSeed> {
    match name {
        "parsnip"     => Some(CropSeed::Parsnip),
        "potato"      => Some(CropSeed::Potato),
        "cauliflower" => Some(CropSeed::Cauliflower),
        "melon"       => Some(CropSeed::Melon),
        "blueberry"   => Some(CropSeed::Blueberry),
        "tomato"      => Some(CropSeed::Tomato),
        "pumpkin"     => Some(CropSeed::Pumpkin),
        "yam"         => Some(CropSeed::Yam),
        "cranberry"   => Some(CropSeed::Cranberry),
        _ => None,
    }
}

pub fn season_from_name(name: &str) -> Season {
    match name {
        "Summer" => Season::Summer,
        "Fall"   => Season::Fall,
        "Winter" => Season::Winter,
        _        => Season::Spring,
    }
}

// ── Platform save/load ────────────────────────────────────────────────────────

/// Write save data string to platform storage (localStorage on WASM, file on native).
pub fn platform_save(json: &str) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        bv_save_game(json.as_ptr(), json.len());
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = std::fs::write("save.json", json);
    }
}

/// Read save data string from platform storage. Returns None if no save exists.
pub fn platform_load() -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        let mut buf = vec![0u8; 131072]; // 128 KB max
        let len = unsafe { bv_load_game(buf.as_mut_ptr(), buf.len()) };
        if len == 0 {
            return None;
        }
        String::from_utf8(buf[..len].to_vec()).ok()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::fs::read_to_string("save.json").ok()
    }
}

/// Delete the save from platform storage.
pub fn platform_clear() {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        bv_clear_save();
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = std::fs::remove_file("save.json");
    }
}

// WASM imports — implemented in index.html via miniquad_add_plugin.
#[cfg(target_arch = "wasm32")]
extern "C" {
    fn bv_save_game(ptr: *const u8, len: usize);
    fn bv_load_game(ptr: *mut u8, max_len: usize) -> usize;
    fn bv_clear_save();
    fn bv_play_sound(ptr: *const u8, len: usize);
    fn bv_toggle_music() -> u32;
    fn bv_read_ride_cmd(buf_ptr: *mut u8, buf_max: usize) -> usize;
    fn bv_mp_create(buf_ptr: *mut u8, buf_max: usize) -> usize;
    fn bv_mp_join(ptr: *const u8, len: usize, buf_ptr: *mut u8, buf_max: usize) -> usize;
    fn bv_mp_sync(ptr: *const u8, len: usize);
    fn bv_mp_send_input(ptr: *const u8, len: usize);
    fn bv_mp_read(buf_ptr: *mut u8, buf_max: usize) -> usize;
    fn bv_mp_role(buf_ptr: *mut u8, buf_max: usize) -> usize;
}

/// Create a multiplayer room (host). Returns room code or empty string.
pub fn mp_create_room() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        let mut buf = [0u8; 16];
        let len = unsafe { bv_mp_create(buf.as_mut_ptr(), buf.len()) };
        return String::from_utf8_lossy(&buf[..len]).to_string();
    }
    #[cfg(not(target_arch = "wasm32"))]
    String::new()
}

/// Join a multiplayer room (guest). Returns "ok" or "fail".
pub fn mp_join_room(code: &str) -> String {
    #[cfg(target_arch = "wasm32")]
    {
        let mut buf = [0u8; 16];
        let len = unsafe { bv_mp_join(code.as_ptr(), code.len(), buf.as_mut_ptr(), buf.len()) };
        return String::from_utf8_lossy(&buf[..len]).to_string();
    }
    #[cfg(not(target_arch = "wasm32"))]
    String::new()
}

/// Host sends state snapshot to relay server.
pub fn mp_sync_state(json: &str) {
    #[cfg(target_arch = "wasm32")]
    unsafe { bv_mp_sync(json.as_ptr(), json.len()); }
}

/// Guest sends input event to relay server.
pub fn mp_send_input(json: &str) {
    #[cfg(target_arch = "wasm32")]
    unsafe { bv_mp_send_input(json.as_ptr(), json.len()); }
}

/// Read data from relay: guest gets state, host gets inputs.
pub fn mp_read() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        let mut buf = vec![0u8; 32768]; // 32KB buffer
        let len = unsafe { bv_mp_read(buf.as_mut_ptr(), buf.len()) };
        return String::from_utf8_lossy(&buf[..len]).to_string();
    }
    #[cfg(not(target_arch = "wasm32"))]
    String::new()
}

/// Get current multiplayer role: "host", "guest", or "none".
pub fn mp_role() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        let mut buf = [0u8; 16];
        let len = unsafe { bv_mp_role(buf.as_mut_ptr(), buf.len()) };
        return String::from_utf8_lossy(&buf[..len]).to_string();
    }
    #[cfg(not(target_arch = "wasm32"))]
    "none".to_string()
}

/// Toggle background music. Returns 0=off, 1=guqin, 2=pop.
pub fn toggle_music() -> u32 {
    #[cfg(target_arch = "wasm32")]
    { return unsafe { bv_toggle_music() }; }
    #[cfg(not(target_arch = "wasm32"))]
    0
}

/// Read and clear the pending ride command from JS. Returns empty string if none.
pub fn read_ride_cmd() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        let mut buf = [0u8; 64];
        let len = unsafe { bv_read_ride_cmd(buf.as_mut_ptr(), buf.len()) };
        if len == 0 { return String::new(); }
        return String::from_utf8_lossy(&buf[..len]).to_string();
    }
    #[cfg(not(target_arch = "wasm32"))]
    String::new()
}

/// Play a named sound effect. Sound names: step, hoe, water, plant, harvest,
/// mine, rockBreak, fishCast, fishCatch, fishFail, buy, notify, door, horse, leap, egg.
pub fn play_sound(name: &str) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        bv_play_sound(name.as_ptr(), name.len());
    }
}
