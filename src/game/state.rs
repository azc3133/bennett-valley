use crate::game::{
    config::GameConfig,
    crop::CropKind,
    dialogue::DialogueState,
    farming::{self, ActionError},
    inventory::{CropSeed, FishKind, HarvestedCrop, ItemKind, OreKind, PlayerInventory},
    npc::NPC,
    player::{Direction, Player},
    save::{NpcSave, SaveData, item_from_key, item_to_key, season_from_name, seed_from_name},
    shop::{make_default_shop, ShopInventory},
    time::{GameClock, TimeEvent},
    world::FarmMap,
};

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

#[derive(Debug, Clone)]
pub struct DaySummary {
    pub day: u8,
    pub gold_earned: u32,
    pub crops_shipped: u32,
    /// Set to the season name that just ended when a season transition occurred.
    pub season_ended: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GamePhase {
    Playing,
    ShopOpen,
    DialogueOpen,
    DialogueChoice,  // NPC greeted player; player picks one of 3 responses
    LlmWaiting,      // NPC is "thinking"; blocks all input until response arrives
    LetterWaiting,   // fetching an old-friend letter from the LLM
    LetterOpen,      // letter ready to read
    LetterReply,     // player choosing a reply topic
    DaySummary,
    Won, // Victor reached 5 hearts and delivered his final speech
    FarmhouseInterior,
    ResetConfirm,    // confirmation overlay before wiping save
    ShipSelect,      // player choosing which items to ship
    FurnitureShopOpen, // browsing the furniture shop
    AnimalShopOpen,    // browsing the animal shop
    OutfitShopOpen,    // browsing the outfit shop
    RestaurantOpen,    // ordering food at the restaurant
    ArcadePlaying,     // playing an arcade mini-game
    ArenaEditor,       // placing jumps in the riding arena
    FishingMinigame,   // active fishing minigame
    FestivalAnnounce,  // announcing today's festival
    FestivalPlaying,   // playing the festival minigame
    FestivalResults,   // showing festival results
    HorseDestination,  // picking a destination for the horse
    IceCreamShopOpen,  // ordering ice cream (summer only)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FestivalKind {
    EggHunt,       // Spring day 14
    FishingDerby,  // Summer day 14
    MushroomForage,// Fall day 14
    TreasureHunt,  // Winter day 14
}

impl FestivalKind {
    pub fn name(&self) -> &'static str {
        match self {
            FestivalKind::EggHunt        => "Egg Hunt",
            FestivalKind::FishingDerby   => "Fishing Derby",
            FestivalKind::MushroomForage => "Mushroom Forage",
            FestivalKind::TreasureHunt   => "Treasure Hunt",
        }
    }
    pub fn item_name(&self) -> &'static str {
        match self {
            FestivalKind::EggHunt        => "eggs",
            FestivalKind::FishingDerby   => "fish",
            FestivalKind::MushroomForage => "mushrooms",
            FestivalKind::TreasureHunt   => "treasures",
        }
    }
    pub fn prize_per_item(&self) -> u32 {
        match self {
            FestivalKind::EggHunt        => 100,
            FestivalKind::FishingDerby   => 150,
            FestivalKind::MushroomForage => 120,
            FestivalKind::TreasureHunt   => 200,
        }
    }
    pub fn hidden_count(&self) -> usize {
        match self {
            FestivalKind::EggHunt        => 8,
            FestivalKind::FishingDerby   => 6,
            FestivalKind::MushroomForage => 7,
            FestivalKind::TreasureHunt   => 5,
        }
    }
    pub fn max_searches(&self) -> u32 {
        match self {
            FestivalKind::EggHunt        => 15,
            FestivalKind::FishingDerby   => 12,
            FestivalKind::MushroomForage => 14,
            FestivalKind::TreasureHunt   => 10,
        }
    }
    /// Which festival (if any) for the given season?
    pub fn for_season(season: &crate::game::time::Season) -> Option<Self> {
        use crate::game::time::Season;
        match season {
            Season::Spring => Some(FestivalKind::EggHunt),
            Season::Summer => Some(FestivalKind::FishingDerby),
            Season::Fall   => Some(FestivalKind::MushroomForage),
            Season::Winter => Some(FestivalKind::TreasureHunt),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BuildingKind {
    Farmhouse,
    Inn,
    Market,
    Tavern,
    Clinic,
    Library,
    TownHall,
    FurnitureShop,
    AnimalShop,
    Arcade,
    Restaurant,
    IceCreamShop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum FurnitureKind {
    TV,
    Couch,
    Lamp,
    FishTank,
    Rug,
    PottedPlant,
}

impl FurnitureKind {
    pub fn name(&self) -> &'static str {
        match self {
            FurnitureKind::TV          => "TV",
            FurnitureKind::Couch       => "Couch",
            FurnitureKind::Lamp        => "Lamp",
            FurnitureKind::FishTank    => "Fish Tank",
            FurnitureKind::Rug         => "Rug",
            FurnitureKind::PottedPlant => "Potted Plant",
        }
    }
    pub fn price(&self) -> u32 {
        match self {
            FurnitureKind::TV          => 2000,
            FurnitureKind::Couch       => 1500,
            FurnitureKind::Lamp        => 500,
            FurnitureKind::FishTank    => 3000,
            FurnitureKind::Rug         => 800,
            FurnitureKind::PottedPlant => 300,
        }
    }
    pub const ALL: &'static [FurnitureKind] = &[
        FurnitureKind::Lamp,
        FurnitureKind::PottedPlant,
        FurnitureKind::Rug,
        FurnitureKind::Couch,
        FurnitureKind::TV,
        FurnitureKind::FishTank,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum AnimalKind {
    Chicken,
    Cat,
    Pig,
    Sheep,
    Cow,
    Horse,
}

impl AnimalKind {
    pub fn name(&self) -> &'static str {
        match self {
            AnimalKind::Chicken => "Chicken",
            AnimalKind::Cat     => "Cat",
            AnimalKind::Pig     => "Pig",
            AnimalKind::Sheep   => "Sheep",
            AnimalKind::Cow     => "Cow",
            AnimalKind::Horse   => "Horse",
        }
    }
    pub fn price(&self) -> u32 {
        match self {
            AnimalKind::Chicken => 200,
            AnimalKind::Cat     => 500,
            AnimalKind::Pig     => 1000,
            AnimalKind::Sheep   => 1500,
            AnimalKind::Cow     => 2500,
            AnimalKind::Horse   => 5000,
        }
    }
    pub const ALL: &'static [AnimalKind] = &[
        AnimalKind::Chicken,
        AnimalKind::Cat,
        AnimalKind::Pig,
        AnimalKind::Sheep,
        AnimalKind::Cow,
        AnimalKind::Horse,
    ];
    /// Where this animal appears on the farm when owned (col, row).
    pub fn farm_tile(&self) -> (usize, usize) {
        match self {
            AnimalKind::Chicken => (6, 15),   // inside chicken coop
            AnimalKind::Cat     => (4, 4),    // near farmhouse (no pen)
            AnimalKind::Pig     => (10, 16),  // inside pig pen
            AnimalKind::Sheep   => (16, 16),  // inside sheep pasture
            AnimalKind::Cow     => (7, 19),   // inside cow barn
            AnimalKind::Horse   => (12, 19),  // inside horse stable
        }
    }
}

fn animal_from_name(name: &str) -> Option<AnimalKind> {
    match name {
        "Chicken" => Some(AnimalKind::Chicken),
        "Cat"     => Some(AnimalKind::Cat),
        "Pig"     => Some(AnimalKind::Pig),
        "Sheep"   => Some(AnimalKind::Sheep),
        "Cow"     => Some(AnimalKind::Cow),
        "Horse"   => Some(AnimalKind::Horse),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Outfit {
    pub name: &'static str,
    pub shirt: (u8, u8, u8),
    pub pants: (u8, u8, u8),
    pub shoes: (u8, u8, u8),
    pub hat: (u8, u8, u8),
    pub price: u32, // 0 = free (default outfit)
}

pub const OUTFITS: &[Outfit] = &[
    Outfit { name: "Farmer",      shirt: (230,235,255), pants: (56,82,158),  shoes: (56,36,20),  hat: (38,140,64),   price: 0 },
    Outfit { name: "Rancher",     shirt: (200,160,120), pants: (100,70,40),  shoes: (60,30,10),   hat: (139,90,43),   price: 500 },
    Outfit { name: "Fisherman",   shirt: (255,220,100), pants: (60,90,130),  shoes: (40,40,40),   hat: (40,100,160),  price: 800 },
    Outfit { name: "Lumberjack",  shirt: (200,50,50),   pants: (50,50,50),   shoes: (80,50,20),   hat: (60,60,60),    price: 1000 },
    Outfit { name: "Royal",       shirt: (100,50,150),  pants: (40,40,80),   shoes: (20,20,20),   hat: (180,140,40),  price: 2000 },
    Outfit { name: "Chef",        shirt: (255,255,255), pants: (30,30,30),   shoes: (20,20,20),   hat: (255,255,255), price: 1500 },
    Outfit { name: "Pirate",      shirt: (60,60,60),    pants: (100,80,50),  shoes: (40,30,20),   hat: (30,30,30),    price: 3000 },
    Outfit { name: "Knight",      shirt: (180,180,190), pants: (140,140,150),shoes: (100,100,110), hat: (160,160,170), price: 5000 },
];

pub const HAIR_COLORS: &[(u8, u8, u8, &str)] = &[
    (82, 46, 20, "Brown"),
    (30, 30, 30, "Black"),
    (180, 140, 60, "Blonde"),
    (160, 50, 30, "Auburn"),
    (200, 80, 80, "Pink"),
    (100, 100, 110, "Silver"),
];

pub const HAIRSTYLES: &[&str] = &[
    "Short",       // 0 — default/male
    "Long",        // 1 — straight long
    "Ponytail",    // 2 — tied back
    "Curly",       // 3 — curly/wavy
    "Braids",      // 4 — two braids
    "Bob",         // 5 — short bob
    "Headband",    // 6 — hair with headband
];

/// Returns the gender of an NPC by name: 0 = Male, 1 = Female.
/// Generate 5 random rain days for a season using a simple PRNG.
pub fn generate_rain_days(season: &crate::game::time::Season, year: u16) -> Vec<u8> {
    let mut seed = (year as u64).wrapping_mul(31)
        .wrapping_add(season.name().len() as u64 * 97);
    let mut days = Vec::new();
    while days.len() < 5 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let d = ((seed >> 16) % 28) as u8 + 1;
        if !days.contains(&d) {
            days.push(d);
        }
    }
    days.sort();
    days
}

/// Pick one rainbow day that doesn't overlap with rain days.
pub fn generate_rainbow_day(rain_days: &[u8], season: &crate::game::time::Season, year: u16) -> u8 {
    let mut seed = (year as u64).wrapping_mul(53)
        .wrapping_add(season.name().len() as u64 * 41);
    loop {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let d = ((seed >> 16) % 28) as u8 + 1;
        if !rain_days.contains(&d) {
            return d;
        }
    }
}

pub fn npc_gender(name: &str) -> u8 {
    match name {
        "Elara" | "Maya" | "Lily" | "Ivy" | "Vera" | "Tess" | "Suki" |
        "Bea" | "Nora" | "Cleo" | "Faye" | "Ada" | "Cass" | "Mira" | "Rin" => 1, // Female
        _ => 0, // Male
    }
}

#[derive(Debug, Clone)]
pub struct Squirrel {
    pub x: f32,          // world x pixels
    pub y: f32,          // world y pixels
    pub home_x: f32,     // oak tree x
    pub home_y: f32,     // oak tree y
    pub target_x: f32,   // where it's running to
    pub target_y: f32,
    pub phase: u8,       // 0=running out, 1=pausing, 2=running back, 3=done
    pub timer: f32,
    pub speed: f32,
}

#[derive(Debug, Clone)]
pub struct Bird {
    pub x: f32,           // world x in pixels (col * 32.0 + offset)
    pub y: f32,           // world y in pixels
    pub home_col: usize,  // tile col where bird spawns
    pub home_row: usize,  // tile row where bird spawns
    pub flying: bool,     // true = currently flying away
    pub fly_timer: f32,   // seconds since flight started
    pub respawn_timer: f32, // countdown to respawn after flying away
    pub variant: u8,      // 0-2: visual variant (color)
}

impl Bird {
    pub fn new(col: usize, row: usize, variant: u8) -> Self {
        Self {
            x: col as f32 * 32.0 + 8.0 + (variant as f32 * 7.0) % 16.0,
            y: row as f32 * 32.0 + 10.0 + (variant as f32 * 11.0) % 12.0,
            home_col: col,
            home_row: row,
            flying: false,
            fly_timer: 0.0,
            respawn_timer: 0.0,
            variant,
        }
    }
}

/// Identify which building a tile belongs to by map coordinates.
pub fn building_at(col: usize, row: usize) -> Option<BuildingKind> {
    // Farmhouse: base 1-3×1-3, upgraded 1-5×1-4
    if col >= 1 && col <= 5 && row >= 1 && row <= 4 { return Some(BuildingKind::Farmhouse); }
    if col >= 41 && col <= 47 && row >= 2 && row <= 5 { return Some(BuildingKind::Inn); }
    if col >= 49 && col <= 55 && row >= 2 && row <= 5 { return Some(BuildingKind::Market); }
    if col >= 57 && col <= 63 && row >= 2 && row <= 5 { return Some(BuildingKind::Tavern); }
    if col >= 65 && col <= 71 && row >= 2 && row <= 5 { return Some(BuildingKind::Clinic); }
    if col >= 41 && col <= 47 && row >= 7 && row <= 12 { return Some(BuildingKind::Library); }
    if col >= 65 && col <= 75 && row >= 7 && row <= 12 { return Some(BuildingKind::TownHall); }
    if col >= 49 && col <= 55 && row >= 7 && row <= 12 { return Some(BuildingKind::FurnitureShop); }
    if col >= 63 && col <= 69 && row >= 14 && row <= 19 { return Some(BuildingKind::AnimalShop); }
    if col >= 49 && col <= 55 && row >= 21 && row <= 25 { return Some(BuildingKind::Arcade); }
    if col >= 41 && col <= 47 && row >= 21 && row <= 23 { return Some(BuildingKind::Restaurant); }
    if col >= 65 && col <= 69 && row >= 21 && row <= 23 { return Some(BuildingKind::IceCreamShop); }
    None
}

/// Check if a tile is a door (south edge) of a building. The entire south
/// row of each building is enterable so the player can walk in from anywhere
/// along the front.
pub fn door_at(col: usize, row: usize) -> Option<BuildingKind> {
    // Farmhouse south edge: row 3 (base) or row 4 (upgraded), cols 1-5
    if (row == 3 || row == 4) && col >= 1 && col <= 5 { return Some(BuildingKind::Farmhouse); }
    // Inn south edge: row 5, cols 41-47
    if row == 5 && col >= 41 && col <= 47 { return Some(BuildingKind::Inn); }
    // Market south edge: row 5, cols 49-55
    if row == 5 && col >= 49 && col <= 55 { return Some(BuildingKind::Market); }
    // Tavern south edge: row 5, cols 57-63
    if row == 5 && col >= 57 && col <= 63 { return Some(BuildingKind::Tavern); }
    // Clinic south edge: row 5, cols 65-71
    if row == 5 && col >= 65 && col <= 71 { return Some(BuildingKind::Clinic); }
    // Library south edge: row 12, cols 41-47
    if row == 12 && col >= 41 && col <= 47 { return Some(BuildingKind::Library); }
    // Town Hall south edge: row 12, cols 65-75
    if row == 12 && col >= 65 && col <= 75 { return Some(BuildingKind::TownHall); }
    // Furniture Shop south edge: row 12, cols 49-55
    if row == 12 && col >= 49 && col <= 55 { return Some(BuildingKind::FurnitureShop); }
    // Animal Shop south edge: row 19, cols 63-69
    if row == 19 && col >= 63 && col <= 69 { return Some(BuildingKind::AnimalShop); }
    // Arcade south edge: row 25, cols 49-55
    if row == 25 && col >= 49 && col <= 55 { return Some(BuildingKind::Arcade); }
    // Restaurant north edge: row 21, cols 41-47 (entered from south exit path at row 20)
    if row == 21 && col >= 41 && col <= 47 { return Some(BuildingKind::Restaurant); }
    // Ice Cream Shop north edge: row 21, cols 65-69
    if row == 21 && col >= 65 && col <= 69 { return Some(BuildingKind::IceCreamShop); }
    None
}

pub const REPLY_TOPICS: &[&str] = &[
    "Tell them about the farm",
    "Ask how they're doing",
    "Say you miss them",
];

pub struct OldFriend {
    pub name: &'static str,
    pub personality: &'static str,
}

pub const OLD_FRIENDS: &[OldFriend] = &[
    OldFriend { name: "Jules",  personality: "witty and sardonic" },
    OldFriend { name: "Maren", personality: "gentle and nostalgic" },
    OldFriend { name: "Colt",  personality: "adventurous and boisterous" },
    OldFriend { name: "Pria",  personality: "warm and philosophical" },
    OldFriend { name: "Dax",   personality: "dry and practical" },
];

/// A letter from an old friend — consumed by main.rs to start the coroutine.
pub struct LetterRequest {
    pub url: String,
    pub friend_name: String,
}

/// Fired when the player talks to an NPC — consumed by main.rs to start the
/// LLM coroutine (which lives outside pure-game code to keep macroquad out).
pub struct LlmRequest {
    pub npc_idx: usize,
    pub url: String,
}

/// Fired when dialogue closes — consumed by main.rs to persist the exchange.
pub struct MemorySave {
    pub npc_id: u8,
    pub text: String,
}

pub struct GameState {
    pub config: GameConfig,
    pub clock: GameClock,
    pub map: FarmMap,
    pub player: Player,
    pub npcs: Vec<NPC>,
    pub shop: ShopInventory,
    pub dialogue: Option<DialogueState>,
    pub phase: GamePhase,
    pub pending_gold: u32,
    pub day_summary: Option<DaySummary>,
    pub ships_today: u32,
    pub notification: Option<(String, f32)>,
    pub shop_cursor: usize,
    /// Set when player talks to NPC; consumed by main.rs to start a coroutine.
    pub pending_llm: Option<LlmRequest>,
    /// NPC name shown in the "thinking..." overlay during LlmWaiting.
    pub waiting_npc_name: Option<String>,
    /// Index into npcs vec for the currently-active (or just-finished) dialogue.
    pub waiting_npc_idx: Option<usize>,
    /// Set when dialogue closes; consumed by main.rs to fire the memory-save request.
    pub pending_memory: Option<MemorySave>,
    /// True when the current LLM dialogue is Victor's final win-condition scene.
    pub is_victor_final: bool,
    /// Set after dismissing day summary on a letter day; consumed by main.rs.
    pub pending_letter: Option<LetterRequest>,
    /// The currently open letter: (friend_name, letter_text).
    pub current_letter: Option<(String, String)>,
    /// Accumulates real time; fires an NPC step every NPC_STEP_INTERVAL seconds.
    pub npc_step_timer: f32,
    /// Player-response options shown during DialogueChoice phase.
    pub response_options: Vec<String>,
    /// Currently highlighted option index in DialogueChoice phase.
    pub choice_cursor: usize,
    /// True when the pending/arriving LLM result is a follow-up reaction (not an initial greeting).
    pub is_followup_dialogue: bool,
    /// Set when player picks a response in Morgan's interview (0=Humble, 1=Confident, 2=Deflecting).
    /// Consumed by _close_dialogue() to apply multi-NPC reputation effects.
    pub journalist_choice: Option<usize>,
    /// ID of the NPC the player has married, if any.
    pub married_npc_id: Option<u8>,
    /// True when the current DialogueChoice screen is a marriage proposal.
    /// Consumed by _close_dialogue() to seal the engagement.
    pub is_proposal: bool,
    /// The choice index the player confirmed (set by confirm_choice; read by _close_dialogue).
    pub last_choice_idx: usize,
    /// Player's tile position inside the farmhouse interior (col, row), 0-indexed.
    /// Range: col 0–11, row 0–7. Row 7 south edge = door / exit.
    pub farmhouse_tile: (i32, i32),
    /// Whether the relationships overlay is visible.
    pub show_relationships: bool,
    /// Cursor index for LetterReply topic selection.
    pub reply_cursor: usize,
    /// Set to true when state should be serialized and written to storage.
    /// Consumed by main.rs (which has access to the platform save API).
    pub pending_save: bool,
    /// Which building the player is currently inside.
    pub current_building: BuildingKind,
    /// Items available for shipping in the ShipSelect UI.
    /// Each entry: (item, quantity_owned, price_per_unit, selected).
    pub ship_manifest: Vec<(ItemKind, u32, u32, bool)>,
    /// Cursor position in the ship manifest (last slot = "Ship" button).
    pub ship_cursor: usize,
    /// Furniture the player has purchased for their farmhouse.
    pub owned_furniture: std::collections::HashSet<FurnitureKind>,
    /// Cursor position in the furniture shop UI.
    pub furniture_cursor: usize,
    /// Whether it's raining today.
    pub raining: bool,
    /// Whether it's a rainbow day (50% off everything).
    pub rainbow_day: bool,
    /// The rainbow day for this season (1-28, guaranteed not a rain day).
    pub rainbow_day_num: u8,
    /// Which days this season have rain (5 random days per 28-day month).
    pub rain_days: Vec<u8>,
    /// Restaurant menu cursor.
    pub restaurant_cursor: usize,
    pub icecream_cursor: usize,
    /// Arcade mini-game state: 0=waiting, 1=ready(green), 2=done
    pub arcade_phase: u8,
    /// Timer for the arcade game
    pub arcade_timer: f32,
    /// Delay before the light turns green
    pub arcade_delay: f32,
    /// Player's reaction time (seconds)
    pub arcade_reaction: f32,
    /// Prize won
    pub arcade_prize: u32,
    /// ID of the NPC player 2 has married, if any.
    pub married_npc_id_p2: Option<u8>,
    /// Whether co-op mode is active.
    pub coop_active: bool,
    /// P2's friendship values with NPCs (npc_id → friendship).
    pub p2_friendships: std::collections::HashMap<u8, u8>,
    /// Whether P2's relationships overlay is showing.
    pub show_relationships_p2: bool,
    /// Multiplayer role: "none", "host", or "guest".
    pub mp_role: String,
    /// Room code for multiplayer.
    pub mp_room: String,
    /// Sync timer — host sends state every 100ms.
    pub mp_sync_timer: f32,
    /// Player 2 state (co-op).
    pub player2: crate::game::player::Player,
    /// Whether player 2 is riding the horse.
    pub riding_horse_p2: bool,
    /// Whether the player is currently riding the horse.
    pub riding_horse: bool,
    /// Horse autopilot destination (col, row). None = manual control.
    pub horse_target: Option<(usize, usize)>,
    /// Pre-computed A* path for horse autopilot. Each entry is the next tile to move to.
    pub horse_path: Vec<(usize, usize)>,
    /// How many consecutive steps the horse has been blocked by an NPC.
    pub horse_blocked_count: u8,
    /// Cursor for the destination picker menu.
    pub horse_dest_cursor: usize,
    /// Whether the player has purchased the equestrian center.
    pub has_equestrian_center: bool,
    /// Jump positions in the riding arena: (col_offset, row_offset, orientation).
    /// Orientation: 0=horizontal, 1=vertical, 2=diagonal-right, 3=diagonal-left.
    pub arena_jumps: Vec<(u8, u8, u8)>,
    /// Cursor position in the arena editor (col_offset, row_offset).
    pub arena_cursor: (u8, u8),
    /// Horse leap timer (> 0 = currently leaping, counts down).
    pub horse_leap_timer: f32,
    /// Horse leap visual height offset in pixels.
    pub horse_leap_height: f32,
    /// Whether the player has purchased the house extension.
    pub house_upgraded: bool,
    /// Animals the player has purchased for their farm.
    pub owned_animals: std::collections::HashSet<AnimalKind>,
    /// Cursor position in the animal shop UI.
    pub animal_cursor: usize,
    /// Outfits the player has purchased (indices into OUTFITS). Index 0 is always owned.
    pub owned_outfits: std::collections::HashSet<u8>,
    /// Cursor for the outfit shop.
    pub outfit_cursor: usize,
    /// Fishing minigame state
    pub fish_bar: f32,       // catch zone position (0.0 = bottom, 1.0 = top)
    pub fish_bar_vel: f32,   // catch zone velocity
    pub fish_pos: f32,       // fish position (0.0 = bottom, 1.0 = top)
    pub fish_vel: f32,       // fish velocity
    pub fish_progress: f32,  // catch progress (0.0 to 1.0; 1.0 = caught)
    pub fish_target: Option<crate::game::inventory::FishKind>,
    pub fish_difficulty: f32, // how erratic the fish is
    pub fish_player2: bool,   // true if P2 is the one fishing

    /// Current music track: 0=off, 1=guqin, 2=pop.
    pub music_track: u32,
    /// Squirrels that run from oak trees.
    pub squirrels: Vec<Squirrel>,
    /// Timer for squirrel spawning.
    pub squirrel_timer: f32,

    /// Ambient birds that fly away when the player approaches.
    pub birds: Vec<Bird>,

    /// Current festival state
    pub festival_kind: Option<FestivalKind>,
    pub festival_grid: Vec<Vec<bool>>,     // 8x6 grid, true = item hidden here
    pub festival_revealed: Vec<Vec<bool>>, // true = tile has been searched
    pub festival_cursor: (usize, usize),   // (col, row) in the grid
    pub festival_found: u32,
    pub festival_searches_left: u32,
    pub festival_prize: u32,               // calculated at end
}

impl GameState {
    pub fn new(config: GameConfig) -> Self {
        let start_energy = config.energy.start;
        let player = Player::new(start_energy, 500);
        let map = FarmMap::default_farm();
        let shop = make_default_shop(&config);
        let npcs = build_npcs(&config);

        Self {
            config,
            clock: GameClock::new(),
            map,
            player,
            npcs,
            shop,
            dialogue: None,
            phase: GamePhase::Playing,
            pending_gold: 0,
            day_summary: None,
            ships_today: 0,
            notification: Some(("Everyone knows you won the lottery. Welcome to Bennett Valley.".into(), 5.0)),
            shop_cursor: 0,
            pending_llm: None,
            waiting_npc_name: None,
            waiting_npc_idx: None,
            pending_memory: None,
            is_victor_final: false,
            pending_letter: None,
            current_letter: None,
            npc_step_timer: 0.0,
            response_options: Vec::new(),
            choice_cursor: 0,
            is_followup_dialogue: false,
            journalist_choice: None,
            married_npc_id: None,
            is_proposal: false,
            last_choice_idx: 0,
            farmhouse_tile: (5, 6),
            show_relationships: false,
            reply_cursor: 0,
            pending_save: false,
            current_building: BuildingKind::Farmhouse,
            ship_manifest: Vec::new(),
            ship_cursor: 0,
            raining: {
                let rd = generate_rain_days(&crate::game::time::Season::Spring, 1);
                rd.contains(&1u8)
            },
            rainbow_day: {
                let rd = generate_rain_days(&crate::game::time::Season::Spring, 1);
                let rb = generate_rainbow_day(&rd, &crate::game::time::Season::Spring, 1);
                rb == 1
            },
            rainbow_day_num: {
                let rd = generate_rain_days(&crate::game::time::Season::Spring, 1);
                generate_rainbow_day(&rd, &crate::game::time::Season::Spring, 1)
            },
            rain_days: generate_rain_days(&crate::game::time::Season::Spring, 1),
            restaurant_cursor: 0,
            icecream_cursor: 0,
            arcade_phase: 0,
            arcade_timer: 0.0,
            arcade_delay: 0.0,
            arcade_reaction: 0.0,
            arcade_prize: 0,
            married_npc_id_p2: None,
            coop_active: false,
            p2_friendships: std::collections::HashMap::new(),
            show_relationships_p2: false,
            mp_role: "none".to_string(),
            mp_room: String::new(),
            mp_sync_timer: 0.0,
            player2: {
                let mut p2 = crate::game::player::Player::new(start_energy, 0);
                p2.tile = (12, 10);
                p2.outfit = 1;
                p2
            },
            riding_horse_p2: false,
            riding_horse: false,
            horse_target: None,
            horse_path: Vec::new(),
            horse_blocked_count: 0,
            horse_dest_cursor: 0,
            has_equestrian_center: false,
            arena_jumps: vec![(2, 1, 0), (5, 2, 1)], // default: one horizontal, one vertical
            arena_cursor: (0, 0),
            horse_leap_timer: 0.0,
            horse_leap_height: 0.0,
            house_upgraded: false,
            owned_furniture: std::collections::HashSet::new(),
            furniture_cursor: 0,
            owned_animals: std::collections::HashSet::new(),
            animal_cursor: 0,
            owned_outfits: { let mut s = std::collections::HashSet::new(); s.insert(0u8); s },
            outfit_cursor: 0,
            fish_bar: 0.5,
            fish_bar_vel: 0.0,
            fish_pos: 0.5,
            fish_vel: 0.3,
            fish_progress: 0.3,
            fish_target: None,
            fish_difficulty: 1.0,
            fish_player2: false,
            squirrels: Vec::new(),
            squirrel_timer: 0.0,
            birds: Self::spawn_birds(),
            festival_kind: None,
            festival_grid: Vec::new(),
            festival_revealed: Vec::new(),
            festival_cursor: (0, 0),
            festival_found: 0,
            festival_searches_left: 0,
            festival_prize: 0,
            music_track: 0,
        }
    }

    pub fn notify(&mut self, msg: impl Into<String>) {
        self.notification = Some((msg.into(), 2.5));
    }

    pub fn tick_notification(&mut self, dt: f32) {
        if let Some((_, ttl)) = &mut self.notification {
            *ttl -= dt;
            if *ttl <= 0.0 {
                self.notification = None;
            }
        }
    }

    pub fn tick_time(&mut self) {
        if self.phase != GamePhase::Playing {
            return;
        }
        let event = self.clock.tick();
        if event == TimeEvent::ForcedSleep {
            self.advance_day();
        }
    }

    /// Called once on init / day-start — teleports all NPCs to their correct tile,
    /// nudging them to nearby passable tiles if the target is already occupied.
    pub fn snap_npcs(&mut self) {
        let hour = self.clock.hour;
        let mut occupied = std::collections::HashSet::new();
        // Reserve the player tile so NPCs don't land on the player.
        occupied.insert(self.player.tile);

        for npc in &mut self.npcs {
            let target = npc.schedule_target(hour);
            let tile = Self::find_open_tile(target, &occupied, &self.map);
            npc.tile = tile;
            occupied.insert(tile);
        }
    }

    /// Find a passable tile near `target` that isn't in `occupied` or in the farm zone.
    /// Tries the target first, then spirals outward.
    fn find_open_tile(
        target: (usize, usize),
        occupied: &std::collections::HashSet<(usize, usize)>,
        map: &FarmMap,
    ) -> (usize, usize) {
        use crate::game::npc::is_npc_excluded;
        // Try the target itself first
        if !occupied.contains(&target) && !is_npc_excluded(target.0, target.1, map) {
            if let Some(t) = map.get(target.0, target.1) {
                if t.kind.is_passable() {
                    return target;
                }
            }
        }
        // Spiral outward looking for a free passable tile
        for radius in 1..15 {
            for dx in -(radius as i32)..=(radius as i32) {
                for dy in -(radius as i32)..=(radius as i32) {
                    if dx.unsigned_abs() as usize != radius && dy.unsigned_abs() as usize != radius {
                        continue;
                    }
                    let nx = target.0 as i32 + dx;
                    let ny = target.1 as i32 + dy;
                    if nx < 0 || ny < 0 { continue; }
                    let pos = (nx as usize, ny as usize);
                    if occupied.contains(&pos) { continue; }
                    if is_npc_excluded(pos.0, pos.1, map) { continue; }
                    if let Some(t) = map.get(pos.0, pos.1) {
                        if t.kind.is_passable() {
                            return pos;
                        }
                    }
                }
            }
        }
        target // fallback
    }

    /// Serialize current state into a SaveData snapshot.
    pub fn to_save(&self) -> SaveData {
        SaveData {
            day: self.clock.day,
            hour: self.clock.hour,
            minute: self.clock.minute,
            season: self.clock.season.name().to_string(),
            year: self.clock.year,
            tile: [self.player.tile.0, self.player.tile.1],
            energy: self.player.energy,
            max_energy: self.player.max_energy,
            gold: self.player.gold,
            hoe_level: self.player.hoe_level,
            can_level: self.player.can_level,
            charisma_xp: self.player.charisma_xp,
            selected_seed: self.player.selected_seed.as_ref().map(|s| s.name().to_string()),
            inventory: self.player.inventory.items().iter()
                .map(|(k, &v)| (item_to_key(k), v))
                .collect(),
            npcs: self.npcs.iter().map(|n| NpcSave {
                id: n.id,
                friendship: n.friendship,
                loved_gift_given: n.loved_gift_given,
            }).collect(),
            pending_gold: self.pending_gold,
            ships_today: self.ships_today,
            married_npc_id: self.married_npc_id,
            house_upgraded: self.house_upgraded,
            owned_furniture: self.owned_furniture.iter().map(|f| f.name().to_string()).collect(),
            owned_animals: self.owned_animals.iter().map(|a| a.name().to_string()).collect(),
            has_equestrian_center: self.has_equestrian_center,
            arena_jumps: self.arena_jumps.clone(),
            outfit: self.player.outfit,
            gender: self.player.gender,
            hairstyle: self.player.hairstyle,
            hair_color: self.player.hair_color,
            owned_outfits: self.owned_outfits.iter().copied().collect(),
        }
    }

    /// Restore game state from a SaveData snapshot.
    pub fn apply_save(&mut self, data: SaveData) {
        self.clock.day    = data.day;
        self.clock.hour   = 6;
        self.clock.minute = 0;
        self.clock.season = season_from_name(&data.season);
        self.clock.year   = data.year;

        self.player.tile         = (data.tile[0], data.tile[1]);
        self.player.energy       = data.energy;
        self.player.max_energy   = data.max_energy;
        self.player.gold         = data.gold;
        self.player.hoe_level    = data.hoe_level;
        self.player.can_level    = data.can_level;
        self.player.charisma_xp  = data.charisma_xp;
        self.player.selected_seed = data.selected_seed.as_deref().and_then(seed_from_name);
        self.player.inventory    = PlayerInventory::new();
        for (key, qty) in &data.inventory {
            if let Some(item) = item_from_key(key) {
                self.player.inventory.add(item, *qty);
            }
        }

        for npc_save in &data.npcs {
            if let Some(npc) = self.npcs.iter_mut().find(|n| n.id == npc_save.id) {
                npc.friendship       = npc_save.friendship;
                npc.loved_gift_given = npc_save.loved_gift_given;
            }
        }

        self.pending_gold    = data.pending_gold;
        self.ships_today     = data.ships_today;
        self.married_npc_id  = data.married_npc_id;
        // Restore house upgrade
        self.house_upgraded = data.house_upgraded;
        if self.house_upgraded {
            for row in 1..5 {
                for col in 1..6 {
                    self.map.tiles[row][col].kind = crate::game::world::TileKind::Farmhouse;
                }
            }
        }

        // Restore owned furniture
        self.owned_furniture.clear();
        for name in &data.owned_furniture {
            if let Some(fk) = furniture_from_name(name) {
                self.owned_furniture.insert(fk);
            }
        }

        // Restore equestrian center
        self.has_equestrian_center = data.has_equestrian_center;
        if !data.arena_jumps.is_empty() {
            self.arena_jumps = data.arena_jumps.clone();
        }

        // Restore outfit and gender
        self.player.outfit = data.outfit;
        self.player.gender = data.gender;
        self.player.hairstyle = data.hairstyle;
        self.player.hair_color = data.hair_color;
        self.owned_outfits.clear();
        self.owned_outfits.insert(0); // default always owned
        for idx in &data.owned_outfits {
            self.owned_outfits.insert(*idx);
        }

        // Restore owned animals
        self.owned_animals.clear();
        for name in &data.owned_animals {
            if let Some(ak) = animal_from_name(name) {
                self.owned_animals.insert(ak);
            }
        }

        self.phase           = GamePhase::Playing;
        self.snap_npcs();
    }

    /// Called every frame with delta-time — moves NPCs one tile every 0.35 s.
    pub fn tick_npc_movement(&mut self, dt: f32) {
        const STEP_INTERVAL: f32 = 0.35;
        self.npc_step_timer += dt;
        if self.npc_step_timer < STEP_INTERVAL {
            return;
        }
        self.npc_step_timer -= STEP_INTERVAL;

        let hour = self.clock.hour;
        // Collect targets first to avoid a double-borrow on self.
        let targets: Vec<(usize, usize)> = self.npcs.iter()
            .map(|n| n.schedule_target(hour))
            .collect();

        // Occupied set: updated as each NPC claims its new tile so no two NPCs
        // end up on the same tile after a step.
        let mut occupied: std::collections::HashSet<(usize, usize)> =
            self.npcs.iter().map(|n| n.tile).collect();

        for i in 0..self.npcs.len() {
            let old_tile = self.npcs[i].tile;
            occupied.remove(&old_tile);           // free this NPC's current tile
            self.npcs[i].step_toward(targets[i], &self.map, &occupied);
            occupied.insert(self.npcs[i].tile);   // claim the new (or same) tile
        }
    }

    pub fn advance_day(&mut self) {
        let gold_earned = self.pending_gold;
        let crops_shipped = self.ships_today;
        let ending_season = self.clock.season.name().to_string();

        // Credit pending gold
        self.player.gold += self.pending_gold;
        self.pending_gold = 0;
        self.ships_today = 0;

        // Restore energy (both players)
        self.player.restore_energy();
        self.player2.energy = self.player2.max_energy;

        // Reset NPC daily state
        for npc in &mut self.npcs {
            npc.reset_daily();
        }

        // Chickens lay eggs each morning
        if self.owned_animals.contains(&AnimalKind::Chicken) {
            self.player.inventory.add(ItemKind::Egg, 1);
            self.notify("Your chicken laid an egg!");
            crate::game::save::play_sound("egg");
        }
        if self.owned_animals.contains(&AnimalKind::Cow) {
            self.player.inventory.add(ItemKind::Milk, 1);
            self.notify("Your cow produced milk!");
        }

        // Advance clock first so we know if a season ended
        let event = self.clock.advance_day();
        let season_ended = if event == TimeEvent::SeasonEnd {
            Some(ending_season)
        } else {
            None
        };

        // Update rain and rainbow schedule
        if season_ended.is_some() {
            self.rain_days = generate_rain_days(&self.clock.season, self.clock.year);
            self.rainbow_day_num = generate_rainbow_day(&self.rain_days, &self.clock.season, self.clock.year);
        }
        self.raining = self.rain_days.contains(&self.clock.day);
        self.rainbow_day = self.clock.day == self.rainbow_day_num && !self.raining;

        // Advance crops, reset watered state, replenish forage patches.
        // On season end: clear all crops and reset Tilled/Watered tiles to Grass.
        for row in &mut self.map.tiles {
            for tile in row {
                // Forage patches always replenish each morning
                if tile.kind == crate::game::world::TileKind::ForagePatchEmpty {
                    tile.kind = crate::game::world::TileKind::ForagePatch;
                }
                // Oak trees replenish acorns each morning during Fall,
                // and reset to OakTree on season start (so they're ready next Fall).
                if tile.kind == crate::game::world::TileKind::OakTreeEmpty {
                    if self.clock.season == crate::game::time::Season::Fall {
                        tile.kind = crate::game::world::TileKind::OakTree;
                    }
                }

                if season_ended.is_some() {
                    tile.crop = None;
                    if matches!(tile.kind, crate::game::world::TileKind::Tilled | crate::game::world::TileKind::Watered) {
                        tile.kind = crate::game::world::TileKind::Grass;
                    }
                    // Reset oak trees for next Fall
                    if tile.kind == crate::game::world::TileKind::OakTreeEmpty {
                        tile.kind = crate::game::world::TileKind::OakTree;
                    }
                } else {
                    if let Some(crop) = &mut tile.crop {
                        let crop_name = crop.kind.name().to_string();
                        let grow_days = self
                            .config
                            .crops
                            .get(&crop_name)
                            .map(|c| c.grow_days)
                            .unwrap_or(4);
                        crop.advance_day(grow_days);
                    }
                    if tile.kind == crate::game::world::TileKind::Watered {
                        tile.kind = crate::game::world::TileKind::Tilled;
                    }
                    // Rain auto-waters all tilled tiles with crops
                    if self.raining && tile.kind == crate::game::world::TileKind::Tilled && tile.crop.is_some() {
                        tile.kind = crate::game::world::TileKind::Watered;
                    }
                }
            }
        }

        // Regrow long grass at the start of Spring (day 1 after Winter ends)
        if self.clock.season == crate::game::time::Season::Spring && self.clock.day == 1 {
            let year_seed = self.clock.year as usize * 997;
            for row in 1..30 {
                for col in 1..40 {
                    if self.map.tiles[row][col].kind == crate::game::world::TileKind::Grass
                        && self.map.tiles[row][col].crop.is_none()
                    {
                        let h = col.wrapping_mul(17).wrapping_add(row.wrapping_mul(31)).wrapping_add(year_seed);
                        if h % 6 == 0 {
                            self.map.tiles[row][col].kind = crate::game::world::TileKind::LongGrass;
                        }
                    }
                }
            }
            // South wilderness too
            for row in 30..50 {
                for col in 1..75 {
                    if self.map.tiles[row][col].kind == crate::game::world::TileKind::Grass
                        && self.map.tiles[row][col].crop.is_none()
                    {
                        let h = col.wrapping_mul(13).wrapping_add(row.wrapping_mul(19)).wrapping_add(year_seed);
                        if h % 5 == 0 {
                            self.map.tiles[row][col].kind = crate::game::world::TileKind::LongGrass;
                        }
                    }
                }
            }
        }

        // Snap NPCs to their new-day schedule positions (collision-aware).
        self.snap_npcs();

        // ── Victor pressure: from Year 4 on, Victor loses 1 heart per season ──
        if season_ended.is_some() && self.clock.year >= 4 {
            if let Some(victor) = self.npcs.iter_mut()
                .find(|n| n.id == crate::game::npc::VICTOR_ID)
            {
                if victor.hearts() < crate::game::npc::VICTOR_FINAL_HEARTS {
                    let decay = victor.friendship.min(25);
                    victor.friendship -= decay;
                    if decay > 0 {
                        self.notification = Some((
                            format!(
                                "Victor grows more distant... ({}/{} hearts)",
                                victor.hearts(),
                                crate::game::npc::VICTOR_FINAL_HEARTS
                            ),
                            5.0,
                        ));
                    }
                }
            }
        }

        let summary = DaySummary {
            day: self.clock.day,
            gold_earned,
            crops_shipped,
            season_ended,
        };
        self.day_summary = Some(summary);
        self.phase = GamePhase::DaySummary;
        self.pending_save = true;
    }

    pub fn dismiss_summary(&mut self) {
        if self.phase == GamePhase::DaySummary {
            self.day_summary = None;

            // Every 4th day a letter arrives from an old friend.
            if self.clock.day % 4 == 0 {
                let idx = ((self.clock.day / 4) as usize).wrapping_sub(1) % OLD_FRIENDS.len();
                let friend = &OLD_FRIENDS[idx];
                let personality_enc = friend.personality.replace(' ', "+");
                let url = format!(
                    "/api/letter?friend={}&personality={}&season={}&day={}&gold={}",
                    friend.name,
                    personality_enc,
                    self.clock.season.name(),
                    self.clock.day,
                    self.player.gold,
                );
                self.pending_letter = Some(LetterRequest {
                    url,
                    friend_name: friend.name.to_string(),
                });
                self.phase = GamePhase::LetterWaiting;
            } else if FestivalKind::for_season(&self.clock.season).is_some() && self.clock.day == 14 {
                self.check_festival();
            } else {
                self.phase = GamePhase::Playing;
            }
        }
    }

    pub fn dismiss_letter(&mut self) {
        if self.phase == GamePhase::LetterOpen {
            self.current_letter = None;
            self.phase = GamePhase::Playing;
        }
    }

    pub fn start_letter_reply(&mut self) {
        if self.phase == GamePhase::LetterOpen && self.current_letter.is_some() {
            self.reply_cursor = 0;
            self.phase = GamePhase::LetterReply;
        }
    }

    pub fn move_reply_cursor(&mut self, delta: i32) {
        let len = REPLY_TOPICS.len() as i32;
        self.reply_cursor = ((self.reply_cursor as i32 + delta).rem_euclid(len)) as usize;
    }

    pub fn confirm_letter_reply(&mut self) {
        if self.phase != GamePhase::LetterReply { return; }
        let topic = REPLY_TOPICS[self.reply_cursor];
        let friend_name = match &self.current_letter {
            Some((name, _)) => name.clone(),
            None => return,
        };
        let personality = OLD_FRIENDS.iter()
            .find(|f| f.name == friend_name.as_str())
            .map(|f| f.personality)
            .unwrap_or("friendly");
        let url = format!(
            "/api/reply?friend={}&personality={}&topic={}&season={}&day={}",
            url_encode(&friend_name),
            url_encode(personality),
            url_encode(topic),
            self.clock.season.name(),
            self.clock.day,
        );
        self.pending_letter = Some(LetterRequest { url, friend_name });
        self.phase = GamePhase::LetterWaiting;
    }


    /// Try to hoe the tile(s) the player is facing, based on hoe_level.
    /// Level 0: 1 tile. Level 1: cheaper. Level 2+: 3-tile line. Level 3: 5-tile line.
    pub fn try_hoe(&mut self) -> Result<(), ActionError> {
        let base_cost = self.config.energy.hoe_cost;
        let level = self.player.hoe_level;
        let cost = (base_cost - level as i16).max(1);
        let tiles = hoe_tiles(self.player.tile, &self.player.facing, level);
        // Deduct energy once up front.
        if !self.player.spend_energy(cost) {
            return Err(ActionError::NotEnoughEnergy);
        }
        let mut any_ok = false;
        for (col, row) in tiles {
            // Pass cost=0 since energy already spent above.
            if farming::hoe_tile(&mut self.map, &mut self.player, col, row, 0).is_ok() {
                any_ok = true;
            }
        }
        if any_ok { Ok(()) } else { Err(ActionError::InvalidTile) }
    }

    /// Try to water tile(s) the player is facing, based on can_level.
    /// Level 0: 1 tile. Level 1: 3-tile row. Level 2: 5-tile cross. Level 3: 3×3 area.
    pub fn try_water(&mut self) -> Result<(), ActionError> {
        let base_cost = self.config.energy.water_cost;
        let level = self.player.can_level;
        let cost = (base_cost - level as i16).max(1);
        let tiles = water_tiles(self.player.tile, &self.player.facing, level);
        let mut any_ok = false;
        // Deduct energy once up front, then water all tiles at 0 cost.
        if !self.player.spend_energy(cost) {
            return Err(ActionError::NotEnoughEnergy);
        }
        for (col, row) in tiles {
            // Pass cost=0 since energy already spent above.
            if farming::water_tile(&mut self.map, &mut self.player, col, row, 0).is_ok() {
                any_ok = true;
            }
        }
        if any_ok { Ok(()) } else { Err(ActionError::InvalidTile) }
    }

    /// Gift the best available item to the adjacent NPC.
    pub fn try_gift(&mut self) {
        let facing = self.player.facing_tile();
        let npc_idx = self.npcs.iter().position(|n| n.tile == facing);
        let Some(idx) = npc_idx else {
            self.notify("No one to gift here.");
            return;
        };

        let npc_name = self.npcs[idx].name.clone();

        if self.npcs[idx].gifted_today {
            self.notify(format!("{} already received a gift today.", npc_name));
            return;
        }

        // Find the best item in inventory the NPC has preferences for.
        let best = self.player.inventory.items().iter()
            .filter(|(_, &qty)| qty > 0)
            .filter_map(|(item, _)| {
                let name = item_kind_to_name(item);
                let pref = *self.npcs[idx].gift_preferences.get(name).unwrap_or(&0);
                if pref >= 0 { Some((item.clone(), name, pref)) } else { None }
            })
            .max_by_key(|(_, _, p)| *p);

        let Some((item, item_name, _)) = best else {
            self.notify(format!("{} doesn't want anything you have.", npc_name));
            return;
        };

        // Consume 1 of the item.
        self.player.inventory.remove(&item, 1);

        let pref = self.npcs[idx].give_gift_by_name(item_name);

        let reaction = match pref {
            p if p >= 2 => format!("{} loved the {}! (♥♥)", npc_name, item_name),
            1            => format!("{} liked the {}! (♥)", npc_name, item_name),
            _            => format!("{} accepted the {}.", npc_name, item_name),
        };
        self.notify(reaction);

        // Hint when conversation is now capped without a loved gift.
        if self.npcs[idx].friendship_capped() && !self.npcs[idx].loved_gift_given {
            self.notify(format!("{} wants something truly special to grow closer.", npc_name));
        }
    }

    /// Try to plant the selected seed on the tile the player is facing.
    pub fn try_plant(&mut self) -> Result<(), ActionError> {
        let seed = self.player.selected_seed.clone().ok_or(ActionError::NoSeedSelected)?;
        let kind = seed_to_crop_kind(&seed);
        let (col, row) = self.player.facing_tile();
        let cost = self.config.energy.plant_cost;
        farming::plant_seed(&mut self.map, &mut self.player, col, row, kind, cost)?;
        Ok(())
    }

    /// Strike the rock the player is facing.
    pub fn try_mine(&mut self) -> Result<(), ActionError> {
        let (col, row) = self.player.facing_tile();
        let cost = self.config.energy.mine_cost;
        let rock_hp = self.config.ore.rock_hp;
        farming::mine_tile(&mut self.map, &mut self.player, col, row, cost, rock_hp)?;
        Ok(())
    }

    /// Try to fish at the FishingSpot the player is standing on.
    pub fn try_fish(&mut self) -> Result<(), ActionError> {
        let (col, row) = self.player.tile;
        let tile = self.map.get(col, row).ok_or(ActionError::InvalidTile)?;
        if tile.kind != crate::game::world::TileKind::FishingSpot {
            return Err(ActionError::NotAFishingSpot);
        }
        let cost = self.config.energy.fish_cost;
        if !self.player.spend_energy(cost) {
            return Err(ActionError::NotEnoughEnergy);
        }
        // Determine the fish and start the minigame
        let season = self.clock.season.clone();
        let fish = farming::season_fish_pub(&season, col, row);
        self.fish_target = Some(fish.clone());
        self.fish_bar = 0.5;
        self.fish_bar_vel = 0.0;
        self.fish_pos = 0.5;
        self.fish_vel = 0.3;
        self.fish_progress = 0.3;
        self.fish_difficulty = match fish {
            crate::game::inventory::FishKind::Bass    => 0.6,
            crate::game::inventory::FishKind::Catfish => 0.9,
            crate::game::inventory::FishKind::Trout   => 1.2,
            crate::game::inventory::FishKind::Salmon  => 1.5,
        };
        self.fish_player2 = false;
        self.phase = GamePhase::FishingMinigame;
        Ok(())
    }

    /// Update the fishing minigame state. Called every frame.
    pub fn tick_fishing(&mut self, dt: f32, reel_up: bool) {
        if self.phase != GamePhase::FishingMinigame { return; }

        let diff = self.fish_difficulty;
        let catch_zone_size = 0.2; // 20% of the bar

        // Fish movement — bounces erratically
        self.fish_pos += self.fish_vel * dt;
        if self.fish_pos >= 1.0 { self.fish_pos = 1.0; self.fish_vel = -self.fish_vel.abs(); }
        if self.fish_pos <= 0.0 { self.fish_pos = 0.0; self.fish_vel = self.fish_vel.abs(); }
        // Random direction changes (use frame-dependent pseudo-randomness)
        let t = self.fish_progress * 100.0 + self.fish_pos * 73.0 + dt * 1000.0;
        if (t * 17.3).sin() > 0.7 {
            self.fish_vel += (t * 31.7).sin() * diff * 2.0 * dt;
        }
        self.fish_vel = self.fish_vel.clamp(-diff, diff);

        // Catch bar — player controls with W (reel up), gravity pulls down
        if reel_up {
            self.fish_bar_vel += 3.0 * dt;
        }
        self.fish_bar_vel -= 0.5 * dt; // gravity (gentle)
        self.fish_bar += self.fish_bar_vel;
        if self.fish_bar >= 1.0 { self.fish_bar = 1.0; self.fish_bar_vel = 0.0; }
        if self.fish_bar <= 0.0 { self.fish_bar = 0.0; self.fish_bar_vel = 0.0; }

        // Check if fish is inside catch zone
        let bar_top = self.fish_bar + catch_zone_size / 2.0;
        let bar_bot = self.fish_bar - catch_zone_size / 2.0;
        let caught = self.fish_pos >= bar_bot && self.fish_pos <= bar_top;

        if caught {
            self.fish_progress += 0.3 * dt; // fill rate
        } else {
            self.fish_progress -= 0.2 * dt; // drain rate
        }

        // Check win/lose
        if self.fish_progress >= 1.0 {
            // Caught! Add to the correct player's inventory
            if let Some(fish) = self.fish_target.take() {
                let inv = if self.fish_player2 { &mut self.player2.inventory } else { &mut self.player.inventory };
                inv.add(ItemKind::Fish(fish.clone()), 1);
                let who = if self.fish_player2 { "P2 caught" } else { "Caught" };
                self.notify(&format!("{} a {}!", who, fish.name()));
                crate::game::save::play_sound("fishCatch");
            }
            self.phase = GamePhase::Playing;
        } else if self.fish_progress <= 0.0 {
            // Escaped!
            self.fish_target = None;
            self.notify("The fish got away...");
            crate::game::save::play_sound("fishFail");
            self.phase = GamePhase::Playing;
        }
    }

    /// Try to forage the tile the player is standing on, or an adjacent oak tree.
    pub fn try_forage(&mut self) -> Result<(), ActionError> {
        let (col, row) = self.player.tile;
        let cost = self.config.energy.forage_cost;
        let season = self.clock.season.clone();

        // Check standing tile first (regular forage patch)
        let on_forage = self.map.get(col, row)
            .map(|t| t.kind == crate::game::world::TileKind::ForagePatch)
            .unwrap_or(false);

        if on_forage {
            farming::forage_tile(&mut self.map, &mut self.player, col, row, cost, &season)?;
            return Ok(());
        }

        // Check facing tile (oak tree — player can't walk onto it)
        let (fcol, frow) = self.player.facing_tile();
        let facing_oak = self.map.get(fcol, frow)
            .map(|t| t.kind == crate::game::world::TileKind::OakTree)
            .unwrap_or(false);

        if facing_oak {
            farming::forage_oak(&mut self.map, &mut self.player, fcol, frow, cost, &season)?;
            return Ok(());
        }

        Err(ActionError::NothingToForage)
    }

    /// Try to harvest the tile the player is facing.
    pub fn try_harvest(&mut self) -> Result<(), ActionError> {
        let (col, row) = self.player.facing_tile();
        let tile = self.map.get(col, row).ok_or(ActionError::InvalidTile)?;
        let crop_name = tile
            .crop
            .as_ref()
            .ok_or(ActionError::NothingToHarvest)?
            .kind
            .name()
            .to_string();
        let grow_days = self
            .config
            .crops
            .get(&crop_name)
            .map(|c| c.grow_days)
            .unwrap_or(4);
        let cost = self.config.energy.harvest_cost;
        farming::harvest_tile(&mut self.map, &mut self.player, col, row, grow_days, cost)?;
        Ok(())
    }

    /// Ship all crops and forageables at the shipping box.
    pub fn ship_all(&mut self) {
        let mut sell_prices: Vec<(ItemKind, u32)> = self
            .config
            .crops
            .iter()
            .filter_map(|(name, cfg)| {
                let item = crop_name_to_item(name)?;
                Some((item, cfg.sell_price))
            })
            .collect();

        // Add forage items
        for (name, &price) in &self.config.forage.sell_prices {
            if let Some(kind) = forage_name_to_kind(name) {
                sell_prices.push((ItemKind::Forage(kind), price));
            }
        }

        // Add fish
        for (name, &price) in &self.config.fish.sell_prices {
            if let Some(kind) = fish_name_to_kind(name) {
                sell_prices.push((ItemKind::Fish(kind), price));
            }
        }

        // Add ore
        for (name, &price) in &self.config.ore.sell_prices {
            if let Some(kind) = ore_name_to_kind(name) {
                sell_prices.push((ItemKind::Ore(kind), price));
            }
        }
        // Add eggs, milk, fiber
        sell_prices.push((ItemKind::Egg, 50));
        sell_prices.push((ItemKind::Milk, 75));
        sell_prices.push((ItemKind::Fiber, 5));

        for (item, price) in sell_prices {
            let qty = self.player.inventory.count(&item);
            if qty > 0 {
                if let Ok(result) = farming::ship_item(&mut self.player, &item, qty, price) {
                    if let crate::game::farming::ActionResult::Shipped { gold } = result {
                        self.pending_gold += gold;
                        self.ships_today += qty;
                    }
                }
            }
        }
    }

    /// Build the ship manifest from inventory and open the shipping selection UI.
    pub fn open_ship_select(&mut self) {
        let mut sell_prices: Vec<(ItemKind, u32)> = self
            .config
            .crops
            .iter()
            .filter_map(|(name, cfg)| {
                let item = crop_name_to_item(name)?;
                Some((item, cfg.sell_price))
            })
            .collect();

        for (name, &price) in &self.config.forage.sell_prices {
            if let Some(kind) = forage_name_to_kind(name) {
                sell_prices.push((ItemKind::Forage(kind), price));
            }
        }
        for (name, &price) in &self.config.fish.sell_prices {
            if let Some(kind) = fish_name_to_kind(name) {
                sell_prices.push((ItemKind::Fish(kind), price));
            }
        }
        for (name, &price) in &self.config.ore.sell_prices {
            if let Some(kind) = ore_name_to_kind(name) {
                sell_prices.push((ItemKind::Ore(kind), price));
            }
        }
        // Add eggs, milk, fiber
        sell_prices.push((ItemKind::Egg, 50));
        sell_prices.push((ItemKind::Milk, 75));
        sell_prices.push((ItemKind::Fiber, 5));

        self.ship_manifest.clear();
        for (item, price) in sell_prices {
            let qty = self.player.inventory.count(&item);
            if qty > 0 {
                self.ship_manifest.push((item, qty, price, true));
            }
        }
        // Sort by item label for consistent ordering
        self.ship_manifest.sort_by(|a, b| {
            format!("{:?}", a.0).cmp(&format!("{:?}", b.0))
        });

        if self.ship_manifest.is_empty() {
            self.notify("Nothing to ship!");
            return;
        }
        self.ship_cursor = 0;
        self.phase = GamePhase::ShipSelect;
    }

    /// Toggle whether the item at `ship_cursor` is selected for shipping.
    pub fn ship_toggle(&mut self) {
        if self.ship_cursor < self.ship_manifest.len() {
            self.ship_manifest[self.ship_cursor].3 = !self.ship_manifest[self.ship_cursor].3;
        }
    }

    /// Move the ship cursor up or down.
    pub fn ship_move_cursor(&mut self, delta: i32) {
        // manifest.len() slots for items + 1 for the "Ship" button
        let total = self.ship_manifest.len() + 1;
        if total == 0 { return; }
        let new = self.ship_cursor as i32 + delta;
        self.ship_cursor = new.rem_euclid(total as i32) as usize;
    }

    /// Ship all selected items and close the UI.
    pub fn ship_confirm(&mut self) {
        for i in 0..self.ship_manifest.len() {
            let (ref item, qty, price, selected) = self.ship_manifest[i];
            if selected && qty > 0 {
                if let Ok(result) = farming::ship_item(&mut self.player, item, qty, price) {
                    if let crate::game::farming::ActionResult::Shipped { gold } = result {
                        self.pending_gold += gold;
                        self.ships_today += qty;
                    }
                }
            }
        }
        self.ship_manifest.clear();
        self.phase = GamePhase::Playing;
    }

    /// Move cursor in furniture shop.
    pub fn furniture_move_cursor(&mut self, delta: i32) {
        let total = FurnitureKind::ALL.len();
        if total == 0 { return; }
        let new = self.furniture_cursor as i32 + delta;
        self.furniture_cursor = new.rem_euclid(total as i32) as usize;
    }

    /// Try to buy the selected furniture item.
    pub fn furniture_try_buy(&mut self) {
        let item = FurnitureKind::ALL[self.furniture_cursor];
        if self.owned_furniture.contains(&item) {
            self.notify("Already owned!");
            return;
        }
        let price = self.effective_price(item.price());
        if self.player.gold < price {
            self.notify("Not enough gold!");
            return;
        }
        self.player.gold -= price;
        self.owned_furniture.insert(item);
        self.notify(&format!("Bought {}!", item.name()));
    }

    fn spawn_birds() -> Vec<Bird> {
        // Place small flocks of birds around the map on grass areas
        let spots: &[(usize, usize)] = &[
            (8, 8), (9, 8),           // near farm
            (15, 14), (16, 14),       // farm center
            (6, 18), (7, 18),         // south farm
            (30, 10), (31, 10),       // east farm
            (20, 6), (21, 6),         // north farm
            (45, 15), (46, 15),       // town park
            (55, 17), (56, 17),       // near playground
            (70, 14), (71, 14),       // east town
            (12, 30), (13, 30),       // south wilderness
            (50, 45), (51, 45),       // far south
            (35, 25), (36, 25),       // mid area
            (4, 34),                  // forest south
            (74, 28),                 // east path
        ];
        spots.iter().enumerate().map(|(i, &(col, row))| {
            Bird::new(col, row, (i % 3) as u8)
        }).collect()
    }

    /// Update birds — fly away when player is close, respawn after a delay.
    pub fn tick_birds(&mut self, dt: f32) {
        let (px, py) = self.player.tile;
        let flee_dist = 3.0; // tiles

        for bird in &mut self.birds {
            if bird.flying {
                bird.fly_timer += dt;
                // Move upward and to the side while flying
                bird.y -= dt * 120.0;
                bird.x += dt * (30.0 - bird.variant as f32 * 20.0);
                // After 1.5 seconds of flight, start respawn timer
                if bird.fly_timer > 1.5 {
                    bird.flying = false;
                    bird.respawn_timer = 8.0 + (bird.variant as f32) * 3.0; // 8-14 seconds
                }
            } else if bird.respawn_timer > 0.0 {
                bird.respawn_timer -= dt;
                if bird.respawn_timer <= 0.0 {
                    // Respawn at home
                    bird.x = bird.home_col as f32 * 32.0 + 8.0 + (bird.variant as f32 * 7.0) % 16.0;
                    bird.y = bird.home_row as f32 * 32.0 + 10.0 + (bird.variant as f32 * 11.0) % 12.0;
                    bird.respawn_timer = 0.0;
                }
            } else {
                // Check player distance
                let dx = (bird.home_col as f32 - px as f32).abs();
                let dy = (bird.home_row as f32 - py as f32).abs();
                if dx <= flee_dist && dy <= flee_dist {
                    bird.flying = true;
                    bird.fly_timer = 0.0;
                }
            }
        }
    }

    /// Spawn and update squirrels from oak trees.
    pub fn tick_squirrels(&mut self, dt: f32) {
        self.squirrel_timer += dt;

        // Spawn a new squirrel every 5 seconds from a random oak tree
        if self.squirrel_timer >= 5.0 && self.squirrels.len() < 6 {
            self.squirrel_timer = 0.0;
            // Find oak tree tiles
            let mut oaks = Vec::new();
            for row in 0..self.map.height {
                for col in 0..self.map.width {
                    if let Some(tile) = self.map.get(col, row) {
                        if matches!(tile.kind, crate::game::world::TileKind::OakTree | crate::game::world::TileKind::OakTreeEmpty) {
                            oaks.push((col, row));
                        }
                    }
                }
            }
            if !oaks.is_empty() {
                // Pick a pseudo-random oak
                let seed = (self.clock.minute as usize * 31 + self.clock.hour as usize * 97 + self.squirrels.len() * 53) % oaks.len();
                let (col, row) = oaks[seed];
                let home_x = col as f32 * 32.0 + 16.0;
                let home_y = row as f32 * 32.0 + 16.0;
                // Pick a random direction to run (2 tiles = 64 pixels)
                let dir_seed = (col * 7 + row * 13 + self.clock.minute as usize) % 4;
                let (dx, dy) = match dir_seed {
                    0 => (64.0f32, 0.0f32),
                    1 => (-64.0, 0.0),
                    2 => (0.0, 64.0),
                    _ => (0.0, -64.0),
                };
                self.squirrels.push(Squirrel {
                    x: home_x,
                    y: home_y,
                    home_x,
                    home_y,
                    target_x: home_x + dx,
                    target_y: home_y + dy,
                    phase: 0,
                    timer: 0.0,
                    speed: 120.0 + (seed as f32 % 3.0) * 20.0,
                });
            }
        }

        // Update each squirrel
        let mut remove = Vec::new();
        for (i, sq) in self.squirrels.iter_mut().enumerate() {
            sq.timer += dt;
            match sq.phase {
                0 => {
                    // Running out to target
                    let dx = sq.target_x - sq.x;
                    let dy = sq.target_y - sq.y;
                    let dist = (dx * dx + dy * dy).sqrt();
                    if dist < 2.0 {
                        sq.phase = 1;
                        sq.timer = 0.0;
                    } else {
                        let step = sq.speed * dt;
                        sq.x += dx / dist * step;
                        sq.y += dy / dist * step;
                    }
                }
                1 => {
                    // Pausing (look around for 0.8 seconds)
                    if sq.timer > 0.8 {
                        sq.phase = 2;
                        sq.timer = 0.0;
                    }
                }
                2 => {
                    // Running back to tree
                    let dx = sq.home_x - sq.x;
                    let dy = sq.home_y - sq.y;
                    let dist = (dx * dx + dy * dy).sqrt();
                    if dist < 2.0 {
                        sq.phase = 3;
                    } else {
                        let step = sq.speed * dt;
                        sq.x += dx / dist * step;
                        sq.y += dy / dist * step;
                    }
                }
                _ => {
                    remove.push(i);
                }
            }
        }
        // Remove finished squirrels
        for i in remove.into_iter().rev() {
            self.squirrels.remove(i);
        }
    }

    /// Check if today is a festival day and trigger the announcement.
    pub fn check_festival(&mut self) {
        const FESTIVAL_DAY: u8 = 14;
        if self.clock.day == FESTIVAL_DAY {
            if let Some(kind) = FestivalKind::for_season(&self.clock.season) {
                self.festival_kind = Some(kind);
                self.phase = GamePhase::FestivalAnnounce;
            }
        }
    }

    /// Start the festival minigame: build the grid and hide items.
    pub fn festival_start(&mut self) {
        let kind = match self.festival_kind {
            Some(k) => k,
            None => return,
        };
        const COLS: usize = 8;
        const ROWS: usize = 6;
        // Build empty grid
        self.festival_grid = vec![vec![false; COLS]; ROWS];
        self.festival_revealed = vec![vec![false; COLS]; ROWS];
        self.festival_cursor = (0, 0);
        self.festival_found = 0;
        self.festival_searches_left = kind.max_searches();
        self.festival_prize = 0;

        // Randomly place items using a simple PRNG seeded from game state
        let mut seed = (self.clock.day as u64)
            .wrapping_mul(31)
            .wrapping_add(self.clock.year as u64 * 127)
            .wrapping_add(self.clock.season.name().len() as u64 * 53);
        let mut placed = 0usize;
        while placed < kind.hidden_count() {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let col = (seed >> 16) as usize % COLS;
            let row = (seed >> 24) as usize % ROWS;
            if !self.festival_grid[row][col] {
                self.festival_grid[row][col] = true;
                placed += 1;
            }
        }

        self.phase = GamePhase::FestivalPlaying;
    }

    /// Move the festival cursor.
    pub fn festival_move(&mut self, dx: i32, dy: i32) {
        let cols = if self.festival_grid.is_empty() { 8 } else { self.festival_grid[0].len() };
        let rows = self.festival_grid.len().max(1);
        let (cx, cy) = self.festival_cursor;
        self.festival_cursor = (
            (cx as i32 + dx).rem_euclid(cols as i32) as usize,
            (cy as i32 + dy).rem_euclid(rows as i32) as usize,
        );
    }

    /// Search the tile under the cursor.
    pub fn festival_search(&mut self) {
        if self.festival_searches_left == 0 { return; }
        let (col, row) = self.festival_cursor;
        if row >= self.festival_revealed.len() || col >= self.festival_revealed[0].len() { return; }
        if self.festival_revealed[row][col] { return; } // already searched

        self.festival_revealed[row][col] = true;
        self.festival_searches_left -= 1;

        if self.festival_grid[row][col] {
            self.festival_found += 1;
        }

        // Auto-end if no searches left
        if self.festival_searches_left == 0 {
            self.festival_end();
        }
    }

    /// End the festival and calculate prizes.
    pub fn festival_end(&mut self) {
        if let Some(kind) = self.festival_kind {
            self.festival_prize = self.festival_found * kind.prize_per_item();
            self.player.gold += self.festival_prize;
            self.phase = GamePhase::FestivalResults;
        }
    }

    /// Dismiss the festival results screen.
    pub fn festival_dismiss(&mut self) {
        self.festival_kind = None;
        self.festival_grid.clear();
        self.festival_revealed.clear();
        self.phase = GamePhase::Playing;
    }

    /// Close furniture shop.
    pub fn furniture_close(&mut self) {
        self.phase = GamePhase::Playing;
    }

    /// Move cursor in animal shop.
    /// Total items in the animal shop (animals + equestrian center if not owned).
    pub fn animal_shop_count(&self) -> usize {
        AnimalKind::ALL.len() + if self.has_equestrian_center { 0 } else { 1 }
    }

    pub fn animal_move_cursor(&mut self, delta: i32) {
        let total = self.animal_shop_count();
        if total == 0 { return; }
        let new = self.animal_cursor as i32 + delta;
        self.animal_cursor = new.rem_euclid(total as i32) as usize;
    }

    /// Try to buy the selected animal or equestrian center.
    pub fn animal_try_buy(&mut self) {
        let animal_count = AnimalKind::ALL.len();
        if self.animal_cursor < animal_count {
            // Buying an animal
            let item = AnimalKind::ALL[self.animal_cursor];
            if self.owned_animals.contains(&item) {
                self.notify("Already owned!");
                return;
            }
            let price = self.effective_price(item.price());
            if self.player.gold < price {
                self.notify("Not enough gold!");
                return;
            }
            self.player.gold -= price;
            self.owned_animals.insert(item);
            self.notify(&format!("Bought a {}!", item.name()));
        } else {
            // Buying equestrian center
            if self.has_equestrian_center {
                self.notify("Already built!");
                return;
            }
            let price = self.effective_price(10000);
            if self.player.gold < price {
                self.notify("Not enough gold!");
                return;
            }
            self.player.gold -= price;
            self.has_equestrian_center = true;
            self.notify("Equestrian Center built!");
        }
    }

    /// Try to scythe the facing tile (or the tile the player stands on).
    pub fn try_scythe(&mut self) -> Result<(), ActionError> {
        let cost = 2i16;
        if !self.player.spend_energy(cost) {
            return Err(ActionError::NotEnoughEnergy);
        }
        let facing = self.player.facing_tile();
        let standing = self.player.tile;
        // Check facing tile first, then standing tile
        let target = if self.map.get(facing.0, facing.1)
            .map(|t| t.kind == crate::game::world::TileKind::LongGrass)
            .unwrap_or(false)
        {
            facing
        } else if self.map.get(standing.0, standing.1)
            .map(|t| t.kind == crate::game::world::TileKind::LongGrass)
            .unwrap_or(false)
        {
            standing
        } else {
            // Refund energy if no long grass found
            self.player.energy = (self.player.energy + cost).min(self.player.max_energy);
            return Err(ActionError::InvalidTile);
        };
        self.map.tiles[target.1][target.0].kind = crate::game::world::TileKind::Grass;
        // 1-3 fiber per cut
        let seed = (target.0.wrapping_mul(13).wrapping_add(target.1.wrapping_mul(7))
            .wrapping_add(self.clock.day as usize)) % 3 + 1;
        self.player.inventory.add(crate::game::inventory::ItemKind::Fiber, seed as u32);
        self.notify(&format!("+{} fiber", seed));
        Ok(())
    }

    /// Named locations the horse can travel to.
    pub fn horse_destinations() -> &'static [(&'static str, usize, usize)] {
        &[
            ("Farmhouse",    2,   4),
            ("Shop",         37,  4),
            ("Shipping Box", 1,   28),
            ("Inn",          44,  6),
            ("Market",       52,  6),
            ("Tavern",       60,  6),
            ("Clinic",       68,  6),
            ("Library",      44,  13),
            ("Furniture Shop",52, 13),
            ("Town Hall",    70,  13),
            ("Animal Shop",  66,  20),
            ("Arcade",       52,  20),
            ("Restaurant",   44,  20),
            ("Pool",         60,  20),
            ("Playground",   55,  13),
            ("Mine",         23,  4),
            ("South Lake",   28,  32),
            ("East Meadow",  103, 6),
            ("East Pond",    103, 14),
            ("Horse Stable", 12,  17),
            ("Beach",        96,  58),
        ]
    }

    /// Open the horse destination picker.
    pub fn open_horse_destinations(&mut self) {
        if !self.riding_horse { return; }
        self.horse_dest_cursor = 0;
        self.phase = GamePhase::HorseDestination;
    }

    /// Confirm destination selection, compute A* path, and start autopilot.
    pub fn confirm_horse_destination(&mut self) {
        let dests = Self::horse_destinations();
        if self.horse_dest_cursor < dests.len() {
            let (name, col, row) = dests[self.horse_dest_cursor];
            let start = self.player.tile;
            let goal = (col, row);
            match Self::astar_path(&self.map, start, goal) {
                Some(path) => {
                    self.horse_target = Some(goal);
                    self.horse_path = path;
                    self.phase = GamePhase::Playing;
                    self.notify(&format!("Horse heading to {}...", name));
                }
                None => {
                    self.phase = GamePhase::Playing;
                    self.notify("No path found!");
                }
            }
        }
    }

    /// Set autopilot destination by name. Mounts horse automatically if near it.
    /// Returns true if the destination was found and path computed.
    pub fn ride_to(&mut self, dest_name: &str) -> bool {
        let dests = Self::horse_destinations();
        let needle = dest_name.to_lowercase();
        let found = dests.iter().find(|(name, _, _)| name.to_lowercase() == needle);
        let (name, col, row) = match found {
            Some(d) => *d,
            None => {
                // Try partial match
                match dests.iter().find(|(name, _, _)| name.to_lowercase().contains(&needle)) {
                    Some(d) => *d,
                    None => {
                        self.notify(&format!("Unknown destination: {}", dest_name));
                        return false;
                    }
                }
            }
        };

        // Auto-mount if not riding
        if !self.riding_horse {
            if self.owned_animals.contains(&crate::game::state::AnimalKind::Horse) {
                self.riding_horse = true;
            } else {
                self.notify("You don't own a horse!");
                return false;
            }
        }

        let start = self.player.tile;
        let goal = (col, row);
        match Self::astar_path(&self.map, start, goal) {
            Some(path) => {
                self.horse_target = Some(goal);
                self.horse_path = path;
                self.horse_blocked_count = 0;
                self.phase = GamePhase::Playing;
                self.notify(&format!("Horse heading to {}...", name));
                true
            }
            None => {
                self.notify("No path found!");
                false
            }
        }
    }

    /// A* pathfinding on the tile map. Returns a list of tiles from start to goal
    /// (excluding start, including goal). Ignores NPCs (they move).
    fn astar_path(
        map: &crate::game::world::FarmMap,
        start: (usize, usize),
        goal: (usize, usize),
    ) -> Option<Vec<(usize, usize)>> {
        use std::collections::{BinaryHeap, HashMap};
        use std::cmp::Reverse;

        let w = map.width;
        let h = map.height;

        // Each node: (Reverse(f_score), g_score, (col, row))
        let mut open: BinaryHeap<(Reverse<i32>, i32, (usize, usize))> = BinaryHeap::new();
        let mut came_from: HashMap<(usize, usize), (usize, usize)> = HashMap::new();
        let mut g_score: HashMap<(usize, usize), i32> = HashMap::new();

        let heuristic = |a: (usize, usize), b: (usize, usize)| -> i32 {
            (a.0 as i32 - b.0 as i32).abs() + (a.1 as i32 - b.1 as i32).abs()
        };

        g_score.insert(start, 0);
        open.push((Reverse(heuristic(start, goal)), 0, start));

        // Limit search to prevent freezing on huge maps
        let mut iterations = 0u32;
        const MAX_ITERATIONS: u32 = 20000;

        while let Some((_, g, current)) = open.pop() {
            iterations += 1;
            if iterations > MAX_ITERATIONS {
                return None; // give up — too far or unreachable
            }

            if current == goal {
                // Reconstruct path
                let mut path = Vec::new();
                let mut node = goal;
                while node != start {
                    path.push(node);
                    node = *came_from.get(&node).unwrap();
                }
                path.reverse();
                return Some(path);
            }

            // Skip if we already found a better path to this node
            if g > *g_score.get(&current).unwrap_or(&i32::MAX) {
                continue;
            }

            let (cx, cy) = current;
            let neighbors: [(usize, usize); 4] = [
                (cx, cy.wrapping_sub(1)),                    // up
                (cx, (cy + 1).min(h - 1)),                   // down
                (cx.wrapping_sub(1), cy),                    // left
                ((cx + 1).min(w - 1), cy),                   // right
            ];

            for &next in &neighbors {
                if next == current { continue; }
                if next.0 >= w || next.1 >= h { continue; }

                // Check passability — allow the goal tile even if it's not normally passable
                // (e.g. standing next to a building door)
                let passable = if next == goal {
                    true
                } else if let Some(tile) = map.get(next.0, next.1) {
                    tile.kind.is_passable()
                } else {
                    false
                };
                if !passable { continue; }

                let tentative_g = g + 1;
                if tentative_g < *g_score.get(&next).unwrap_or(&i32::MAX) {
                    came_from.insert(next, current);
                    g_score.insert(next, tentative_g);
                    let f = tentative_g + heuristic(next, goal);
                    open.push((Reverse(f), tentative_g, next));
                }
            }
        }

        None // no path found
    }

    /// A* that also treats NPC tiles as impassable (for rerouting around them).
    fn astar_path_avoiding_npcs(
        map: &crate::game::world::FarmMap,
        start: (usize, usize),
        goal: (usize, usize),
        npc_tiles: &[(usize, usize)],
    ) -> Option<Vec<(usize, usize)>> {
        use std::collections::{BinaryHeap, HashMap, HashSet};
        use std::cmp::Reverse;

        let blocked: HashSet<(usize, usize)> = npc_tiles.iter().copied().collect();
        let w = map.width;
        let h = map.height;

        let mut open: BinaryHeap<(Reverse<i32>, i32, (usize, usize))> = BinaryHeap::new();
        let mut came_from: HashMap<(usize, usize), (usize, usize)> = HashMap::new();
        let mut g_score: HashMap<(usize, usize), i32> = HashMap::new();

        let heuristic = |a: (usize, usize), b: (usize, usize)| -> i32 {
            (a.0 as i32 - b.0 as i32).abs() + (a.1 as i32 - b.1 as i32).abs()
        };

        g_score.insert(start, 0);
        open.push((Reverse(heuristic(start, goal)), 0, start));

        let mut iterations = 0u32;
        const MAX_ITERATIONS: u32 = 20000;

        while let Some((_, g, current)) = open.pop() {
            iterations += 1;
            if iterations > MAX_ITERATIONS { return None; }

            if current == goal {
                let mut path = Vec::new();
                let mut node = goal;
                while node != start {
                    path.push(node);
                    node = *came_from.get(&node).unwrap();
                }
                path.reverse();
                return Some(path);
            }

            if g > *g_score.get(&current).unwrap_or(&i32::MAX) { continue; }

            let (cx, cy) = current;
            let neighbors = [
                (cx, cy.wrapping_sub(1)),
                (cx, (cy + 1).min(h - 1)),
                (cx.wrapping_sub(1), cy),
                ((cx + 1).min(w - 1), cy),
            ];

            for &next in &neighbors {
                if next == current || next.0 >= w || next.1 >= h { continue; }
                if blocked.contains(&next) && next != goal { continue; }

                let passable = if next == goal {
                    true
                } else if let Some(tile) = map.get(next.0, next.1) {
                    tile.kind.is_passable()
                } else {
                    false
                };
                if !passable { continue; }

                let tentative_g = g + 1;
                if tentative_g < *g_score.get(&next).unwrap_or(&i32::MAX) {
                    came_from.insert(next, current);
                    g_score.insert(next, tentative_g);
                    let f = tentative_g + heuristic(next, goal);
                    open.push((Reverse(f), tentative_g, next));
                }
            }
        }
        None
    }

    /// Step the horse one tile along the pre-computed A* path. Returns true if it moved.
    pub fn horse_autopilot_step(&mut self) -> bool {
        if self.horse_target.is_none() || self.horse_path.is_empty() {
            return false;
        }
        if !self.riding_horse {
            self.horse_target = None;
            self.horse_path.clear();
            return false;
        }

        let next_tile = self.horse_path[0];

        // Check if the next tile is blocked by an NPC
        let blocked = self.npcs.iter().any(|n| n.tile == next_tile);
        if blocked {
            self.horse_blocked_count += 1;
            // After 8 blocked steps (~1 second), recompute path avoiding NPCs
            if self.horse_blocked_count >= 8 {
                self.horse_blocked_count = 0;
                let start = self.player.tile;
                let goal = self.horse_target.unwrap();
                let npc_tiles: Vec<(usize, usize)> = self.npcs.iter().map(|n| n.tile).collect();
                if let Some(new_path) = Self::astar_path_avoiding_npcs(&self.map, start, goal, &npc_tiles) {
                    self.horse_path = new_path;
                }
                // If repath also fails, the next attempt will try again
            }
            return false;
        }

        self.horse_blocked_count = 0;

        // Determine facing direction
        let (px, py) = self.player.tile;
        let (nx, ny) = next_tile;
        let dir = if nx > px { Direction::Right }
            else if nx < px { Direction::Left }
            else if ny > py { Direction::Down }
            else { Direction::Up };

        self.player.tile = next_tile;
        self.player.facing = dir;
        self.horse_path.remove(0);

        // Arrived?
        if self.horse_path.is_empty() {
            self.horse_target = None;
            self.notify("You have arrived!");
            crate::game::save::play_sound("horse");
        }

        true
    }

    /// Toggle riding the horse. Must own a horse and be near it to mount.
    pub fn toggle_horse(&mut self) {
        if self.riding_horse {
            self.riding_horse = false;
            self.horse_target = None;
            self.horse_path.clear();
            self.notify("You dismounted.");
            return;
        }
        if !self.owned_animals.contains(&AnimalKind::Horse) {
            self.notify("You don't own a horse!");
            return;
        }
        let horse_tile = AnimalKind::Horse.farm_tile();
        let (px, py) = self.player.tile;
        let dx = (px as i32 - horse_tile.0 as i32).abs();
        let dy = (py as i32 - horse_tile.1 as i32).abs();
        if dx <= 2 && dy <= 2 {
            self.riding_horse = true;
            self.notify("Giddyup!");
        } else {
            self.notify("Get closer to your horse!");
        }
    }

    /// Apply rainbow day discount (50% off).
    pub fn effective_price(&self, price: u32) -> u32 {
        if self.rainbow_day { price / 2 } else { price }
    }

    /// Move cursor in outfit shop.
    pub fn outfit_move_cursor(&mut self, delta: i32) {
        let total = OUTFITS.len();
        let new = self.outfit_cursor as i32 + delta;
        self.outfit_cursor = new.rem_euclid(total as i32) as usize;
    }

    /// Buy or equip the selected outfit.
    pub fn outfit_try_buy(&mut self) {
        let idx = self.outfit_cursor as u8;
        let outfit = &OUTFITS[idx as usize];
        if self.owned_outfits.contains(&idx) {
            // Already owned — equip it
            self.player.outfit = idx;
            self.notify(&format!("Wearing {}!", outfit.name));
            crate::game::save::play_sound("buy");
        } else {
            // Buy it
            let price = self.effective_price(outfit.price);
            if self.player.gold < price {
                self.notify("Not enough gold!");
                return;
            }
            self.player.gold -= price;
            self.owned_outfits.insert(idx);
            self.player.outfit = idx;
            self.notify(&format!("Bought & wearing {}!", outfit.name));
            crate::game::save::play_sound("buy");
        }
    }

    /// Toggle player gender.
    pub fn toggle_gender(&mut self) {
        self.player.gender = if self.player.gender == 0 { 1 } else { 0 };
        let label = if self.player.gender == 0 { "Male" } else { "Female" };
        self.notify(&format!("Gender: {}", label));
    }

    /// Close outfit shop.
    pub fn outfit_close(&mut self) {
        self.phase = GamePhase::Playing;
    }

    /// Create a multiplayer room as host.
    pub fn mp_host(&mut self) {
        let code = crate::game::save::mp_create_room();
        if !code.is_empty() {
            self.mp_role = "host".to_string();
            self.mp_room = code.clone();
            self.coop_active = true;
            self.player2.tile = (self.player.tile.0 + 1, self.player.tile.1);
            self.player2.energy = self.player.max_energy;
            self.notify(&format!("Hosting! Code: {}", code));
        } else {
            self.notify("Failed to create room.");
        }
    }

    /// Join a multiplayer room as guest.
    pub fn mp_join(&mut self, code: &str) {
        let result = crate::game::save::mp_join_room(code);
        if result == "ok" {
            self.mp_role = "guest".to_string();
            self.mp_room = code.to_uppercase();
            self.coop_active = true;
            self.notify(&format!("Joined room {}!", code.to_uppercase()));
        } else {
            self.notify("Room not found!");
        }
    }

    /// Disconnect from multiplayer.
    pub fn mp_disconnect(&mut self) {
        self.mp_role = "none".to_string();
        self.mp_room.clear();
        self.notify("Disconnected from multiplayer.");
    }

    /// Host: send state snapshot to server. Guest: apply received state.
    pub fn mp_tick(&mut self, dt: f32) {
        if self.mp_role == "none" { return; }

        if self.mp_role == "host" {
            self.mp_sync_timer += dt;
            if self.mp_sync_timer >= 0.1 {
                self.mp_sync_timer = 0.0;
                // Build compact state snapshot
                let snap = format!(
                    r#"{{"p1":[{},{}],"p1f":{},"p2":[{},{}],"p2f":{},"day":{},"hour":{},"min":{},"season":"{}","year":{}}}"#,
                    self.player.tile.0, self.player.tile.1,
                    match self.player.facing { Direction::Up=>0, Direction::Down=>1, Direction::Left=>2, Direction::Right=>3 },
                    self.player2.tile.0, self.player2.tile.1,
                    match self.player2.facing { Direction::Up=>0, Direction::Down=>1, Direction::Left=>2, Direction::Right=>3 },
                    self.clock.day, self.clock.hour, self.clock.minute,
                    self.clock.season.name(), self.clock.year,
                );
                crate::game::save::mp_sync_state(&snap);

                // Read guest inputs
                let inputs_json = crate::game::save::mp_read();
                if inputs_json.len() > 2 {
                    // Parse input array and apply guest movements
                    // Simple: each input is a direction string
                    if let Ok(inputs) = serde_json::from_str::<Vec<String>>(&inputs_json) {
                        for input in inputs {
                            match input.as_str() {
                                "up"    => self.move_player2(Direction::Up),
                                "down"  => self.move_player2(Direction::Down),
                                "left"  => self.move_player2(Direction::Left),
                                "right" => self.move_player2(Direction::Right),
                                _ => {}
                            }
                        }
                    }
                }
            }
        } else if self.mp_role == "guest" {
            // Apply received state
            let state_json = crate::game::save::mp_read();
            if state_json.len() > 2 && !state_json.contains("error") {
                if let Ok(snap) = serde_json::from_str::<serde_json::Value>(&state_json) {
                    if let (Some(p1), Some(p2)) = (snap.get("p1"), snap.get("p2")) {
                        if let (Some(x), Some(y)) = (p1.get(0), p1.get(1)) {
                            self.player.tile = (x.as_u64().unwrap_or(10) as usize, y.as_u64().unwrap_or(10) as usize);
                        }
                        if let (Some(x), Some(y)) = (p2.get(0), p2.get(1)) {
                            self.player2.tile = (x.as_u64().unwrap_or(12) as usize, y.as_u64().unwrap_or(10) as usize);
                        }
                    }
                    if let Some(day) = snap.get("day").and_then(|v| v.as_u64()) {
                        self.clock.day = day as u8;
                    }
                    if let Some(hour) = snap.get("hour").and_then(|v| v.as_u64()) {
                        self.clock.hour = hour as u8;
                    }
                    if let Some(min) = snap.get("min").and_then(|v| v.as_u64()) {
                        self.clock.minute = min as u8;
                    }
                }
            }
        }
    }

    /// Guest: send an input event to the host.
    pub fn mp_send_input(&self, input: &str) {
        if self.mp_role == "guest" {
            crate::game::save::mp_send_input(&format!("\"{}\"", input));
        }
    }

    /// Italian restaurant menu items: (name, price, energy_restored).
    pub const MENU: &'static [(&'static str, u32, i16)] = &[
        ("Margherita Pizza", 50, 60),
        ("Spaghetti Bolognese", 40, 50),
        ("Lasagna", 65, 80),
        ("Risotto", 55, 65),
        ("Tiramisu", 30, 35),
        ("Bruschetta", 20, 25),
        ("Ravioli", 45, 55),
        ("Gelato", 15, 20),
    ];

    /// Open the restaurant menu.
    pub fn open_restaurant(&mut self) {
        self.restaurant_cursor = 0;
        self.phase = GamePhase::RestaurantOpen;
    }

    /// Move cursor in restaurant menu.
    pub fn restaurant_move_cursor(&mut self, delta: i32) {
        let total = Self::MENU.len();
        let new = self.restaurant_cursor as i32 + delta;
        self.restaurant_cursor = new.rem_euclid(total as i32) as usize;
    }

    /// Order the selected food item.
    pub fn restaurant_order(&mut self) {
        let (name, price, energy) = Self::MENU[self.restaurant_cursor];
        let eff_price = self.effective_price(price);
        if self.player.gold < eff_price {
            self.notify("Not enough gold!");
            return;
        }
        self.player.gold -= eff_price;
        self.player.energy = (self.player.energy + energy).min(self.player.max_energy);
        self.notify(&format!("Delicious {}! +{} energy", name, energy));
        crate::game::save::play_sound("buy");
    }

    /// Close restaurant.
    pub fn restaurant_close(&mut self) {
        self.phase = GamePhase::Playing;
    }

    // ── Ice Cream Shop ───────────────────────────────────────────────
    pub const ICE_CREAM_MENU: &'static [(&'static str, u32, i16)] = &[
        ("Vanilla Cone",     10, 15),
        ("Strawberry Swirl", 15, 20),
        ("Chocolate Sundae", 20, 30),
        ("Mint Chip",        15, 20),
        ("Mango Sorbet",     20, 25),
        ("Cookie Dough",     25, 35),
    ];

    pub fn open_icecream(&mut self) {
        if self.clock.season != crate::game::time::Season::Summer {
            self.notify("Closed for the season! Come back in Summer.");
            return;
        }
        self.icecream_cursor = 0;
        self.phase = GamePhase::IceCreamShopOpen;
    }

    pub fn icecream_move_cursor(&mut self, delta: i32) {
        let total = Self::ICE_CREAM_MENU.len();
        let new = self.icecream_cursor as i32 + delta;
        self.icecream_cursor = new.rem_euclid(total as i32) as usize;
    }

    pub fn icecream_order(&mut self) {
        let (name, price, energy) = Self::ICE_CREAM_MENU[self.icecream_cursor];
        let eff_price = self.effective_price(price);
        if self.player.gold < eff_price {
            self.notify("Not enough gold!");
            return;
        }
        self.player.gold -= eff_price;
        self.player.energy = (self.player.energy + energy).min(self.player.max_energy);
        self.notify(&format!("Yummy {}! +{} energy", name, energy));
        crate::game::save::play_sound("buy");
    }

    pub fn icecream_close(&mut self) {
        self.phase = GamePhase::Playing;
    }

    /// Start the arcade reaction game.
    pub fn start_arcade(&mut self) {
        self.arcade_phase = 0; // waiting
        self.arcade_timer = 0.0;
        // Random delay 1-4 seconds before light turns green
        let seed = (self.clock.day as f32 * 17.3 + self.clock.minute as f32 * 7.1).sin().abs();
        self.arcade_delay = 1.0 + seed * 3.0;
        self.arcade_reaction = 0.0;
        self.arcade_prize = 0;
        self.phase = GamePhase::ArcadePlaying;
        crate::game::save::play_sound("notify");
    }

    /// Tick the arcade game timer.
    pub fn tick_arcade(&mut self, dt: f32) {
        if self.phase != GamePhase::ArcadePlaying { return; }
        self.arcade_timer += dt;
        if self.arcade_phase == 0 && self.arcade_timer >= self.arcade_delay {
            // Light turns green!
            self.arcade_phase = 1;
            self.arcade_timer = 0.0; // reset for reaction timing
            crate::game::save::play_sound("notify");
        }
    }

    /// Player presses E during arcade — react to the green light.
    pub fn arcade_react(&mut self) {
        if self.arcade_phase == 0 {
            // Pressed too early!
            self.arcade_phase = 2;
            self.arcade_reaction = 0.0;
            self.arcade_prize = 0;
            self.notify("Too early! No prize.");
            crate::game::save::play_sound("fishFail");
        } else if self.arcade_phase == 1 {
            // Got it!
            self.arcade_reaction = self.arcade_timer;
            self.arcade_phase = 2;
            // Prize based on reaction time
            self.arcade_prize = if self.arcade_reaction < 0.2 {
                500 // incredible
            } else if self.arcade_reaction < 0.4 {
                200 // great
            } else if self.arcade_reaction < 0.7 {
                100 // good
            } else if self.arcade_reaction < 1.0 {
                50 // ok
            } else {
                10 // slow
            };
            self.player.gold += self.arcade_prize;
            crate::game::save::play_sound("fishCatch");
        }
    }

    /// Close arcade and return to playing.
    pub fn arcade_close(&mut self) {
        self.phase = GamePhase::Playing;
    }

    /// Open the arena jump editor.
    pub fn open_arena_editor(&mut self) {
        if !self.has_equestrian_center {
            self.notify("Build the Equestrian Center first!");
            return;
        }
        self.arena_cursor = (0, 0);
        self.phase = GamePhase::ArenaEditor;
    }

    /// Move cursor in arena editor.
    pub fn arena_move_cursor(&mut self, dx: i32, dy: i32) {
        self.arena_cursor.0 = (self.arena_cursor.0 as i32 + dx).clamp(0, 8) as u8;
        self.arena_cursor.1 = (self.arena_cursor.1 as i32 + dy).clamp(0, 3) as u8;
    }

    /// Place, rotate, or remove a jump at the cursor position.
    /// First press places horizontal. Subsequent presses cycle: horizontal → vertical → diagonal-right → diagonal-left → remove.
    pub fn arena_toggle_jump(&mut self) {
        let (c, r) = (self.arena_cursor.0, self.arena_cursor.1);
        if let Some(idx) = self.arena_jumps.iter().position(|j| j.0 == c && j.1 == r) {
            let orient = self.arena_jumps[idx].2;
            if orient < 3 {
                // Cycle to next orientation
                self.arena_jumps[idx].2 = orient + 1;
            } else {
                // Remove after last orientation
                self.arena_jumps.remove(idx);
            }
        } else if self.arena_jumps.len() < 6 {
            self.arena_jumps.push((c, r, 0)); // place horizontal
        } else {
            self.notify("Max 6 jumps!");
        }
    }

    /// Close arena editor.
    pub fn close_arena_editor(&mut self) {
        self.phase = GamePhase::Playing;
    }

    /// Make the horse leap forward 2 tiles, clearing obstacles.
    pub fn horse_leap(&mut self) {
        if !self.riding_horse || self.horse_leap_timer > 0.0 { return; }

        let (col, row) = self.player.tile;
        let (dx, dy): (i32, i32) = match self.player.facing {
            crate::game::player::Direction::Up    => (0, -2),
            crate::game::player::Direction::Down  => (0, 2),
            crate::game::player::Direction::Left  => (-2, 0),
            crate::game::player::Direction::Right => (2, 0),
        };
        let land_col = (col as i32 + dx).clamp(0, self.map.width as i32 - 1) as usize;
        let land_row = (row as i32 + dy).clamp(0, self.map.height as i32 - 1) as usize;

        // Can only land on a passable tile
        let can_land = self.map.get(land_col, land_row)
            .map(|t| t.kind.is_passable())
            .unwrap_or(false);

        if can_land {
            self.player.tile = (land_col, land_row);
            self.horse_leap_timer = 0.4; // 0.4 second leap animation
        }
    }

    /// Tick the horse leap animation.
    pub fn tick_horse_leap(&mut self, dt: f32) {
        if self.horse_leap_timer > 0.0 {
            self.horse_leap_timer -= dt;
            // Parabolic arc: peak at midpoint of the leap
            let t = 1.0 - (self.horse_leap_timer / 0.4); // 0 to 1
            self.horse_leap_height = -40.0 * (t - 0.5).powi(2) + 10.0; // arc peaking at 10px up
            if self.horse_leap_timer <= 0.0 {
                self.horse_leap_timer = 0.0;
                self.horse_leap_height = 0.0;
            }
        }
    }

    /// Close animal shop.
    pub fn animal_close(&mut self) {
        self.phase = GamePhase::Playing;
    }

    /// Close shipping UI without shipping.
    pub fn ship_cancel(&mut self) {
        self.ship_manifest.clear();
        self.phase = GamePhase::Playing;
    }

    /// Interact with whatever the player is adjacent to.
    pub fn try_interact(&mut self) {
        // Check standing tile first
        let (pcol, prow) = self.player.tile;

        // Arena editor sign — near the arena entrance (col 29, row 19-20)
        if self.has_equestrian_center && pcol >= 28 && pcol <= 30 && prow >= 19 && prow <= 20 {
            self.open_arena_editor();
            return;
        }

        if let Some(standing) = self.map.get(pcol, prow) {
            if standing.kind == crate::game::world::TileKind::Bench {
                let restore = 20i16;
                self.player.energy = (self.player.energy + restore).min(self.player.max_energy);
                self.notify("You sit and rest for a moment...");
                return;
            }
        }

        let facing = self.player.facing_tile();
        let tile = match self.map.get(facing.0, facing.1) {
            Some(t) => t.kind.clone(),
            None => return,
        };

        match tile {
            crate::game::world::TileKind::Farmhouse => {
                if let Some(kind) = door_at(facing.0, facing.1) {
                    if kind == BuildingKind::FurnitureShop {
                        self.furniture_cursor = 0;
                        self.phase = GamePhase::FurnitureShopOpen;
                    } else if kind == BuildingKind::AnimalShop {
                        self.animal_cursor = 0;
                        self.phase = GamePhase::AnimalShopOpen;
                    } else if kind == BuildingKind::Arcade {
                        self.start_arcade();
                    } else if kind == BuildingKind::Restaurant {
                        self.open_restaurant();
                    } else if kind == BuildingKind::IceCreamShop {
                        self.open_icecream();
                    } else {
                        self.current_building = kind;
                        self.phase = GamePhase::FarmhouseInterior;
                        self.farmhouse_tile = (5, 6);
                        self.player.facing = crate::game::player::Direction::Up;
                    }
                }
            }
            crate::game::world::TileKind::ShipBox => {
                self.open_ship_select();
            }
            crate::game::world::TileKind::Shop => {
                // Only the original farm shop (cols 36-39, rows 1-3) opens the shop UI.
                // Market/Clinic are now Farmhouse tiles with interiors.
                self.phase = GamePhase::ShopOpen;
            }
            _ => {
                // Check if adjacent to an NPC
                let npc_idx = self.npcs.iter().position(|n| {
                    let (ncol, nrow) = n.tile;
                    let (pcol, prow) = facing;
                    ncol == pcol && nrow == prow
                });
                if let Some(idx) = npc_idx {
                    let npc = &self.npcs[idx];
                    let p2_victor_hearts = *self.p2_friendships.get(&crate::game::npc::VICTOR_ID).unwrap_or(&0) / 25;
                    let is_victor_win = npc.id == crate::game::npc::VICTOR_ID
                        && (npc.hearts() >= crate::game::npc::VICTOR_FINAL_HEARTS
                            || p2_victor_hearts >= crate::game::npc::VICTOR_FINAL_HEARTS);
                    let url = if is_victor_win {
                        format!(
                            "/api/victor_final?npc={}&season={}&day={}",
                            npc.name, self.clock.season.name(), self.clock.day
                        )
                    } else {
                        format!(
                            "/api/chat?npc_id={}&npc={}&personality={}&friendship={}&season={}&day={}&charisma={}&year={}",
                            npc.id, npc.name, npc.personality, npc.friendship,
                            self.clock.season.name(), self.clock.day,
                            self.player.charisma_level(), self.clock.year
                        )
                    };
                    self.waiting_npc_name = Some(npc.name.clone());
                    self.waiting_npc_idx = Some(idx);
                    self.is_victor_final = is_victor_win;
                    self.pending_llm = Some(LlmRequest { npc_idx: idx, url });
                    self.phase = GamePhase::LlmWaiting;
                }
            }
        }
    }

    /// Try to propose marriage to an adjacent marriageable NPC.
    /// Requires: pendant in inventory + NPC is marriageable + 8+ hearts + not already married.
    pub fn try_propose(&mut self) {
        if self.phase != GamePhase::Playing { return; }

        // Must not already be married.
        if self.married_npc_id.is_some() {
            self.notify("You're already married.");
            return;
        }

        // Must have a pendant.
        if self.player.inventory.count(&ItemKind::Pendant) == 0 {
            self.notify("You need a pendant to propose. Buy one at the shop.");
            return;
        }

        // Find adjacent marriageable NPC (must be opposite gender).
        let facing = self.player.facing_tile();
        let player_gender = self.player.gender;
        let npc_idx = self.npcs.iter().position(|n| {
            n.tile == facing && n.marriageable && npc_gender(&n.name) != player_gender
        });

        let idx = match npc_idx {
            Some(i) => i,
            None => {
                self.notify("No one here to propose to.");
                return;
            }
        };

        let npc = &self.npcs[idx];
        if npc.hearts() < 8 {
            self.notify(format!("You need 8 hearts with {} first.", npc.name));
            return;
        }

        // Consume the pendant.
        self.player.inventory.remove(&ItemKind::Pendant, 1);

        let url = format!(
            "/api/proposal?npc_id={}&npc={}&personality={}&friendship={}&season={}&day={}&charisma={}&year={}",
            npc.id, npc.name, npc.personality, npc.friendship,
            self.clock.season.name(), self.clock.day,
            self.player.charisma_level(), self.clock.year,
        );
        self.waiting_npc_name = Some(npc.name.clone());
        self.waiting_npc_idx = Some(idx);
        self.is_proposal = true;
        self.is_followup_dialogue = false;
        self.pending_llm = Some(LlmRequest { npc_idx: idx, url });
        self.phase = GamePhase::LlmWaiting;
    }

    pub fn advance_dialogue(&mut self) {
        let done = if let Some(dlg) = &mut self.dialogue {
            dlg.advance()
        } else {
            return;
        };

        if done {
            // If this was the initial NPC greeting and we have response options,
            // transition to choice phase (don't close yet).
            if !self.is_followup_dialogue && !self.response_options.is_empty() {
                self.dialogue = None;
                self.choice_cursor = 0;
                self.phase = GamePhase::DialogueChoice;
                return;
            }

            // Otherwise close the dialogue normally.
            self._close_dialogue();
        }
    }

    /// Called internally when a dialogue sequence is fully complete.
    fn _close_dialogue(&mut self) {
        // Capture the NPC's text before clearing so we can persist it.
        let text = self.dialogue.as_ref()
            .and_then(|d| d.lines.first())
            .cloned()
            .unwrap_or_default();

        // Gain conversation friendship (+5 once per day per NPC) and notify
        // on Victor progress.
        if let Some(idx) = self.waiting_npc_idx {
            if let Some(npc) = self.npcs.get_mut(idx) {
                let gained = npc.gain_conversation_friendship(5);
                if gained > 0 && npc.id == crate::game::npc::VICTOR_ID {
                    let hearts = npc.hearts();
                    self.notification = Some((
                        format!("Victor: {}/{} hearts", hearts, crate::game::npc::VICTOR_FINAL_HEARTS),
                        3.0,
                    ));
                } else if gained == 0 && npc.friendship_capped() && !npc.loved_gift_given {
                    self.notification = Some((
                        format!("{} wants a special gift to grow closer. (T to gift)", npc.name),
                        4.0,
                    ));
                }
                if !text.is_empty() {
                    self.pending_memory = Some(MemorySave {
                        npc_id: npc.id,
                        text,
                    });
                }
            }
        }

        // Gain 1 charisma XP per completed NPC conversation.
        self.player.gain_charisma_xp(1);

        // ── Morgan journalist: apply multi-NPC reputation effects ────────────
        if let Some(choice) = self.journalist_choice.take() {
            // Names of NPCs in each attitude group (must match config.json names).
            // 0 = Humble: Victor + resentful group all gain friendship.
            // 1 = Confident: impressed group gains, resentful group loses.
            // 2 = Deflecting: modest gain for the curious / default group.
            let effects: &[(&str, i16)] = match choice {
                0 => &[
                    ("Victor", 10),
                    ("Bram", 8), ("Rin", 8), ("Cass", 8), ("Dex", 8), ("Kit", 8),
                ],
                1 => &[
                    ("Tess", 10), ("Finn", 10), ("Vera", 10), ("Sage", 10),
                    ("Pip", 10), ("Faye", 10), ("Bea", 10), ("Sol", 10),
                    ("Bram", -10), ("Rin", -10), ("Cass", -10), ("Dex", -10), ("Kit", -10),
                ],
                _ => &[
                    ("Elara", 5), ("Maya", 5), ("Suki", 5), ("Moe", 5),
                ],
            };
            for &(name, delta) in effects {
                if let Some(npc) = self.npcs.iter_mut().find(|n| n.name == name) {
                    if delta > 0 {
                        npc.friendship = npc.friendship.saturating_add(delta as u8);
                    } else {
                        npc.friendship = npc.friendship.saturating_sub((-delta) as u8);
                    }
                }
            }
            let attitude = match choice {
                0 => "Humble",
                1 => "Confident",
                _ => "Deflecting",
            };
            self.notification = Some((
                format!("Morgan's article will shape how the valley sees you. ({})", attitude),
                4.0,
            ));
        }

        // ── Marriage proposal outcome ────────────────────────────────────────
        if self.is_proposal {
            self.is_proposal = false;
            if self.last_choice_idx == 0 {
                // Player chose to accept — seal the marriage.
                if let Some(idx) = self.waiting_npc_idx {
                    if let Some(npc) = self.npcs.get(idx) {
                        self.married_npc_id = Some(npc.id);
                        self.notification = Some((
                            format!("You and {} are engaged! Your life in Bennett Valley changes forever.", npc.name),
                            6.0,
                        ));
                    }
                }
            } else {
                self.notification = Some(("They weren't ready yet...".to_string(), 3.0));
            }
        }

        let victor_final = self.is_victor_final;
        self.dialogue = None;
        self.response_options.clear();
        self.waiting_npc_name = None;
        let npc_idx = self.waiting_npc_idx.take();
        self.is_victor_final = false;
        self.is_followup_dialogue = false;
        let _ = npc_idx;

        if victor_final {
            self.phase = GamePhase::Won;
        } else {
            self.phase = GamePhase::Playing;
        }
    }

    /// Move the choice cursor in DialogueChoice phase.
    pub fn move_choice(&mut self, delta: i32) {
        let n = self.response_options.len();
        if n == 0 { return; }
        self.choice_cursor = ((self.choice_cursor as i32 + delta).rem_euclid(n as i32)) as usize;
    }

    /// Confirm the selected choice — fires a follow-up LLM request.
    /// Returns the chosen option text (for display purposes).
    /// Returns None if the choice short-circuits to a local action (e.g., proposal accepted).
    pub fn confirm_choice(&mut self) -> Option<String> {
        let chosen = self.response_options.get(self.choice_cursor)?.clone();
        let npc_idx = self.waiting_npc_idx?;
        let npc = self.npcs.get(npc_idx)?;

        // Record which index was chosen so _close_dialogue can act on it.
        self.last_choice_idx = self.choice_cursor;

        // Record journalist choice index so reputation effects can be applied on close.
        if npc.name == "Morgan" {
            self.journalist_choice = Some(self.choice_cursor);
        }

        let url = format!(
            "/api/chat_reply?npc_id={}&npc={}&personality={}&friendship={}&season={}&day={}&charisma={}&year={}&player_said={}&married={}",
            npc.id, npc.name, npc.personality, npc.friendship,
            self.clock.season.name(), self.clock.day,
            self.player.charisma_level(), self.clock.year,
            url_encode(&chosen),
            self.married_npc_id.map(|id| id == npc.id).unwrap_or(false),
        );

        self.is_followup_dialogue = true;
        self.waiting_npc_name = Some(npc.name.clone());
        self.pending_llm = Some(LlmRequest { npc_idx, url });
        self.phase = GamePhase::LlmWaiting;
        Some(chosen)
    }

    pub fn close_shop(&mut self) {
        self.phase = GamePhase::Playing;
    }

    pub fn exit_farmhouse(&mut self) {
        self.phase = GamePhase::Playing;
        crate::game::save::play_sound("door");
    }

    /// Move the player one step inside the farmhouse. Exits if they walk south past row 7.
    pub fn move_in_farmhouse(&mut self, dir: crate::game::player::Direction) {
        use crate::game::player::Direction;
        self.player.facing = dir;
        let (col, row) = self.farmhouse_tile;
        let (nc, nr) = match dir {
            Direction::Up    => (col,     row - 1),
            Direction::Down  => (col,     row + 1),
            Direction::Left  => (col - 1, row    ),
            Direction::Right => (col + 1, row    ),
        };
        // Walking south past the last row exits the building
        let max_row = if self.current_building == BuildingKind::Farmhouse && self.house_upgraded { 9 } else { 7 };
        let max_col = if self.current_building == BuildingKind::Farmhouse && self.house_upgraded { 15 } else { 11 };
        if nr > max_row {
            self.exit_farmhouse();
            return;
        }
        // Clamp to grid bounds
        if nc < 0 || nc > max_col || nr < 0 {
            return;
        }
        // Furniture collision — depends on which building we're in
        let blocked = match self.current_building {
            BuildingKind::Farmhouse => {
                let base = matches!((nc, nr),
                    (0..=1, 0..=2) | (0..=2, 3..=4) | (9..=11, 2..=4)
                );
                // P2 bed collision when co-op active
                let p2_bed = self.coop_active && matches!((nc, nr), (9..=11, 5..=6));
                let furn =
                    (self.owned_furniture.contains(&FurnitureKind::Lamp) && nc == 4 && nr == 0) ||
                    (self.owned_furniture.contains(&FurnitureKind::FishTank) && matches!((nc, nr), (6..=7, 0))) ||
                    (self.owned_furniture.contains(&FurnitureKind::TV) && matches!((nc, nr), (9..=10, 0..=1))) ||
                    (self.owned_furniture.contains(&FurnitureKind::Couch) && matches!((nc, nr), (4..=6, 4..=5))) ||
                    (self.owned_furniture.contains(&FurnitureKind::PottedPlant) && nc == 11 && nr == 5);
                // Rug has no collision
                base || furn || p2_bed
            }
            BuildingKind::Inn => matches!(
                (nc, nr),
                (4..=7, 1..=2) | (10..=11, 0..=2) | (1..=2, 4..=5) | (7..=8, 4..=5)
            ),
            BuildingKind::Market => matches!(
                (nc, nr),
                (4..=7, 1..=2) | (0..=1, 0..=4) | (10..=11, 0..=4) | (8..=9, 4..=5)
            ),
            BuildingKind::Tavern => matches!(
                (nc, nr),
                (1..=8, 1) | (9..=11, 0..=2) | (1..=3, 4..=5) | (7..=9, 4..=5)
            ),
            BuildingKind::Clinic => matches!(
                (nc, nr),
                (0..=1, 0..=2) | (7..=9, 2..=3) | (3..=5, 1)
            ),
            BuildingKind::Library => matches!(
                (nc, nr),
                (0..=1, 0..=5) | (10..=11, 0..=5) | (5..=6, 0..=2) | (3..=4, 3..=4) | (7..=8, 3..=4)
            ),
            BuildingKind::TownHall => matches!(
                (nc, nr),
                (3..=8, 1..=2) | (0..=1, 0..=2) | (10..=11, 0..=1) | (2..=4, 4) | (7..=9, 4)
            ),
            BuildingKind::FurnitureShop => false,
            BuildingKind::AnimalShop => false,
            BuildingKind::Arcade => false,
            BuildingKind::Restaurant => false,
            BuildingKind::IceCreamShop => false,
        };
        if !blocked {
            self.farmhouse_tile = (nc, nr);
        }
    }

    /// Returns shop item names for the current season, alphabetically sorted.
    /// Non-crop items (e.g. "pendant") are always included regardless of season.
    pub fn shop_sorted_names(&self) -> Vec<String> {
        let current_season = &self.clock.season;
        let season_str = current_season.name().to_lowercase();
        let hl = self.player.hoe_level;
        let cl = self.player.can_level;
        let mut names: Vec<&String> = self.shop.items.keys()
            .filter(|name| {
                // Season-filter crops; always show non-crops
                match self.config.crops.get(*name) {
                    Some(crop) => crop.season.name().to_lowercase() == season_str,
                    None => true,
                }
            })
            .filter(|name| {
                // Show only the next available tool upgrade tier
                match name.as_str() {
                    "copper_hoe" => hl == 0,
                    "iron_hoe"   => hl == 1,
                    "gold_hoe"   => hl == 2,
                    "copper_can" => cl == 0,
                    "iron_can"   => cl == 1,
                    "gold_can"   => cl == 2,
                    "house_extension" => !self.house_upgraded,
                    _ => true,
                }
            })
            .collect();
        names.sort();
        names.into_iter().cloned().collect()
    }

    pub fn shop_move_cursor(&mut self, delta: i32) {
        let count = self.shop_sorted_names().len();
        if count == 0 { return; }
        self.shop_cursor = ((self.shop_cursor as i32 + delta).rem_euclid(count as i32)) as usize;
    }

    pub fn shop_try_buy(&mut self) -> Result<(), crate::game::shop::ShopError> {
        let names = self.shop_sorted_names();
        let name = names.get(self.shop_cursor)
            .ok_or(crate::game::shop::ShopError::ItemNotAvailable)?
            .clone();

        // Pendant is a special non-seed item.
        if name == "pendant" {
            let item = self.shop.items.get("pendant")
                .ok_or(crate::game::shop::ShopError::ItemNotAvailable)?;
            let price = self.effective_price(item.buy_price);
            if self.player.gold < price {
                return Err(crate::game::shop::ShopError::NotEnoughGold);
            }
            self.player.gold -= price;
            self.player.inventory.add(ItemKind::Pendant, 1);
            return Ok(());
        }

        // House extension.
        if name == "house_extension" {
            let price = self.effective_price(
                self.shop.items.get("house_extension")
                    .ok_or(crate::game::shop::ShopError::ItemNotAvailable)?
                    .buy_price
            );
            if self.player.gold < price {
                return Err(crate::game::shop::ShopError::NotEnoughGold);
            }
            self.player.gold -= price;
            self.house_upgraded = true;
            // Expand the farmhouse on the map (cols 1-5, rows 1-4)
            for row in 1..5 {
                for col in 1..6 {
                    self.map.tiles[row][col].kind = crate::game::world::TileKind::Farmhouse;
                }
            }
            self.notify("House extension built! Your home is bigger now.");
            return Ok(());
        }

        // Tool upgrades.
        let upgrade = match name.as_str() {
            "copper_hoe" => Some((true,  1u8)),
            "iron_hoe"   => Some((true,  2u8)),
            "gold_hoe"   => Some((true,  3u8)),
            "copper_can" => Some((false, 1u8)),
            "iron_can"   => Some((false, 2u8)),
            "gold_can"   => Some((false, 3u8)),
            _ => None,
        };
        if let Some((is_hoe, level)) = upgrade {
            let price = self.effective_price(
                self.shop.items.get(&name)
                    .ok_or(crate::game::shop::ShopError::ItemNotAvailable)?
                    .buy_price
            );
            if self.player.gold < price {
                return Err(crate::game::shop::ShopError::NotEnoughGold);
            }
            self.player.gold -= price;
            if is_hoe { self.player.hoe_level = level; } else { self.player.can_level = level; }
            let tool = if is_hoe { "Hoe" } else { "Watering Can" };
            let tier = ["", "Copper", "Iron", "Gold"][level as usize];
            self.notify(format!("{} {} upgrade purchased!", tier, tool));
            return Ok(());
        }

        let seed = shop_name_to_seed(&name)
            .ok_or(crate::game::shop::ShopError::ItemNotAvailable)?;
        let item = self.shop.items.get(&name).ok_or(crate::game::shop::ShopError::ItemNotAvailable)?;
        let price = self.effective_price(item.buy_price);
        if self.player.gold < price {
            return Err(crate::game::shop::ShopError::NotEnoughGold);
        }
        self.player.gold -= price;
        self.player.inventory.add(ItemKind::Seed(seed), 1);
        Ok(())
    }

    pub fn move_player(&mut self, dir: Direction) {
        if self.phase != GamePhase::Playing {
            return;
        }
        let (col, row) = self.player.tile;
        let map = &self.map;
        let new_tile = match dir {
            Direction::Up    => (col, row.saturating_sub(1)),
            Direction::Down  => (col, (row + 1).min(map.height - 1)),
            Direction::Left  => (col.saturating_sub(1), row),
            Direction::Right => ((col + 1).min(map.width - 1), row),
        };
        self.player.facing = dir;
        if let Some(tile) = self.map.get(new_tile.0, new_tile.1) {
            // Only the door tile of a building allows entry.
            if tile.kind == crate::game::world::TileKind::Farmhouse {
                if let Some(kind) = door_at(new_tile.0, new_tile.1) {
                    if kind == BuildingKind::FurnitureShop {
                        self.furniture_cursor = 0;
                        self.phase = GamePhase::FurnitureShopOpen;
                    } else if kind == BuildingKind::AnimalShop {
                        self.animal_cursor = 0;
                        self.phase = GamePhase::AnimalShopOpen;
                    } else if kind == BuildingKind::Arcade {
                        self.start_arcade();
                    } else if kind == BuildingKind::Restaurant {
                        self.open_restaurant();
                    } else if kind == BuildingKind::IceCreamShop {
                        self.open_icecream();
                    } else {
                        self.current_building = kind;
                        self.phase = GamePhase::FarmhouseInterior;
                        self.farmhouse_tile = (5, 6);
                        self.player.facing = crate::game::player::Direction::Up;
                    }
                    return;
                }
                // Non-door building tiles are walls — block movement
                return;
            }
            if !tile.kind.is_passable() {
                return;
            }

            // Check if an NPC is on the target tile
            let npc_idx = self.npcs.iter().position(|n| n.tile == new_tile);
            if let Some(idx) = npc_idx {
                // Try to nudge the NPC: perpendicular (step aside) first, then same direction as fallback
                let w = self.map.width;
                let h = self.map.height;
                let candidates = match dir {
                    Direction::Up | Direction::Down => vec![
                        // Step aside left/right first
                        (new_tile.0.saturating_sub(1), new_tile.1),
                        ((new_tile.0 + 1).min(w - 1), new_tile.1),
                        // Then push in player's direction
                        if dir == Direction::Up {
                            (new_tile.0, new_tile.1.saturating_sub(1))
                        } else {
                            (new_tile.0, (new_tile.1 + 1).min(h - 1))
                        },
                    ],
                    Direction::Left | Direction::Right => vec![
                        // Step aside up/down first
                        (new_tile.0, new_tile.1.saturating_sub(1)),
                        (new_tile.0, (new_tile.1 + 1).min(h - 1)),
                        // Then push in player's direction
                        if dir == Direction::Left {
                            (new_tile.0.saturating_sub(1), new_tile.1)
                        } else {
                            ((new_tile.0 + 1).min(w - 1), new_tile.1)
                        },
                    ],
                };
                let mut nudged = false;
                for nudge_tile in candidates {
                    if nudge_tile == new_tile || nudge_tile == self.player.tile { continue; }
                    let passable = self.map.get(nudge_tile.0, nudge_tile.1)
                        .map(|t| t.kind.is_passable())
                        .unwrap_or(false);
                    let free = !self.npcs.iter().any(|n| n.tile == nudge_tile);
                    if passable && free {
                        self.npcs[idx].tile = nudge_tile;
                        self.player.tile = new_tile;
                        nudged = true;
                        break;
                    }
                }
                // If no nudge direction worked, player is blocked
            } else {
                self.player.tile = new_tile;
                crate::game::save::play_sound("step");
            }
        }
    }

    /// Move player 2 (co-op). Same logic as move_player but for player2.
    pub fn move_player2(&mut self, dir: Direction) {
        if !self.coop_active || self.phase != GamePhase::Playing { return; }
        let (col, row) = self.player2.tile;
        let map = &self.map;
        let new_tile = match dir {
            Direction::Up    => (col, row.saturating_sub(1)),
            Direction::Down  => (col, (row + 1).min(map.height - 1)),
            Direction::Left  => (col.saturating_sub(1), row),
            Direction::Right => ((col + 1).min(map.width - 1), row),
        };
        self.player2.facing = dir;
        if let Some(tile) = self.map.get(new_tile.0, new_tile.1) {
            // P2 can enter buildings through doors
            if tile.kind == crate::game::world::TileKind::Farmhouse {
                if let Some(kind) = door_at(new_tile.0, new_tile.1) {
                    if kind == BuildingKind::FurnitureShop {
                        self.furniture_cursor = 0;
                        self.phase = GamePhase::FurnitureShopOpen;
                    } else if kind == BuildingKind::AnimalShop {
                        self.animal_cursor = 0;
                        self.phase = GamePhase::AnimalShopOpen;
                    } else {
                        self.current_building = kind;
                        self.phase = GamePhase::FarmhouseInterior;
                        self.farmhouse_tile = (5, 6);
                        self.player2.facing = crate::game::player::Direction::Up;
                    }
                    return;
                }
                return; // non-door wall
            }
            if !tile.kind.is_passable() { return; }
            // NPC nudge (same as P1)
            let w = self.map.width;
            let h = self.map.height;
            let npc_idx = self.npcs.iter().position(|n| n.tile == new_tile);
            if let Some(idx) = npc_idx {
                let candidates = match self.player2.facing {
                    Direction::Up | Direction::Down => vec![
                        (new_tile.0.saturating_sub(1), new_tile.1),
                        ((new_tile.0 + 1).min(w - 1), new_tile.1),
                    ],
                    Direction::Left | Direction::Right => vec![
                        (new_tile.0, new_tile.1.saturating_sub(1)),
                        (new_tile.0, (new_tile.1 + 1).min(h - 1)),
                    ],
                };
                for nudge_tile in candidates {
                    if nudge_tile == new_tile || nudge_tile == self.player2.tile || nudge_tile == self.player.tile { continue; }
                    let passable = self.map.get(nudge_tile.0, nudge_tile.1)
                        .map(|t| t.kind.is_passable()).unwrap_or(false);
                    let free = !self.npcs.iter().any(|n| n.tile == nudge_tile);
                    if passable && free {
                        self.npcs[idx].tile = nudge_tile;
                        self.player2.tile = new_tile;
                        break;
                    }
                }
            } else if new_tile != self.player.tile {
                self.player2.tile = new_tile;
                crate::game::save::play_sound("step");
            }
        }
    }
}

/// Return the string name of an ItemKind for gift preference lookups.
pub fn item_kind_to_name_pub(item: &ItemKind) -> &'static str {
    item_kind_to_name(item)
}

fn item_kind_to_name(item: &ItemKind) -> &'static str {
    use crate::game::inventory::{CropSeed, FishKind, ForageKind, HarvestedCrop, OreKind};
    match item {
        ItemKind::Seed(s) => match s {
            CropSeed::Parsnip => "parsnip", CropSeed::Potato => "potato",
            CropSeed::Cauliflower => "cauliflower", CropSeed::Melon => "melon",
            CropSeed::Blueberry => "blueberry", CropSeed::Tomato => "tomato",
            CropSeed::Pumpkin => "pumpkin", CropSeed::Yam => "yam",
            CropSeed::Cranberry => "cranberry",
        },
        ItemKind::Crop(c) => match c {
            HarvestedCrop::Parsnip => "parsnip", HarvestedCrop::Potato => "potato",
            HarvestedCrop::Cauliflower => "cauliflower", HarvestedCrop::Melon => "melon",
            HarvestedCrop::Blueberry => "blueberry", HarvestedCrop::Tomato => "tomato",
            HarvestedCrop::Pumpkin => "pumpkin", HarvestedCrop::Yam => "yam",
            HarvestedCrop::Cranberry => "cranberry",
        },
        ItemKind::Forage(f) => match f {
            ForageKind::Mushroom => "mushroom", ForageKind::Berry => "berry",
            ForageKind::Herb => "herb", ForageKind::Flower => "flower",
            ForageKind::Fern => "fern", ForageKind::Acorn => "acorn",
        },
        ItemKind::Fish(f) => match f {
            FishKind::Bass => "bass", FishKind::Catfish => "catfish",
            FishKind::Trout => "trout", FishKind::Salmon => "salmon",
        },
        ItemKind::Ore(o) => match o {
            OreKind::Copper => "copper", OreKind::Iron => "iron", OreKind::Gold => "gold",
        },
        ItemKind::Pendant => "pendant",
        ItemKind::Egg => "egg",
        ItemKind::Milk => "milk",
        ItemKind::Fiber => "fiber",
    }
}

/// Tiles to hoe based on facing direction and hoe level.
/// Level 0-1: 1 tile. Level 2: 3 tiles in a line. Level 3: 5 tiles in a line.
fn hoe_tiles(
    player: (usize, usize),
    facing: &Direction,
    level: u8,
) -> Vec<(usize, usize)> {
    let (pc, pr) = player;
    let (fc, fr) = match facing {
        Direction::Up    => (pc, pr.saturating_sub(1)),
        Direction::Down  => (pc, pr + 1),
        Direction::Left  => (pc.saturating_sub(1), pr),
        Direction::Right => (pc + 1, pr),
    };
    let count: i32 = match level { 2 => 1, 3 => 2, _ => 0 };
    let mut tiles = vec![(fc, fr)];
    // Extend perpendicular to facing direction
    let (perp_dc, perp_dr): (i32, i32) = match facing {
        Direction::Up | Direction::Down => (1, 0),
        Direction::Left | Direction::Right => (0, 1),
    };
    for i in 1..=count {
        let c1 = (fc as i32 + perp_dc * i) as usize;
        let r1 = (fr as i32 + perp_dr * i) as usize;
        tiles.push((c1, r1));
        if fc as i32 - perp_dc * i >= 0 && fr as i32 - perp_dr * i >= 0 {
            tiles.push(((fc as i32 - perp_dc * i) as usize, (fr as i32 - perp_dr * i) as usize));
        }
    }
    tiles
}

/// Tiles to water based on facing direction and can level.
/// Level 0: 1 tile. Level 1: 3-tile row. Level 2: 5-tile cross. Level 3: 3×3 area.
fn water_tiles(
    player: (usize, usize),
    facing: &Direction,
    level: u8,
) -> Vec<(usize, usize)> {
    let (pc, pr) = player;
    let (fc, fr) = match facing {
        Direction::Up    => (pc, pr.saturating_sub(1)),
        Direction::Down  => (pc, pr + 1),
        Direction::Left  => (pc.saturating_sub(1), pr),
        Direction::Right => (pc + 1, pr),
    };
    match level {
        0 => vec![(fc, fr)],
        1 => {
            // 3-tile row perpendicular to facing
            let (dc, dr): (i32, i32) = match facing {
                Direction::Up | Direction::Down => (1, 0),
                Direction::Left | Direction::Right => (0, 1),
            };
            let mut t = vec![(fc, fr)];
            for i in [1i32, -1] {
                let c = fc as i32 + dc * i;
                let r = fr as i32 + dr * i;
                if c >= 0 && r >= 0 { t.push((c as usize, r as usize)); }
            }
            t
        }
        2 => {
            // 5-tile cross (facing + 4 orthogonal)
            let mut t = vec![(fc, fr)];
            for (dc, dr) in [(0i32, -1i32), (0, 1), (-1, 0), (1, 0)] {
                let c = fc as i32 + dc;
                let r = fr as i32 + dr;
                if c >= 0 && r >= 0 { t.push((c as usize, r as usize)); }
            }
            t
        }
        _ => {
            // 3×3 area centered on facing tile
            let mut t = Vec::new();
            for dr in -1i32..=1 {
                for dc in -1i32..=1 {
                    let c = fc as i32 + dc;
                    let r = fr as i32 + dr;
                    if c >= 0 && r >= 0 { t.push((c as usize, r as usize)); }
                }
            }
            t
        }
    }
}

pub fn seed_to_crop_kind_pub(seed: &CropSeed) -> CropKind {
    seed_to_crop_kind(seed)
}

fn seed_to_crop_kind(seed: &CropSeed) -> CropKind {
    match seed {
        CropSeed::Parsnip    => CropKind::Parsnip,
        CropSeed::Potato     => CropKind::Potato,
        CropSeed::Cauliflower => CropKind::Cauliflower,
        CropSeed::Melon      => CropKind::Melon,
        CropSeed::Blueberry  => CropKind::Blueberry,
        CropSeed::Tomato     => CropKind::Tomato,
        CropSeed::Pumpkin    => CropKind::Pumpkin,
        CropSeed::Yam        => CropKind::Yam,
        CropSeed::Cranberry  => CropKind::Cranberry,
    }
}

/// Public alias used by shop_view.rs for rendering seed counts.
pub fn shop_name_to_seed_pub(name: &str) -> Option<CropSeed> {
    shop_name_to_seed(name)
}

fn shop_name_to_seed(name: &str) -> Option<CropSeed> {
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

fn crop_name_to_item(name: &str) -> Option<ItemKind> {
    match name {
        "parsnip"     => Some(ItemKind::Crop(HarvestedCrop::Parsnip)),
        "potato"      => Some(ItemKind::Crop(HarvestedCrop::Potato)),
        "cauliflower" => Some(ItemKind::Crop(HarvestedCrop::Cauliflower)),
        "melon"       => Some(ItemKind::Crop(HarvestedCrop::Melon)),
        "blueberry"   => Some(ItemKind::Crop(HarvestedCrop::Blueberry)),
        "tomato"      => Some(ItemKind::Crop(HarvestedCrop::Tomato)),
        "pumpkin"     => Some(ItemKind::Crop(HarvestedCrop::Pumpkin)),
        "yam"         => Some(ItemKind::Crop(HarvestedCrop::Yam)),
        "cranberry"   => Some(ItemKind::Crop(HarvestedCrop::Cranberry)),
        _ => None,
    }
}

fn forage_name_to_kind(name: &str) -> Option<crate::game::inventory::ForageKind> {
    use crate::game::inventory::ForageKind;
    match name {
        "mushroom" => Some(ForageKind::Mushroom),
        "berry"    => Some(ForageKind::Berry),
        "herb"     => Some(ForageKind::Herb),
        "flower"   => Some(ForageKind::Flower),
        "fern"     => Some(ForageKind::Fern),
        "acorn"    => Some(ForageKind::Acorn),
        _ => None,
    }
}

fn furniture_from_name(name: &str) -> Option<FurnitureKind> {
    match name {
        "TV"           => Some(FurnitureKind::TV),
        "Couch"        => Some(FurnitureKind::Couch),
        "Lamp"         => Some(FurnitureKind::Lamp),
        "Fish Tank"    => Some(FurnitureKind::FishTank),
        "Rug"          => Some(FurnitureKind::Rug),
        "Potted Plant" => Some(FurnitureKind::PottedPlant),
        _ => None,
    }
}

fn ore_name_to_kind(name: &str) -> Option<OreKind> {
    match name {
        "copper" => Some(OreKind::Copper),
        "iron"   => Some(OreKind::Iron),
        "gold"   => Some(OreKind::Gold),
        _ => None,
    }
}

fn fish_name_to_kind(name: &str) -> Option<FishKind> {
    match name {
        "bass"    => Some(FishKind::Bass),
        "catfish" => Some(FishKind::Catfish),
        "trout"   => Some(FishKind::Trout),
        "salmon"  => Some(FishKind::Salmon),
        _ => None,
    }
}

fn build_npcs(config: &GameConfig) -> Vec<NPC> {
    config
        .npcs
        .iter()
        .map(|cfg| {
            let schedule = cfg
                .schedule
                .iter()
                .map(|s| crate::game::npc::ScheduleEntry {
                    start_hour: s.start_hour,
                    end_hour: s.end_hour,
                    tile: s.tile,
                })
                .collect();
            NPC::new(
                cfg.id,
                cfg.name.clone(),
                cfg.personality.clone(),
                schedule,
                cfg.marriageable,
                cfg.gift_preferences.clone(),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::config::test_config;
    use crate::game::inventory::{CropSeed, HarvestedCrop, ItemKind};
    use crate::game::world::TileKind;

    fn test_state() -> GameState {
        GameState::new(test_config())
    }

    #[test]
    fn energy_restored_after_advance_day() {
        let mut state = test_state();
        state.player.energy = 10;
        state.advance_day();
        assert_eq!(state.player.energy, state.player.max_energy);
    }

    #[test]
    fn pending_gold_credited_on_advance_day() {
        let mut state = test_state();
        state.player.gold = 0;
        state.pending_gold = 100;
        state.advance_day();
        assert_eq!(state.player.gold, 100);
    }

    #[test]
    fn collision_player_blocked_by_water() {
        let mut state = test_state();
        // Row 0 is Water in default_farm
        state.player.tile = (5, 1);
        state.move_player(Direction::Up);
        // Should be blocked — water is not passable
        assert_eq!(state.player.tile, (5, 1));
        assert_eq!(state.player.facing, Direction::Up);
    }

    #[test]
    fn collision_player_blocked_by_farmhouse() {
        let mut state = test_state();
        // Farmhouse at rows 1–3, cols 1–3. Player at (0, 2) facing right should be blocked.
        state.player.tile = (0, 2);
        state.move_player(Direction::Right);
        assert_eq!(state.player.tile, (0, 2));
    }

    #[test]
    fn collision_player_can_walk_on_grass() {
        let mut state = test_state();
        state.player.tile = (5, 5);
        state.move_player(Direction::Down);
        assert_eq!(state.player.tile, (5, 6));
    }

    #[test]
    fn interact_facing_farmhouse_enters_interior() {
        let mut state = test_state();
        // Farmhouse occupies rows 1–3, cols 1–3.
        // Player at (2, 4) facing Up looks at (2, 3) which is Farmhouse.
        state.player.tile = (2, 4);
        state.player.facing = Direction::Up;
        state.try_interact();
        assert_eq!(state.phase, GamePhase::FarmhouseInterior);
    }

    #[test]
    fn walk_into_farmhouse_enters_interior() {
        let mut state = test_state();
        // Player at (2, 4) walking Up tries to enter (2, 3) which is Farmhouse.
        state.player.tile = (2, 4);
        state.move_player(Direction::Up);
        assert_eq!(state.phase, GamePhase::FarmhouseInterior);
    }

    #[test]
    fn sleep_from_farmhouse_interior_advances_day() {
        let mut state = test_state();
        state.phase = GamePhase::FarmhouseInterior;
        state.advance_day();
        assert_eq!(state.clock.day, 2);
    }

    #[test]
    fn notification_set_and_expires() {
        let mut state = test_state();
        state.notify("Test!");
        assert!(state.notification.is_some());
        // Tick past the TTL
        state.tick_notification(3.0);
        assert!(state.notification.is_none());
    }

    #[test]
    fn shop_cursor_wraps() {
        let mut state = test_state();
        let count = state.shop_sorted_names().len();
        assert!(count >= 2);
        // Moving -1 from 0 should wrap to last item
        state.shop_move_cursor(-1);
        assert_eq!(state.shop_cursor, count - 1);
        state.shop_move_cursor(1);
        assert_eq!(state.shop_cursor, 0);
    }

    #[test]
    fn shop_buy_via_cursor_deducts_gold() {
        use crate::game::inventory::{CropSeed, ItemKind};
        let mut state = test_state();
        let initial_gold = state.player.gold;
        // Find parsnip's cursor index (upgrades are now also in the shop)
        let names = state.shop_sorted_names();
        let parsnip_idx = names.iter().position(|n| n == "parsnip").expect("parsnip in shop");
        state.shop_cursor = parsnip_idx;
        state.shop_try_buy().unwrap();
        assert_eq!(state.player.gold, initial_gold - 20);
        assert_eq!(state.player.inventory.count(&ItemKind::Seed(CropSeed::Parsnip)), 1);
    }

    #[test]
    fn full_farming_loop_parsnip() {
        let mut state = test_state();
        let config = test_config(); // grow_days = 1 for parsnip in test config

        // Place player so facing tile (5,6) exists and is grass
        state.player.tile = (5, 5);
        state.player.facing = Direction::Down;
        state.map.tiles[6][5].kind = TileKind::Grass;

        // Hoe
        state.try_hoe().unwrap();
        assert_eq!(state.map.get(5, 6).unwrap().kind, TileKind::Tilled);

        // Plant (need a seed in inventory first)
        state.player.inventory.add(ItemKind::Seed(CropSeed::Parsnip), 1);
        state.player.selected_seed = Some(CropSeed::Parsnip);
        state.try_plant().unwrap();
        assert!(state.map.get(5, 6).unwrap().crop.is_some());

        // Water
        state.try_water().unwrap();
        assert!(state.map.get(5, 6).unwrap().crop.as_ref().unwrap().watered_today);

        // Advance day (parsnip grows in 1 day in test config)
        state.advance_day();
        // After advance_day, crop days_grown should be 1 = mature
        let crop = state.map.get(5, 6).unwrap().crop.as_ref().unwrap();
        assert_eq!(crop.days_grown, 1);
        assert!(crop.is_mature(config.crops["parsnip"].grow_days));

        // Harvest
        state.phase = GamePhase::Playing;
        state.try_harvest().unwrap();
        assert_eq!(
            state.player.inventory.count(&ItemKind::Crop(HarvestedCrop::Parsnip)),
            1
        );

        // Ship
        state.ship_all();
        assert!(state.pending_gold > 0);
        assert_eq!(state.player.inventory.count(&ItemKind::Crop(HarvestedCrop::Parsnip)), 0);
    }

    #[test]
    fn season_transitions_on_day_28() {
        let mut state = test_state();
        state.clock.day = 28;
        state.advance_day();
        assert_eq!(state.clock.season, crate::game::time::Season::Summer);
        assert_eq!(state.clock.day, 1);
    }

    #[test]
    fn season_end_clears_tilled_tiles_and_crops() {
        let mut state = test_state();
        // Till a tile and plant a parsnip
        state.map.tiles[6][5].kind = TileKind::Tilled;
        state.map.tiles[6][5].crop = Some(crate::game::crop::CropState::new(
            crate::game::crop::CropKind::Parsnip,
        ));
        // Advance through day 28 → season end
        state.clock.day = 28;
        state.advance_day();
        // Tilled tile should be Grass
        assert_eq!(state.map.get(5, 6).unwrap().kind, TileKind::Grass);
        // Crop should be gone
        assert!(state.map.get(5, 6).unwrap().crop.is_none());
    }

    #[test]
    fn shop_shows_only_current_season_crops() {
        let mut state = test_state();
        // test_config has only Spring crops (parsnip, potato)
        assert_eq!(state.clock.season, crate::game::time::Season::Spring);
        let names = state.shop_sorted_names();
        assert!(names.contains(&"parsnip".to_string()));
        assert!(names.contains(&"potato".to_string()));
        // No summer/fall crops in test_config
        assert!(!names.contains(&"melon".to_string()));
    }

    #[test]
    fn day_summary_notes_season_end() {
        let mut state = test_state();
        state.clock.day = 28;
        state.advance_day();
        let summary = state.day_summary.as_ref().unwrap();
        assert_eq!(summary.season_ended, Some("Spring".to_string()));
    }

    #[test]
    fn normal_day_summary_has_no_season_ended() {
        let mut state = test_state();
        state.advance_day();
        let summary = state.day_summary.as_ref().unwrap();
        assert!(summary.season_ended.is_none());
    }

    #[test]
    fn forage_patch_replenishes_each_morning() {
        let mut state = test_state();
        // tiles[row][col] — place a depleted patch at row=6, col=5
        state.map.tiles[6][5].kind = TileKind::ForagePatchEmpty;
        state.advance_day();
        // get(col, row) — so get(5, 6)
        assert_eq!(state.map.get(5, 6).unwrap().kind, TileKind::ForagePatch);
    }

    #[test]
    fn try_forage_on_standing_tile() {
        use crate::game::inventory::{ForageKind, ItemKind};
        let mut state = test_state();
        // Player starts at tile (10,10) → tiles[10][10]
        state.map.tiles[10][10].kind = TileKind::ForagePatch;
        state.try_forage().unwrap();
        // Inventory should contain one forage item
        let total: u32 = [
            ForageKind::Mushroom, ForageKind::Berry, ForageKind::Herb,
            ForageKind::Flower, ForageKind::Fern, ForageKind::Acorn,
        ]
        .iter()
        .map(|k| state.player.inventory.count(&ItemKind::Forage(k.clone())))
        .sum();
        assert_eq!(total, 1);
        // Tile should now be empty
        assert_eq!(state.map.get(10, 10).unwrap().kind, TileKind::ForagePatchEmpty);
    }

    #[test]
    fn try_mine_facing_rock_reduces_hp() {
        use crate::game::inventory::{ItemKind, OreKind};
        let mut state = test_state();
        // Place a Rock(1) at (5, 6) — player at (5,5) facing Down
        state.player.tile = (5, 5);
        state.player.facing = Direction::Down;
        state.map.tiles[6][5].kind = crate::game::world::TileKind::Rock(1);
        state.try_mine().unwrap();
        // Rock(1) → Grass, ore dropped
        assert_eq!(state.map.get(5, 6).unwrap().kind, crate::game::world::TileKind::Grass);
        let total: u32 = [OreKind::Copper, OreKind::Iron, OreKind::Gold]
            .iter()
            .map(|k| state.player.inventory.count(&ItemKind::Ore(k.clone())))
            .sum();
        assert_eq!(total, 1);
    }

    #[test]
    fn ship_all_includes_ore() {
        use crate::game::inventory::{ItemKind, OreKind};
        let mut state = test_state();
        state.player.inventory.add(ItemKind::Ore(OreKind::Copper), 3);
        state.ship_all();
        assert!(state.pending_gold > 0);
        assert_eq!(state.player.inventory.count(&ItemKind::Ore(OreKind::Copper)), 0);
    }

    #[test]
    fn try_fish_on_fishing_spot_starts_minigame() {
        let mut state = test_state();
        // Player starts at tile (10,10) → tiles[10][10]
        state.map.tiles[10][10].kind = crate::game::world::TileKind::FishingSpot;
        state.try_fish().unwrap();
        assert_eq!(state.phase, GamePhase::FishingMinigame);
        assert!(state.fish_target.is_some());
    }

    #[test]
    fn ship_all_includes_fish() {
        use crate::game::inventory::{FishKind, ItemKind};
        let mut state = test_state();
        state.player.inventory.add(ItemKind::Fish(FishKind::Bass), 2);
        state.ship_all();
        assert!(state.pending_gold > 0);
        assert_eq!(state.player.inventory.count(&ItemKind::Fish(FishKind::Bass)), 0);
    }

    #[test]
    fn ship_all_includes_forage_items() {
        use crate::game::inventory::{ForageKind, ItemKind};
        let mut state = test_state();
        state.player.inventory.add(ItemKind::Forage(ForageKind::Mushroom), 2);
        state.ship_all();
        assert!(state.pending_gold > 0);
        assert_eq!(state.player.inventory.count(&ItemKind::Forage(ForageKind::Mushroom)), 0);
    }
}
