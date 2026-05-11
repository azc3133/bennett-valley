use crate::game::crop::CropState;

#[derive(Debug, Clone, PartialEq)]
pub enum TileKind {
    Grass,
    Tilled,
    Watered,
    Path,
    Farmhouse,
    Shop,
    ShipBox,
    Water,
    /// Forageable item present — player can press G adjacent to collect.
    ForagePatch,
    /// Forage patch already collected today; replenishes each morning.
    ForagePatchEmpty,
    /// Fishing spot — player stands here and presses C to fish.
    FishingSpot,
    /// Rock with hit points remaining. When HP reaches 0 the tile becomes Grass.
    Rock(u8),
    /// Bench — passable. Player can sit on it (E) to restore energy.
    Bench,
    /// Oak tree — impassable. Player stands adjacent and presses G to collect acorns (Fall only).
    OakTree,
    /// Oak tree already harvested today; replenishes each morning during Fall.
    OakTreeEmpty,
    /// Long grass — passable, can be scythed with X to get fiber.
    LongGrass,
}

impl TileKind {
    pub fn is_passable(&self) -> bool {
        matches!(
            self,
            TileKind::Grass | TileKind::Tilled | TileKind::Watered | TileKind::Path
            | TileKind::ForagePatch | TileKind::ForagePatchEmpty | TileKind::FishingSpot
            | TileKind::Bench | TileKind::LongGrass
        )
        // Rock(_) is intentionally excluded — impassable
    }

    pub fn is_rock(&self) -> bool {
        matches!(self, TileKind::Rock(_))
    }

    pub fn is_tillable(&self) -> bool {
        matches!(self, TileKind::Grass)
    }

    pub fn is_plantable(&self) -> bool {
        matches!(self, TileKind::Tilled)
    }

    pub fn is_waterable(&self) -> bool {
        matches!(self, TileKind::Tilled | TileKind::Watered)
    }
}

#[derive(Debug, Clone)]
pub struct Tile {
    pub kind: TileKind,
    pub crop: Option<CropState>,
}

impl Tile {
    pub fn new(kind: TileKind) -> Self {
        Self { kind, crop: None }
    }
}

#[derive(Debug, Clone)]
pub struct FarmMap {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Vec<Tile>>, // tiles[row][col]
}

impl FarmMap {
    pub fn new(width: usize, height: usize) -> Self {
        let tiles = (0..height)
            .map(|_| (0..width).map(|_| Tile::new(TileKind::Grass)).collect())
            .collect();
        Self { width, height, tiles }
    }

    pub fn get(&self, col: usize, row: usize) -> Option<&Tile> {
        self.tiles.get(row)?.get(col)
    }

    pub fn get_mut(&mut self, col: usize, row: usize) -> Option<&mut Tile> {
        self.tiles.get_mut(row)?.get_mut(col)
    }

    /// Build the default farm layout.
    ///
    /// Map: 240 × 140 tiles
    ///
    /// Zones (original, cols 0-39, rows 0-29):
    ///   North shore  (row 0)                — Water border + fishing spots
    ///   Farm zone    (cols 5-35, rows 4-24)  — open grassland for crops
    ///   Forest zone  (cols 0-3, rows 4-24)   — forage patches
    ///   Mine zone    (cols 24-38, rows 4-19) — rock clusters
    ///   Pond         (cols 7-12, rows 22-25) — inland fishing hole
    ///   South wilds  (rows 21-28)            — forage + path to ship box
    ///
    /// Zones (new eastern half, cols 40-79):
    ///   Town district (cols 44-76, rows 1-19) — streets, inn, market, tavern
    ///   Town forage   (cols 40-43, rows 5-28) — tree line east of farm
    ///   Deep mine     (cols 48-75, rows 22-38)— large rock cavern
    ///   East river    (cols 77-79, rows 0-49) — vertical water feature
    ///
    /// Zones (new southern half, rows 30-49):
    ///   South lake    (cols 12-44, rows 33-43)— big fishing lake
    ///   South wilds   (rows 30-49)            — forage + paths
    ///   Bottom shore  (row 49)                — water border
    pub fn default_farm() -> Self {
        let mut map = FarmMap::new(240, 140);

        // ── Water borders ─────────────────────────────────────────────────
        for col in 0..240 {
            map.tiles[0][col].kind  = TileKind::Water; // top
            map.tiles[139][col].kind = TileKind::Water; // bottom
        }
        for row in 0..140 {
            map.tiles[row][0].kind  = TileKind::Water; // left
            map.tiles[row][239].kind = TileKind::Water; // right
        }

        // East river (cols 76-79, rows 0-139) — runs through the map
        for row in 0..140 {
            for col in 76..80 {
                map.tiles[row][col].kind = TileKind::Water;
            }
        }
        // East river fishing spots (left bank)
        for &row in &[5usize, 12, 19, 26, 34, 41, 50, 58, 65, 75, 85, 95, 105, 115, 125, 135] {
            if row < 140 { map.tiles[row][75].kind = TileKind::FishingSpot; }
        }
        // East river fishing spots (right bank)
        for &row in &[8usize, 16, 24, 32, 40, 48, 56, 63, 72, 80, 90, 100, 110, 120, 130] {
            if row < 140 { map.tiles[row][80].kind = TileKind::FishingSpot; }
        }

        // ── Inland pond (rows 24-27, cols 7-12) ─────────────────────────
        for row in 24..28 {
            for col in 7..13 {
                map.tiles[row][col].kind = TileKind::Water;
            }
        }

        // ── South lake (cols 12-44, rows 33-43) ─────────────────────────
        for row in 33..44 {
            for col in 12..45 {
                map.tiles[row][col].kind = TileKind::Water;
            }
        }
        // South lake fishing spots (perimeter)
        for &(col, row) in &[
            (11usize,33usize),(11,36),(11,39),(11,42),   // west bank
            (45,33),(45,36),(45,39),(45,42),              // east bank
            (16,32),(22,32),(28,32),(34,32),(40,32),      // north bank
            (16,44),(22,44),(28,44),(34,44),(40,44),      // south bank
        ] {
            if row < 140 && col < 240 {
                map.tiles[row][col].kind = TileKind::FishingSpot;
            }
        }

        // ── Town fountain (cols 59-61, rows 8-10) ────────────────────────
        for row in 8..11 {
            for col in 59..62 {
                map.tiles[row][col].kind = TileKind::Water;
            }
        }

        // ── Paths ─────────────────────────────────────────────────────────
        // South road (row 29, full width)
        for col in 1..239 {
            map.tiles[29][col].kind = TileKind::Path;
        }
        // East-west connector (row 20) — farm/wilderness boundary
        for col in 1..40 {
            map.tiles[20][col].kind = TileKind::Path;
        }
        // Vertical mine corridor (col 23, rows 4-20)
        for row in 4..21 {
            map.tiles[row][23].kind = TileKind::Path;
        }
        // South path to ship box (col 2, rows 20-28)
        for row in 20..29 {
            map.tiles[row][2].kind = TileKind::Path;
        }
        // East connector road (col 40, rows 1-28) — links farm to town
        for row in 1..29 {
            map.tiles[row][40].kind = TileKind::Path;
        }
        // Town main street (row 6, cols 41-75)
        for col in 41..76 {
            map.tiles[6][col].kind = TileKind::Path;
        }
        // Town cross streets (vertical, cols 48 56 64 72)
        for &col in &[48usize, 56, 64, 72] {
            for row in 1..20 {
                map.tiles[row][col].kind = TileKind::Path;
            }
        }
        // Town north street (row 1, cols 41-75)
        for col in 41..76 {
            map.tiles[1][col].kind = TileKind::Path;
        }
        // Town mid street (row 13, cols 41-75)
        for col in 41..76 {
            map.tiles[13][col].kind = TileKind::Path;
        }
        // Town south exit (row 20, cols 40-75)
        for col in 40..76 {
            map.tiles[20][col].kind = TileKind::Path;
        }

        // ── East district paths (east of river) ──────────────────────────
        // Bridge across river (row 6, cols 76-79) — path over water
        for col in 76..80 {
            map.tiles[6][col].kind = TileKind::Path;
        }
        // Bridge across river (row 20, cols 76-79)
        for col in 76..80 {
            map.tiles[20][col].kind = TileKind::Path;
        }
        // Bridge across river (row 29, already covered by south road)

        // East district streets
        for col in 80..238 {
            map.tiles[6][col].kind  = TileKind::Path; // north street
            map.tiles[13][col].kind = TileKind::Path; // mid street
            map.tiles[20][col].kind = TileKind::Path; // south street
        }
        // East cross streets
        for &col in &[88usize, 96, 104, 112, 120, 128, 136, 144, 152, 160, 168, 176, 184, 192, 200, 208, 216, 224, 232] {
            if col < 240 {
                for row in 1..28 {
                    map.tiles[row][col].kind = TileKind::Path;
                }
            }
        }

        // ── South expansion paths ────────────────────────────────────────
        // South connector road (col 40, rows 29-138)
        for row in 29..139 {
            map.tiles[row][40].kind = TileKind::Path;
        }
        // Far south road (row 50, cols 1-238)
        for col in 1..239 {
            map.tiles[50][col].kind = TileKind::Path;
        }
        // Additional south roads
        for col in 1..239 {
            map.tiles[70][col].kind = TileKind::Path;
            map.tiles[100][col].kind = TileKind::Path;
            map.tiles[130][col].kind = TileKind::Path;
        }
        // Vertical connector roads in south expansion
        for &col in &[80usize, 120, 160, 200] {
            for row in 29..139 {
                if col < 240 { map.tiles[row][col].kind = TileKind::Path; }
            }
        }

        // ── Structures ────────────────────────────────────────────────────
        // Farmhouse (rows 1-3, cols 1-3)
        for row in 1..4 {
            for col in 1..4 {
                map.tiles[row][col].kind = TileKind::Farmhouse;
            }
        }
        // Shop (rows 1-3, cols 36-39)
        for row in 1..4 {
            for col in 36..40 {
                map.tiles[row][col].kind = TileKind::Shop;
            }
        }
        // Shipping box (row 28, col 1) — overwrites path above
        map.tiles[28][1].kind = TileKind::ShipBox;

        // Town Inn (rows 2-5, cols 41-47) — Farmhouse tiles
        for row in 2..6 {
            for col in 41..48 {
                map.tiles[row][col].kind = TileKind::Farmhouse;
            }
        }
        // Town Market (rows 2-5, cols 49-55) — Farmhouse tiles (interior)
        for row in 2..6 {
            for col in 49..56 {
                map.tiles[row][col].kind = TileKind::Farmhouse;
            }
        }
        // Town Tavern (rows 2-5, cols 57-63) — Farmhouse tiles
        for row in 2..6 {
            for col in 57..64 {
                map.tiles[row][col].kind = TileKind::Farmhouse;
            }
        }
        // Town Clinic (rows 2-5, cols 65-71) — Farmhouse tiles (interior)
        for row in 2..6 {
            for col in 65..72 {
                map.tiles[row][col].kind = TileKind::Farmhouse;
            }
        }
        // South block: Town Library (rows 7-12, cols 41-47) — Farmhouse
        for row in 7..13 {
            for col in 41..48 {
                map.tiles[row][col].kind = TileKind::Farmhouse;
            }
        }
        // South block: Town Hall (rows 7-12, cols 65-75) — Farmhouse
        for row in 7..13 {
            for col in 65..76 {
                map.tiles[row][col].kind = TileKind::Farmhouse;
            }
        }
        // Clear fountain area (it was set to Farmhouse above)
        for row in 8..11 {
            for col in 59..62 {
                map.tiles[row][col].kind = TileKind::Water;
            }
        }

        // Furniture Shop (rows 7-12, cols 49-55) — Farmhouse tiles
        for row in 7..13 {
            for col in 49..56 {
                map.tiles[row][col].kind = TileKind::Farmhouse;
            }
        }

        // ── NPC houses (8 small cottages, 3×2 each) ─────────────────────
        let npc_houses: &[(usize, usize)] = &[
            (42, 15), (45, 15),   // south of Library
            (42, 18), (45, 18),   // south residential
            (55, 15), (55, 18),   // central residential
            (71, 15), (71, 18),   // east residential
        ];
        for &(col, row) in npc_houses {
            for r in row..row+2 {
                for c in col..col+3 {
                    if r < 140 && c < 240 {
                        map.tiles[r][c].kind = TileKind::Farmhouse;
                    }
                }
            }
        }

        // Restaurant (rows 21-23, cols 41-47) — Farmhouse tiles (above lake)
        for row in 21..24 {
            for col in 41..48 {
                map.tiles[row][col].kind = TileKind::Farmhouse;
            }
        }

        // Arcade (rows 21-25, cols 49-55) — Farmhouse tiles
        for row in 21..26 {
            for col in 49..56 {
                map.tiles[row][col].kind = TileKind::Farmhouse;
            }
        }

        // Swimming Pool (rows 21-25, cols 57-63)
        // Deck border (Farmhouse = impassable)
        for col in 57..64 {
            map.tiles[21][col].kind = TileKind::Farmhouse; // north deck
            map.tiles[25][col].kind = TileKind::Farmhouse; // south deck
        }
        for row in 21..26 {
            map.tiles[row][57].kind = TileKind::Farmhouse; // west deck
            map.tiles[row][63].kind = TileKind::Farmhouse; // east deck
        }
        // Pool water (interior)
        for row in 22..25 {
            for col in 58..63 {
                map.tiles[row][col].kind = TileKind::Water;
            }
        }

        // Ice Cream Shop (rows 21-23, cols 65-69) — Farmhouse tiles
        for row in 21..24 {
            for col in 65..70 {
                map.tiles[row][col].kind = TileKind::Farmhouse;
            }
        }

        // Animal Shop (rows 14-19, cols 63-69) — Farmhouse tiles
        for row in 14..20 {
            for col in 63..70 {
                map.tiles[row][col].kind = TileKind::Farmhouse;
            }
        }

        // ── Animal pen fences (drawn visually in farm_view, tiles stay grass) ──
        // Pens are cosmetic — the fence is drawn over grass tiles.
        // Only the coop/barn wall tiles are Farmhouse (impassable).
        // Chicken coop roof: row 14, cols 5-7
        for col in 5..8 { map.tiles[14][col].kind = TileKind::Farmhouse; }
        // Cow barn walls: row 18, cols 5-8
        for col in 5..9 { map.tiles[18][col].kind = TileKind::Farmhouse; }
        // Horse stable walls: row 18, cols 10-14
        for col in 10..15 { map.tiles[18][col].kind = TileKind::Farmhouse; }

        // ── Benches ────────────────────────────────────────────────────────
        let bench_coords: &[(usize, usize)] = &[
            // Near farmhouse
            (5, 4),
            // Town main street
            (46, 6), (54, 6), (62, 6),
            // Town mid street
            (46, 13), (54, 13), (62, 13),
            // Near pond
            (14, 23),
            // Near south lake
            (10, 31), (46, 32),
            // Town park (near fountain)
            (58, 13),
            // Near playground
            (49, 19),
            // East side
            (74, 20),
        ];
        for &(col, row) in bench_coords {
            if row < 140 && col < 240 {
                map.tiles[row][col].kind = TileKind::Bench;
            }
        }

        // ── Playground (cols 50-61, rows 14-18) ──────────────────────────
        // Equipment tiles are impassable (Farmhouse); gaps between are grass (passable)
        // Swing set: cols 51-53, rows 14-16
        for row in 14..17 { for col in 51..54 { map.tiles[row][col].kind = TileKind::Farmhouse; } }
        // Slide: cols 55-58, rows 14-17
        for row in 14..17 { for col in 55..59 { map.tiles[row][col].kind = TileKind::Farmhouse; } }
        // Sandbox: cols 59-61, rows 16-18
        for row in 16..18 { for col in 59..62 { map.tiles[row][col].kind = TileKind::Farmhouse; } }
        // Seesaw: cols 51-53, rows 17-18
        for row in 17..19 { for col in 51..54 { map.tiles[row][col].kind = TileKind::Farmhouse; } }

        // ── Fishing spots ─────────────────────────────────────────────────
        // North shore (original + new)
        for &(col, row) in &[
            (8usize,1usize),(14,1),(20,1),(26,1),(33,1),
            (50,1),(60,1),(70,1),
        ] {
            map.tiles[row][col].kind = TileKind::FishingSpot;
        }
        // Pond perimeter
        for &(col, row) in &[
            (6usize,24usize),(6,25),(6,26),(6,27),
            (13,24),(13,25),(13,26),(13,27),
            (9,23),(10,23),(11,23),
        ] {
            map.tiles[row][col].kind = TileKind::FishingSpot;
        }

        // ── Forage patches ────────────────────────────────────────────────
        let forage_coords: &[(usize, usize)] = &[
            // Left forest strip (cols 0-3, rows 4-24)
            (0,4),(1,5),(2,6),(0,7),
            (1,8),(2,9),(0,10),(1,11),
            (0,12),(2,13),(1,14),(0,16),
            (2,15),(1,17),(0,18),(2,19),
            (0,21),(1,23),(2,24),(0,25),
            // South wilderness
            (3,22),(5,23),(4,26),(3,27),(6,27),
            (5,31),(8,32),(4,30),(7,30),(9,30),
            (3,35),(6,36),(8,37),(5,40),(4,44),
            (7,45),(10,46),(3,47),(8,48),
            // East of farm (cols 39-43, rows 5-28)
            (39,5),(41,7),(43,9),(39,11),(42,13),
            (39,16),(41,18),(43,21),(39,24),(41,26),
            (43,27),(39,28),
            // East wilderness (cols 44-50, rows 22-28)
            (44,22),(46,24),(48,26),(45,27),(47,28),
            // Far south (scattered)
            (10,30),(12,31),(50,31),(55,30),(60,31),
            (52,44),(57,46),(62,44),(67,45),(72,46),
            // Town park forage (near fountain)
            (62,11),(63,11),(62,12),
        ];
        for &(col, row) in forage_coords {
            if row < 140 && col < 240 {
                map.tiles[row][col].kind = TileKind::ForagePatch;
            }
        }

        // ── Oak trees ──────────────────────────────────────────────────────
        let oak_coords: &[(usize, usize)] = &[
            // West forest edge
            (3, 5), (1, 9), (3, 13), (1, 16),
            // Near farm (scattered)
            (8, 4), (16, 4), (20, 4),
            // Between farm and mine
            (22, 12), (22, 18),
            // East of farm road
            (42, 8), (42, 14), (42, 20),
            // South of farm
            (10, 28), (18, 26), (25, 28),
            // South wilderness
            (6, 32), (2, 36), (9, 40), (4, 42), (7, 47),
            // Near south lake (not in water)
            (9, 32), (46, 34), (46, 40),
            // Town park area
            (50, 14), (54, 14), (58, 14),
            // East side
            (74, 4), (74, 10), (74, 16),
        ];
        for &(col, row) in oak_coords {
            if row < 140 && col < 240 {
                map.tiles[row][col].kind = TileKind::OakTree;
            }
        }

        // ── Farm rocks (scattered across cols 4-22, rows 5-19) ───────────
        let farm_rocks: &[(usize, usize)] = &[
            (5, 5), (8, 6), (12, 5), (18, 6),
            (6, 9), (10, 8), (14, 9), (19, 8),
            (4, 12), (9, 11), (15, 12), (21, 11),
            (7, 14), (11, 15), (17, 14), (22, 15),
            (5, 17), (13, 18), (20, 17),
            // South farm area (rows 21-27)
            (5, 22), (10, 21), (16, 22), (21, 21),
            (7, 25), (12, 26), (18, 25),
            (4, 27), (14, 27), (20, 27),
        ];
        for &(col, row) in farm_rocks {
            if map.tiles[row][col].kind == TileKind::Grass
                || map.tiles[row][col].kind == TileKind::LongGrass
            {
                map.tiles[row][col].kind = TileKind::Rock(3);
            }
        }

        // ── Mine zone — original (cols 24-38, rows 4-19) ─────────────────
        let rock_coords_orig: &[(usize, usize)] = &[
            (24,4),(25,4),(26,4),(27,4),(28,4),
            (24,5),(26,5),(28,5),
            (25,6),(27,6),
            (24,8),(25,8),(26,8),(27,8),(28,8),(29,8),
            (24,9),(26,9),(28,9),
            (25,10),(27,10),(29,10),
            (24,11),(26,11),(28,11),
            (25,12),(27,12),
            (26,14),(27,14),(28,14),(29,14),(30,14),(31,14),
            (26,15),(28,15),(30,15),
            (27,16),(29,16),(31,16),
            (26,17),(28,17),(30,17),
            (27,18),(29,18),
            // Extended east (cols 32-38)
            (32,5),(34,5),(36,5),(38,6),
            (33,7),(35,7),(37,7),
            (32,9),(34,9),(36,9),(38,9),
            (33,11),(35,11),(37,11),
            (32,13),(34,13),(36,13),(38,13),
            (33,15),(35,15),(37,15),
            (32,17),(34,17),(36,17),(38,17),
        ];
        for &(col, row) in rock_coords_orig {
            map.tiles[row][col].kind = TileKind::Rock(3);
        }

        // ── Deep mine (cols 48-74, rows 22-38) ────────────────────────────
        // Access corridor (col 48, rows 20-22)
        for row in 20..23 {
            map.tiles[row][48].kind = TileKind::Path;
        }
        let rock_coords_deep: &[(usize, usize)] = &[
            // Sparse south cluster (away from town)
            (50,32),(54,32),(58,32),
            (51,33),(55,33),(59,33),
            (50,34),(54,34),(58,34),
            (53,35),(57,35),
            (52,36),(56,36),
            (55,37),
            // Far east deep
            (64,30),(68,30),(72,30),
            (65,31),(69,31),(73,31),
            (66,32),(70,32),
            (67,33),(71,33),
            (68,34),(70,34),
        ];
        for &(col, row) in rock_coords_deep {
            if row < 140 && col < 76 {
                map.tiles[row][col].kind = TileKind::Rock(3);
            }
        }

        // ── Long grass (scattered across farm and wilderness) ────────────
        for row in 4..28 {
            for col in 4..38 {
                if map.tiles[row][col].kind == TileKind::Grass {
                    let h = col.wrapping_mul(11).wrapping_add(row.wrapping_mul(23));
                    if h % 6 == 0 {
                        map.tiles[row][col].kind = TileKind::LongGrass;
                    }
                }
            }
        }
        // South wilderness long grass
        for row in 30..50 {
            for col in 1..75 {
                if map.tiles[row][col].kind == TileKind::Grass {
                    let h = col.wrapping_mul(13).wrapping_add(row.wrapping_mul(19));
                    if h % 5 == 0 {
                        map.tiles[row][col].kind = TileKind::LongGrass;
                    }
                }
            }
        }

        // ── East district content (east of river, cols 80-118) ──────────

        // East meadow pond (cols 100-106, rows 15-19)
        for row in 15..20 {
            for col in 100..107 {
                map.tiles[row][col].kind = TileKind::Water;
            }
        }
        // East pond fishing spots
        for &(col, row) in &[
            (99usize,15usize),(99,17),(99,19),
            (107,15),(107,17),(107,19),
            (102,14),(104,14),
        ] {
            if row < 140 && col < 240 {
                map.tiles[row][col].kind = TileKind::FishingSpot;
            }
        }

        // East benches
        for &(col, row) in &[(85usize,6usize),(93,6),(101,6),(109,6),(85,13),(93,13),(109,13),(98,20)] {
            if row < 140 && col < 240 {
                map.tiles[row][col].kind = TileKind::Bench;
            }
        }

        // East oak trees
        for &(col, row) in &[
            (83usize,3usize),(90,3),(97,3),(105,3),(113,3),
            (82,10),(89,10),(95,10),(110,10),(115,10),
            (83,17),(90,17),(113,17),
            (82,24),(89,24),(95,24),(102,24),(110,24),(116,24),
            // South of east river
            (82,32),(88,34),(94,32),(100,34),(106,32),(112,34),
            (85,40),(92,42),(98,40),(104,42),(110,40),(116,42),
        ] {
            if row < 140 && col < 240 {
                map.tiles[row][col].kind = TileKind::OakTree;
            }
        }

        // East forage patches
        for &(col, row) in &[
            (81usize,4usize),(84,5),(87,4),(91,5),(94,4),(98,5),
            (81,8),(86,9),(92,8),(96,9),(108,8),(114,9),
            (84,15),(88,16),(92,15),(112,16),(116,15),
            (83,22),(87,23),(91,22),(97,23),(108,22),(114,23),
            // South expansion forage
            (81,31),(85,33),(89,31),(93,33),(99,31),(105,33),(111,31),(115,33),
            (83,38),(90,39),(96,38),(103,39),(109,38),(115,39),
            (85,45),(91,46),(97,45),(103,46),(109,45),(115,46),
        ] {
            if row < 140 && col < 240 {
                map.tiles[row][col].kind = TileKind::ForagePatch;
            }
        }

        // ── South expansion content (rows 50-68) ────────────────────────

        // Southern forest (dense trees along bottom)
        for &(col, row) in &[
            (3usize,52usize),(8,54),(14,52),(20,54),(26,52),(32,54),(38,52),
            (5,58),(11,56),(17,58),(23,56),(29,58),(35,56),
            (4,62),(10,64),(16,62),(22,64),(28,62),(34,64),
            (7,67),(13,66),(19,67),(25,66),(31,67),(37,66),
            // East of south connector
            (44,52),(50,54),(56,52),(62,54),(68,52),(74,54),
            (42,58),(48,56),(54,58),(60,56),(66,58),(72,56),
            (44,62),(50,64),(56,62),(62,64),(68,62),(74,64),
            (46,67),(52,66),(58,67),(64,66),(70,67),
        ] {
            if row < 140 && col < 240 {
                map.tiles[row][col].kind = TileKind::OakTree;
            }
        }

        // South forage
        for &(col, row) in &[
            (2usize,51usize),(6,53),(12,51),(18,53),(24,51),(30,53),(36,51),
            (4,55),(10,57),(16,55),(22,57),(28,55),(34,57),
            (44,51),(50,53),(56,51),(62,53),(68,51),(74,53),
            (46,55),(52,57),(58,55),(64,57),(70,55),
        ] {
            if row < 140 && col < 240 {
                map.tiles[row][col].kind = TileKind::ForagePatch;
            }
        }

        // South lake extension (cols 12-44, rows 44-48) — expands existing south lake
        for row in 44..49 {
            for col in 12..45 {
                map.tiles[row][col].kind = TileKind::Water;
            }
        }
        // South lake fishing spots (new south bank)
        for &(col, row) in &[
            (16usize,49usize),(22,49),(28,49),(34,49),(40,49),
        ] {
            if row < 140 && col < 240 {
                map.tiles[row][col].kind = TileKind::FishingSpot;
            }
        }

        // South benches
        for &(col, row) in &[(10usize,50usize),(46,50),(20,50),(30,50)] {
            if row < 140 && col < 240 {
                map.tiles[row][col].kind = TileKind::Bench;
            }
        }

        // ── Beach (cols 82-220, rows 120-138) — wide south coastline ────
        // Sand (Path tiles) — big sandy area
        for row in 124..133 {
            for col in 82..221 {
                map.tiles[row][col].kind = TileKind::Path;
            }
        }
        // Curved shoreline — wider sand at edges
        for col in 84..219 {
            let mid = 151.0;
            let curve = ((col as f32 - mid) / 70.0).powi(2);
            let sand_start = 121 + (curve * 4.0) as usize;
            for row in sand_start..124 {
                if row < 140 {
                    map.tiles[row][col].kind = TileKind::Path;
                }
            }
        }
        // Ocean water (rows 133-138)
        for row in 133..139 {
            for col in 80..222 {
                map.tiles[row][col].kind = TileKind::Water;
            }
        }
        // Shallow water (row 132)
        for col in 82..221 {
            map.tiles[132][col].kind = TileKind::Water;
        }
        // Beach fishing spots (along waterline)
        for &col in &[86usize, 96, 106, 116, 126, 136, 146, 156, 166, 176, 186, 196, 206, 216] {
            if col < 240 { map.tiles[131][col].kind = TileKind::FishingSpot; }
        }
        // Beach benches
        for &col in &[88usize, 100, 112, 124, 140, 156, 172, 188, 204] {
            if col < 240 { map.tiles[124][col].kind = TileKind::Bench; }
        }
        // Path to beach from south road (col 120, rows 100-123)
        for row in 101..124 {
            map.tiles[row][120].kind = TileKind::Path;
        }
        // Second path (col 160)
        for row in 101..124 {
            map.tiles[row][160].kind = TileKind::Path;
        }
        // Clear any trees/forage that landed on the beach
        for row in 120..139 {
            for col in 82..221 {
                if matches!(map.tiles[row][col].kind,
                    TileKind::OakTree | TileKind::OakTreeEmpty |
                    TileKind::ForagePatch | TileKind::ForagePatchEmpty |
                    TileKind::LongGrass | TileKind::Rock(_)
                ) {
                    map.tiles[row][col].kind = TileKind::Path;
                }
            }
        }

        // ── Wizard's Hut (cols 230-234, rows 133-136) ────────────────
        for row in 133..137 {
            for col in 230..235 {
                map.tiles[row][col].kind = TileKind::Farmhouse;
            }
        }
        // Path to wizard from south road (col 232, rows 130-132)
        for row in 130..133 {
            map.tiles[row][232].kind = TileKind::Path;
        }

        // East rocks (small mine extension)
        for &(col, row) in &[
            (83usize,27usize),(85,27),(87,27),(89,27),
            (84,28),(86,28),(88,28),
        ] {
            if row < 140 && col < 240 {
                map.tiles[row][col].kind = TileKind::Rock(3);
            }
        }

        // ── Dense tree coverage across entire map ─────────────────────
        // Uses a hash to place trees procedurally — denser than before
        for row in 2..138 {
            for col in 2..238 {
                if map.tiles[row][col].kind != TileKind::Grass
                    && map.tiles[row][col].kind != TileKind::LongGrass {
                    continue;
                }
                let h = col.wrapping_mul(31).wrapping_add(row.wrapping_mul(17))
                    .wrapping_add(col.wrapping_mul(row).wrapping_mul(7));
                // Different density for different zones
                let threshold = if col < 4 || row < 4 {
                    5  // very dense forest edge
                } else if col < 40 && row < 30 {
                    12 // farm area — moderate (leave room for farming)
                } else if col >= 80 && col < 120 && row < 30 {
                    8  // east district — moderate-dense
                } else if row >= 30 && row < 70 {
                    7  // south wilderness — dense
                } else {
                    5  // deep south + far east — very dense forest
                };
                if h % threshold == 0 {
                    map.tiles[row][col].kind = TileKind::OakTree;
                }
            }
        }

        // ── Forage patches across expanded areas ────────────────────
        for row in 2..138 {
            for col in 2..238 {
                if map.tiles[row][col].kind != TileKind::Grass
                    && map.tiles[row][col].kind != TileKind::LongGrass {
                    continue;
                }
                let h = col.wrapping_mul(43).wrapping_add(row.wrapping_mul(29));
                if h % 18 == 0 {
                    map.tiles[row][col].kind = TileKind::ForagePatch;
                }
            }
        }

        // Deep south benches
        for &(col, row) in &[
            (10usize,70usize),(30,70),(50,70),(70,70),(90,70),(110,70),
            (10,100),(30,100),(50,100),(70,100),(90,100),(110,100),
            (10,130),(30,130),(50,130),(70,130),(90,130),(110,130),
        ] {
            if row < 140 && col < 240 { map.tiles[row][col].kind = TileKind::Bench; }
        }

        // Deep south lakes
        // Lake at cols 20-35, rows 80-88
        for row in 80..89 {
            for col in 20..36 {
                if row < 140 && col < 240 { map.tiles[row][col].kind = TileKind::Water; }
            }
        }
        for &(col, row) in &[(19usize,82usize),(19,85),(36,82),(36,85),(25,79),(30,79),(25,89),(30,89)] {
            if row < 140 && col < 240 { map.tiles[row][col].kind = TileKind::FishingSpot; }
        }
        // Lake at cols 50-65, rows 110-118
        for row in 110..119 {
            for col in 50..66 {
                if row < 140 && col < 240 { map.tiles[row][col].kind = TileKind::Water; }
            }
        }
        for &(col, row) in &[(49usize,112usize),(49,115),(66,112),(66,115),(55,109),(60,109),(55,119),(60,119)] {
            if row < 140 && col < 240 { map.tiles[row][col].kind = TileKind::FishingSpot; }
        }

        // Scatter rocks in the deep south
        for row_base in &[75usize, 88, 95, 105, 118, 125, 135] {
            for col_step in 0..15 {
                let col = 8 + col_step * 10;
                let row = row_base + (col_step % 2);
                if row < 139 && col < 76 && map.tiles[row][col].kind == TileKind::Grass {
                    map.tiles[row][col].kind = TileKind::Rock(3);
                }
            }
        }

        // Long grass in the new areas
        for row in 70..138 {
            for col in 1..75 {
                if map.tiles[row][col].kind == TileKind::Grass {
                    let h = col.wrapping_mul(11).wrapping_add(row.wrapping_mul(23));
                    if h % 7 == 0 { map.tiles[row][col].kind = TileKind::LongGrass; }
                }
            }
        }
        for row in 1..138 {
            for col in 120..238 {
                if map.tiles[row][col].kind == TileKind::Grass {
                    let h = col.wrapping_mul(13).wrapping_add(row.wrapping_mul(19));
                    if h % 8 == 0 { map.tiles[row][col].kind = TileKind::LongGrass; }
                }
            }
        }

        map
    }
}

#[cfg(test)]
pub fn test_farm_map() -> FarmMap {
    let mut map = FarmMap::new(10, 10);
    // Pre-till a few tiles for testing
    map.tiles[5][5].kind = TileKind::Tilled;
    map.tiles[5][6].kind = TileKind::Tilled;
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_map_is_all_grass() {
        let map = FarmMap::new(5, 5);
        for row in &map.tiles {
            for tile in row {
                assert_eq!(tile.kind, TileKind::Grass);
            }
        }
    }

    #[test]
    fn get_tile_by_col_row() {
        let mut map = FarmMap::new(10, 10);
        map.tiles[3][2].kind = TileKind::Path;
        assert_eq!(map.get(2, 3).unwrap().kind, TileKind::Path);
    }

    #[test]
    fn get_out_of_bounds_returns_none() {
        let map = FarmMap::new(5, 5);
        assert!(map.get(10, 10).is_none());
    }

    #[test]
    fn grass_is_tillable() {
        assert!(TileKind::Grass.is_tillable());
        assert!(!TileKind::Tilled.is_tillable());
    }

    #[test]
    fn tilled_is_plantable() {
        assert!(TileKind::Tilled.is_plantable());
        assert!(!TileKind::Grass.is_plantable());
    }
}
