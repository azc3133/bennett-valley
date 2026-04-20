use macroquad::prelude::*;
use crate::game::npc::NPC;
use crate::render::camera::{Camera, TILE_SIZE};
use crate::render::player_view::draw_character;
use crate::game::player::Direction;

// Each NPC gets a unique outfit color (indexed by NPC position in the vec).
const NPC_OUTFIT: [Color; 40] = [
    Color { r: 1.0,  g: 0.5,  b: 0.0,  a: 1.0 }, // orange   (Elara)
    Color { r: 0.8,  g: 0.2,  b: 0.8,  a: 1.0 }, // purple   (Tom)
    Color { r: 0.2,  g: 0.8,  b: 0.8,  a: 1.0 }, // cyan     (Maya)
    Color { r: 0.9,  g: 0.9,  b: 0.2,  a: 1.0 }, // yellow   (Rex)
    Color { r: 1.0,  g: 0.4,  b: 0.4,  a: 1.0 }, // pink     (Suki)
    Color { r: 0.2,  g: 0.5,  b: 1.0,  a: 1.0 }, // blue     (Finn)
    Color { r: 0.7,  g: 0.4,  b: 0.1,  a: 1.0 }, // brown    (Petra)
    Color { r: 0.3,  g: 0.8,  b: 0.3,  a: 1.0 }, // green    (Lily)
    Color { r: 0.9,  g: 0.6,  b: 0.1,  a: 1.0 }, // gold     (Otto)
    Color { r: 0.85, g: 0.85, b: 0.85, a: 1.0 }, // white    (Bea)
    Color { r: 0.5,  g: 0.3,  b: 0.7,  a: 1.0 }, // violet   (Cal)
    Color { r: 0.4,  g: 0.6,  b: 0.2,  a: 1.0 }, // olive    (Rue)
    Color { r: 0.8,  g: 0.8,  b: 0.4,  a: 1.0 }, // lime     (Nora)
    Color { r: 0.6,  g: 0.2,  b: 0.2,  a: 1.0 }, // maroon   (Dash)
    Color { r: 0.3,  g: 0.7,  b: 0.5,  a: 1.0 }, // teal     (Ivy)
    // Stage 3 additions
    Color { r: 0.75, g: 0.1,  b: 0.1,  a: 1.0 }, // crimson  (Victor)
    Color { r: 0.6,  g: 0.6,  b: 0.65, a: 1.0 }, // steel    (Hank)
    Color { r: 0.1,  g: 0.7,  b: 0.3,  a: 1.0 }, // emerald  (Cleo)
    Color { r: 0.85, g: 0.75, b: 0.5,  a: 1.0 }, // wheat    (Ward)
    Color { r: 0.7,  g: 0.6,  b: 0.9,  a: 1.0 }, // lavender (Faye)
    Color { r: 1.0,  g: 0.4,  b: 0.3,  a: 1.0 }, // coral    (Gus)
    Color { r: 0.1,  g: 0.2,  b: 0.7,  a: 1.0 }, // navy     (Mira)
    Color { r: 1.0,  g: 0.8,  b: 0.0,  a: 1.0 }, // golden   (Sol)
    Color { r: 1.0,  g: 0.2,  b: 0.7,  a: 1.0 }, // magenta  (Pip)
    Color { r: 0.4,  g: 0.7,  b: 0.9,  a: 1.0 }, // sky      (Doc)
    Color { r: 0.2,  g: 0.7,  b: 0.5,  a: 1.0 }, // seafoam  (Vera)
    Color { r: 0.35, g: 0.35, b: 0.35, a: 1.0 }, // charcoal (Zed)
    Color { r: 0.1,  g: 0.5,  b: 0.2,  a: 1.0 }, // forest   (Ada)
    Color { r: 0.5,  g: 0.55, b: 0.6,  a: 1.0 }, // slate    (Cass)
    Color { r: 0.8,  g: 0.35, b: 0.1,  a: 1.0 }, // rust     (Holt)
    Color { r: 0.55, g: 0.25, b: 0.6,  a: 1.0 }, // plum     (Moe)
    Color { r: 0.5,  g: 0.7,  b: 0.9,  a: 1.0 }, // powder   (Rin)
    Color { r: 1.0,  g: 0.65, b: 0.5,  a: 1.0 }, // peach    (Tess)
    Color { r: 0.65, g: 0.3,  b: 0.15, a: 1.0 }, // sienna   (Bram)
    Color { r: 0.7,  g: 0.65, b: 0.45, a: 1.0 }, // khaki    (Dex)
    Color { r: 0.5,  g: 0.9,  b: 0.7,  a: 1.0 }, // mint     (Wyn)
    Color { r: 0.5,  g: 0.4,  b: 0.7,  a: 1.0 }, // dusk     (Sage)
    Color { r: 0.7,  g: 0.9,  b: 0.1,  a: 1.0 }, // chartreuse (Kit)
    Color { r: 0.6,  g: 0.4,  b: 0.25, a: 1.0 }, // umber    (Arlo)
    Color { r: 0.9,  g: 0.5,  b: 0.6,  a: 1.0 }, // rose     (spare)
];

// Hair colors cycle through a warm palette (independent of outfit).
const NPC_HAIR: [Color; 8] = [
    Color { r: 0.32, g: 0.18, b: 0.08, a: 1.0 }, // dark brown
    Color { r: 0.65, g: 0.42, b: 0.15, a: 1.0 }, // auburn
    Color { r: 0.85, g: 0.75, b: 0.35, a: 1.0 }, // blonde
    Color { r: 0.15, g: 0.12, b: 0.10, a: 1.0 }, // black
    Color { r: 0.72, g: 0.28, b: 0.10, a: 1.0 }, // red
    Color { r: 0.75, g: 0.72, b: 0.68, a: 1.0 }, // silver
    Color { r: 0.50, g: 0.25, b: 0.50, a: 1.0 }, // purple (fantasy)
    Color { r: 0.35, g: 0.55, b: 0.75, a: 1.0 }, // blue (fantasy)
];

// Skin tones cycle through a diverse set.
const NPC_SKIN: [Color; 6] = [
    Color { r: 0.96, g: 0.80, b: 0.62, a: 1.0 }, // light
    Color { r: 0.88, g: 0.68, b: 0.45, a: 1.0 }, // medium-light
    Color { r: 0.76, g: 0.55, b: 0.35, a: 1.0 }, // medium
    Color { r: 0.62, g: 0.42, b: 0.25, a: 1.0 }, // medium-dark
    Color { r: 0.48, g: 0.30, b: 0.16, a: 1.0 }, // dark
    Color { r: 0.90, g: 0.75, b: 0.55, a: 1.0 }, // warm
];

// Pants colors — varied so NPCs don't all wear the same jeans.
const NPC_PANTS: [Color; 6] = [
    Color { r: 0.22, g: 0.32, b: 0.62, a: 1.0 }, // denim
    Color { r: 0.30, g: 0.20, b: 0.12, a: 1.0 }, // brown
    Color { r: 0.12, g: 0.12, b: 0.12, a: 1.0 }, // black
    Color { r: 0.45, g: 0.45, b: 0.45, a: 1.0 }, // grey
    Color { r: 0.55, g: 0.40, b: 0.25, a: 1.0 }, // tan
    Color { r: 0.25, g: 0.45, b: 0.25, a: 1.0 }, // forest green
];

const SHOES: Color = Color { r: 0.22, g: 0.14, b: 0.08, a: 1.0 };

pub fn draw(npcs: &[NPC], camera: &Camera) {
    for (i, npc) in npcs.iter().enumerate() {
        let (col, row) = npc.tile;
        let (x, y) = camera.world_to_screen(col, row);

        let outfit = NPC_OUTFIT[i % NPC_OUTFIT.len()];
        let hair   = NPC_HAIR[(i * 3 + 1) % NPC_HAIR.len()];
        let skin   = NPC_SKIN[(i * 2 + 1) % NPC_SKIN.len()];
        let pants  = NPC_PANTS[(i + 2) % NPC_PANTS.len()];

        // All NPCs face down by default (toward camera).
        draw_character(x, y, outfit, pants, SHOES, skin, hair, &Direction::Down);

        // Name label above sprite (small, readable)
        let label = format!("{}", &npc.name[..npc.name.len().min(5)]);
        let lw = measure_text(&label, None, 11, 1.0).width;
        // Label background for readability
        draw_rectangle(x + TILE_SIZE / 2.0 - lw / 2.0 - 2.0, y - 13.0, lw + 4.0, 12.0,
                       Color { r: 0.0, g: 0.0, b: 0.0, a: 0.45 });
        draw_text(&label, x + TILE_SIZE / 2.0 - lw / 2.0, y - 3.0, 11.0, WHITE);
    }
}
