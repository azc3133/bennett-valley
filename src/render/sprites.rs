use macroquad::prelude::*;
use std::collections::HashMap;
use std::cell::RefCell;

thread_local! {
    static ATLAS: RefCell<Option<SpriteAtlas>> = RefCell::new(None);
}

/// Initialize the global sprite atlas. Call once at startup.
pub fn init_atlas(atlas: SpriteAtlas) {
    ATLAS.with(|a| *a.borrow_mut() = Some(atlas));
}

/// Draw a tile sprite from the global atlas.
pub fn draw_sprite_tile(name: &str, x: f32, y: f32, tile_size: f32) {
    ATLAS.with(|a| {
        if let Some(atlas) = a.borrow().as_ref() {
            atlas.draw_tile(name, x, y, tile_size);
        }
    });
}

/// Draw a sprite with custom size from the global atlas.
pub fn draw_sprite_sized(name: &str, x: f32, y: f32, w: f32, h: f32) {
    ATLAS.with(|a| {
        if let Some(atlas) = a.borrow().as_ref() {
            atlas.draw_sized(name, x, y, w, h);
        }
    });
}

/// Sprite atlas — holds all pixel art textures generated at runtime.
pub struct SpriteAtlas {
    pub textures: HashMap<String, Texture2D>,
}

impl SpriteAtlas {
    /// Generate all sprite textures. Call once at startup.
    pub fn generate() -> Self {
        let mut textures = HashMap::new();

        // Grass tile (32x32 pixel art)
        textures.insert("grass".into(), make_texture(&GRASS_PIXELS, 16, 16));
        textures.insert("grass2".into(), make_texture(&GRASS2_PIXELS, 16, 16));
        textures.insert("dirt".into(), make_texture(&DIRT_PIXELS, 16, 16));
        textures.insert("water".into(), make_texture(&WATER_PIXELS, 16, 16));
        textures.insert("path".into(), make_texture(&PATH_PIXELS, 16, 16));

        // Player (16x16 facing down)
        textures.insert("player_down".into(), make_texture(&PLAYER_DOWN, 16, 16));
        textures.insert("player_up".into(), make_texture(&PLAYER_UP, 16, 16));
        textures.insert("player_walk1".into(), make_texture(&PLAYER_WALK1, 16, 16));
        textures.insert("player_walk2".into(), make_texture(&PLAYER_WALK2, 16, 16));

        // Tree (16x32 — tall)
        textures.insert("tree_oak".into(), make_texture(&TREE_OAK, 16, 32));
        textures.insert("tree_pine".into(), make_texture(&TREE_PINE, 16, 32));

        // Crops
        textures.insert("crop_seed".into(), make_texture(&CROP_SEED, 16, 16));
        textures.insert("crop_sprout".into(), make_texture(&CROP_SPROUT, 16, 16));
        textures.insert("crop_grow".into(), make_texture(&CROP_GROW, 16, 16));
        textures.insert("crop_mature".into(), make_texture(&CROP_MATURE, 16, 16));

        Self { textures }
    }

    pub fn get(&self, name: &str) -> Option<&Texture2D> {
        self.textures.get(name)
    }

    /// Draw a sprite scaled to fill a tile.
    pub fn draw_tile(&self, name: &str, x: f32, y: f32, tile_size: f32) {
        if let Some(tex) = self.textures.get(name) {
            draw_texture_ex(
                tex,
                x, y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(tile_size, tile_size)),
                    ..Default::default()
                },
            );
        }
    }

    /// Draw a sprite with custom height (for tall sprites like trees).
    pub fn draw_sized(&self, name: &str, x: f32, y: f32, w: f32, h: f32) {
        if let Some(tex) = self.textures.get(name) {
            draw_texture_ex(
                tex,
                x, y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(w, h)),
                    ..Default::default()
                },
            );
        }
    }
}

/// Create a Texture2D from a flat array of hex color values.
/// 0 = transparent.
fn make_texture(pixels: &[u32], w: u16, h: u16) -> Texture2D {
    let mut img = Image::gen_image_color(w, h, Color::new(0.0, 0.0, 0.0, 0.0));
    for py in 0..h {
        for px in 0..w {
            let idx = (py as usize) * (w as usize) + (px as usize);
            if idx < pixels.len() && pixels[idx] != 0 {
                let c = pixels[idx];
                let r = ((c >> 16) & 0xFF) as f32 / 255.0;
                let g = ((c >> 8) & 0xFF) as f32 / 255.0;
                let b = (c & 0xFF) as f32 / 255.0;
                img.set_pixel(px as u32, py as u32, Color::new(r, g, b, 1.0));
            }
        }
    }
    let tex = Texture2D::from_image(&img);
    tex.set_filter(FilterMode::Nearest); // crisp pixel art, no blurring
    tex
}

// ── Pixel art data ───────────────────────────────────────────────────────────
// Each sprite is a flat array of hex colors (0xRRGGBB). 0 = transparent.
// Using functions instead of const to allow local aliases.

// Grass tile 16x16
const GRASS_PIXELS: [u32; 256] = {
    let g1 = 0x5a8a3c; // base green
    let g2 = 0x68a044; // light green
    let g3 = 0x4e7830; // dark green
    let g4 = 0x6aaa4c; // highlight
    [
        g1,g1,g1,g2,g1,g1,g3,g1,g1,g1,g2,g1,g1,g1,g3,g1,
        g1,g2,g1,g1,g1,g3,g1,g1,g2,g1,g1,g1,g3,g1,g1,g2,
        g1,g1,g3,g1,g2,g1,g1,g4,g1,g3,g1,g2,g1,g1,g1,g1,
        g3,g1,g1,g1,g1,g1,g2,g1,g1,g1,g1,g1,g1,g2,g1,g3,
        g1,g1,g2,g1,g1,g3,g1,g1,g1,g2,g1,g1,g3,g1,g1,g1,
        g1,g3,g1,g4,g1,g1,g1,g2,g1,g1,g1,g1,g1,g4,g1,g1,
        g2,g1,g1,g1,g1,g2,g1,g1,g3,g1,g2,g1,g1,g1,g1,g2,
        g1,g1,g1,g3,g1,g1,g1,g1,g1,g1,g1,g3,g1,g1,g1,g1,
        g1,g2,g1,g1,g1,g1,g3,g1,g1,g2,g1,g1,g1,g1,g3,g1,
        g1,g1,g1,g2,g3,g1,g1,g2,g1,g1,g4,g1,g1,g2,g1,g1,
        g3,g1,g1,g1,g1,g2,g1,g1,g1,g1,g1,g2,g1,g1,g1,g3,
        g1,g1,g4,g1,g1,g1,g1,g3,g1,g1,g1,g1,g3,g1,g2,g1,
        g1,g2,g1,g1,g3,g1,g1,g1,g2,g1,g1,g1,g1,g1,g1,g1,
        g1,g1,g1,g2,g1,g1,g2,g1,g1,g3,g1,g2,g1,g1,g3,g1,
        g3,g1,g1,g1,g1,g1,g1,g3,g1,g1,g1,g1,g1,g2,g1,g1,
        g1,g1,g2,g1,g3,g1,g1,g1,g1,g2,g1,g1,g3,g1,g1,g2,
    ]
};

const GRASS2_PIXELS: [u32; 256] = {
    let g1 = 0x5a8a3c;
    let g2 = 0x4e7830;
    let g3 = 0x68a044;
    [
        g1,g1,g2,g1,g1,g1,g3,g1,g1,g2,g1,g1,g1,g1,g2,g1,
        g1,g3,g1,g1,g2,g1,g1,g1,g1,g1,g3,g1,g2,g1,g1,g1,
        g2,g1,g1,g3,g1,g1,g1,g2,g1,g1,g1,g1,g1,g3,g1,g2,
        g1,g1,g1,g1,g1,g3,g1,g1,g3,g1,g1,g2,g1,g1,g1,g1,
        g1,g2,g1,g1,g1,g1,g2,g1,g1,g1,g1,g1,g3,g1,g1,g1,
        g1,g1,g3,g1,g2,g1,g1,g1,g2,g1,g2,g1,g1,g1,g2,g1,
        g3,g1,g1,g1,g1,g1,g3,g1,g1,g1,g1,g1,g1,g2,g1,g3,
        g1,g1,g2,g1,g1,g1,g1,g2,g1,g3,g1,g1,g2,g1,g1,g1,
        g1,g1,g1,g3,g1,g2,g1,g1,g1,g1,g2,g1,g1,g1,g3,g1,
        g2,g1,g1,g1,g1,g1,g1,g3,g1,g1,g1,g1,g1,g1,g1,g2,
        g1,g3,g1,g2,g1,g1,g1,g1,g2,g1,g3,g1,g1,g2,g1,g1,
        g1,g1,g1,g1,g3,g1,g2,g1,g1,g1,g1,g2,g1,g1,g1,g1,
        g1,g2,g1,g1,g1,g1,g1,g1,g3,g1,g1,g1,g3,g1,g2,g1,
        g1,g1,g1,g1,g2,g3,g1,g1,g1,g1,g2,g1,g1,g1,g1,g3,
        g3,g1,g2,g1,g1,g1,g1,g2,g1,g1,g1,g3,g1,g1,g1,g1,
        g1,g1,g1,g1,g1,g1,g3,g1,g1,g2,g1,g1,g1,g2,g1,g1,
    ]
};

// Dirt/tilled soil 16x16
const DIRT_PIXELS: [u32; 256] = {
    let d1 = 0x8b5e3c; // base brown
    let d2 = 0x6b4020; // dark furrow
    let d3 = 0x9b6e4c; // light brown
    [
        d1,d1,d1,d1,d1,d1,d1,d1,d1,d1,d1,d1,d1,d1,d1,d1,
        d1,d1,d3,d1,d1,d1,d1,d3,d1,d1,d1,d3,d1,d1,d1,d1,
        d2,d2,d2,d2,d2,d2,d2,d2,d2,d2,d2,d2,d2,d2,d2,d2,
        d1,d1,d1,d1,d1,d1,d1,d1,d1,d1,d1,d1,d1,d1,d1,d1,
        d1,d3,d1,d1,d1,d3,d1,d1,d1,d3,d1,d1,d1,d3,d1,d1,
        d2,d2,d2,d2,d2,d2,d2,d2,d2,d2,d2,d2,d2,d2,d2,d2,
        d1,d1,d1,d1,d1,d1,d1,d1,d1,d1,d1,d1,d1,d1,d1,d1,
        d1,d1,d1,d3,d1,d1,d1,d1,d3,d1,d1,d1,d1,d3,d1,d1,
        d2,d2,d2,d2,d2,d2,d2,d2,d2,d2,d2,d2,d2,d2,d2,d2,
        d1,d1,d1,d1,d1,d1,d1,d1,d1,d1,d1,d1,d1,d1,d1,d1,
        d1,d3,d1,d1,d3,d1,d1,d1,d3,d1,d1,d3,d1,d1,d1,d1,
        d2,d2,d2,d2,d2,d2,d2,d2,d2,d2,d2,d2,d2,d2,d2,d2,
        d1,d1,d1,d1,d1,d1,d1,d1,d1,d1,d1,d1,d1,d1,d1,d1,
        d1,d1,d3,d1,d1,d1,d3,d1,d1,d1,d1,d3,d1,d1,d3,d1,
        d2,d2,d2,d2,d2,d2,d2,d2,d2,d2,d2,d2,d2,d2,d2,d2,
        d1,d1,d1,d1,d1,d1,d1,d1,d1,d1,d1,d1,d1,d1,d1,d1,
    ]
};

// Water tile 16x16
const WATER_PIXELS: [u32; 256] = {
    let w1 = 0x1a6faa; // deep blue
    let w2 = 0x2a7fba; // mid blue
    let w3 = 0x3a8fca; // light shimmer
    let w4 = 0x4a9fdf; // bright shimmer
    [
        w1,w1,w1,w1,w2,w1,w1,w1,w1,w1,w2,w1,w1,w1,w1,w1,
        w1,w1,w2,w1,w1,w1,w1,w1,w1,w1,w1,w1,w2,w1,w1,w1,
        w1,w1,w1,w1,w1,w3,w3,w1,w1,w1,w1,w1,w1,w1,w1,w1,
        w1,w2,w1,w1,w3,w4,w4,w3,w1,w1,w2,w1,w1,w1,w2,w1,
        w1,w1,w1,w1,w1,w3,w3,w1,w1,w1,w1,w1,w1,w1,w1,w1,
        w1,w1,w1,w1,w1,w1,w1,w1,w1,w1,w1,w1,w1,w1,w1,w1,
        w1,w1,w1,w2,w1,w1,w1,w1,w1,w2,w1,w1,w1,w1,w1,w1,
        w1,w1,w1,w1,w1,w1,w1,w1,w1,w1,w1,w1,w1,w2,w1,w1,
        w1,w1,w1,w1,w1,w1,w1,w1,w1,w1,w1,w1,w1,w1,w1,w1,
        w1,w1,w1,w1,w1,w1,w1,w1,w1,w1,w3,w3,w1,w1,w1,w1,
        w1,w2,w1,w1,w1,w1,w2,w1,w1,w3,w4,w4,w3,w1,w1,w1,
        w1,w1,w1,w1,w1,w1,w1,w1,w1,w1,w3,w3,w1,w1,w2,w1,
        w1,w1,w1,w1,w1,w1,w1,w1,w1,w1,w1,w1,w1,w1,w1,w1,
        w1,w1,w2,w1,w1,w1,w1,w2,w1,w1,w1,w1,w1,w1,w1,w1,
        w1,w1,w1,w1,w1,w1,w1,w1,w1,w1,w1,w2,w1,w1,w1,w1,
        w1,w1,w1,w1,w2,w1,w1,w1,w1,w1,w1,w1,w1,w2,w1,w1,
    ]
};

// Path tile 16x16
const PATH_PIXELS: [u32; 256] = {
    let p1 = 0xc8b89a; // sandy base
    let p2 = 0xb8a888; // darker sand
    let p3 = 0xd8c8aa; // light sand
    [
        p1,p1,p2,p1,p1,p1,p3,p1,p1,p2,p1,p1,p1,p1,p1,p1,
        p1,p3,p1,p1,p1,p1,p1,p1,p1,p1,p1,p3,p1,p1,p2,p1,
        p1,p1,p1,p1,p2,p1,p1,p1,p1,p1,p1,p1,p1,p1,p1,p1,
        p2,p1,p1,p1,p1,p1,p2,p1,p1,p1,p2,p1,p1,p3,p1,p1,
        p1,p1,p1,p3,p1,p1,p1,p1,p3,p1,p1,p1,p1,p1,p1,p2,
        p1,p1,p1,p1,p1,p1,p1,p1,p1,p1,p1,p1,p2,p1,p1,p1,
        p1,p2,p1,p1,p1,p2,p1,p1,p1,p1,p1,p1,p1,p1,p3,p1,
        p1,p1,p1,p1,p1,p1,p1,p3,p1,p2,p1,p1,p1,p1,p1,p1,
        p1,p1,p3,p1,p1,p1,p1,p1,p1,p1,p1,p1,p1,p2,p1,p1,
        p1,p1,p1,p1,p2,p1,p1,p1,p1,p1,p3,p1,p1,p1,p1,p1,
        p2,p1,p1,p1,p1,p1,p3,p1,p1,p1,p1,p1,p1,p1,p1,p2,
        p1,p1,p1,p1,p1,p1,p1,p1,p2,p1,p1,p2,p1,p1,p1,p1,
        p1,p3,p1,p2,p1,p1,p1,p1,p1,p1,p1,p1,p1,p3,p1,p1,
        p1,p1,p1,p1,p1,p2,p1,p1,p1,p1,p1,p1,p1,p1,p1,p1,
        p1,p1,p1,p1,p1,p1,p1,p1,p3,p1,p2,p1,p1,p1,p2,p1,
        p2,p1,p1,p3,p1,p1,p1,p1,p1,p1,p1,p1,p2,p1,p1,p1,
    ]
};

// Player facing down 16x16
const PLAYER_DOWN: [u32; 256] = {

    let S = 0xf5cc9a; // skin
    let H = 0x523012; // hair
    let W = 0xffffff; // white (eyes)
    let E = 0x2a1800; // eye pupil
    let T = 0xe6ecff; // shirt
    let P = 0x385294; // pants
    let B = 0x382414; // shoes/boots
    let M = 0x9b5020; // mouth
    [
        0,0,0,0,0,H,H,H,H,H,H,0,0,0,0,0,
        0,0,0,0,H,H,H,H,H,H,H,H,0,0,0,0,
        0,0,0,0,H,H,H,H,H,H,H,H,0,0,0,0,
        0,0,0,0,H,S,S,S,S,S,S,H,0,0,0,0,
        0,0,0,0,S,S,W,E,S,W,E,S,0,0,0,0,
        0,0,0,0,S,S,S,S,S,S,S,S,0,0,0,0,
        0,0,0,0,0,S,S,M,M,S,S,0,0,0,0,0,
        0,0,0,0,0,0,S,S,S,S,0,0,0,0,0,0,
        0,0,T,T,T,T,T,T,T,T,T,T,T,T,0,0,
        0,0,T,T,T,T,T,T,T,T,T,T,T,T,0,0,
        0,0,T,T,T,T,T,T,T,T,T,T,T,T,0,0,
        0,0,S,S,0,T,T,T,T,T,T,0,S,S,0,0,
        0,0,0,0,0,P,P,P,P,P,P,0,0,0,0,0,
        0,0,0,0,0,P,P,0,0,P,P,0,0,0,0,0,
        0,0,0,0,0,B,B,0,0,B,B,0,0,0,0,0,
        0,0,0,0,B,B,B,0,0,B,B,B,0,0,0,0,
    ]
};

// Player facing up 16x16
const PLAYER_UP: [u32; 256] = {
    let _ = 0;
    let S = 0xf5cc9a;
    let H = 0x523012;
    let T = 0xe6ecff;
    let P = 0x385294;
    let B = 0x382414;
    [
        0,0,0,0,0,H,H,H,H,H,H,0,0,0,0,0,
        0,0,0,0,H,H,H,H,H,H,H,H,0,0,0,0,
        0,0,0,0,H,H,H,H,H,H,H,H,0,0,0,0,
        0,0,0,0,H,H,H,H,H,H,H,H,0,0,0,0,
        0,0,0,0,H,H,H,H,H,H,H,H,0,0,0,0,
        0,0,0,0,S,H,H,H,H,H,H,S,0,0,0,0,
        0,0,0,0,0,S,S,S,S,S,S,0,0,0,0,0,
        0,0,0,0,0,0,S,S,S,S,0,0,0,0,0,0,
        0,0,T,T,T,T,T,T,T,T,T,T,T,T,0,0,
        0,0,T,T,T,T,T,T,T,T,T,T,T,T,0,0,
        0,0,T,T,T,T,T,T,T,T,T,T,T,T,0,0,
        0,0,S,S,0,T,T,T,T,T,T,0,S,S,0,0,
        0,0,0,0,0,P,P,P,P,P,P,0,0,0,0,0,
        0,0,0,0,0,P,P,0,0,P,P,0,0,0,0,0,
        0,0,0,0,0,B,B,0,0,B,B,0,0,0,0,0,
        0,0,0,0,B,B,B,0,0,B,B,B,0,0,0,0,
    ]
};

// Player walk frame 1 (left foot forward)
const PLAYER_WALK1: [u32; 256] = {
    let _ = 0;
    let S = 0xf5cc9a;
    let H = 0x523012;
    let W = 0xffffff;
    let E = 0x2a1800;
    let T = 0xe6ecff;
    let P = 0x385294;
    let B = 0x382414;
    let M = 0x9b5020;
    [
        0,0,0,0,0,H,H,H,H,H,H,0,0,0,0,0,
        0,0,0,0,H,H,H,H,H,H,H,H,0,0,0,0,
        0,0,0,0,H,H,H,H,H,H,H,H,0,0,0,0,
        0,0,0,0,H,S,S,S,S,S,S,H,0,0,0,0,
        0,0,0,0,S,S,W,E,S,W,E,S,0,0,0,0,
        0,0,0,0,S,S,S,S,S,S,S,S,0,0,0,0,
        0,0,0,0,0,S,S,M,M,S,S,0,0,0,0,0,
        0,0,0,0,0,0,S,S,S,S,0,0,0,0,0,0,
        0,0,T,T,T,T,T,T,T,T,T,T,T,T,0,0,
        0,S,S,T,T,T,T,T,T,T,T,T,T,S,S,0,
        0,0,0,T,T,T,T,T,T,T,T,T,T,0,0,0,
        0,0,0,0,0,T,T,T,T,T,T,0,0,0,0,0,
        0,0,0,0,P,P,P,P,P,P,P,P,0,0,0,0,
        0,0,0,0,P,P,0,0,0,0,P,P,0,0,0,0,
        0,0,0,B,B,B,0,0,0,0,0,B,B,0,0,0,
        0,0,B,B,B,0,0,0,0,0,0,0,B,B,0,0,
    ]
};

// Player walk frame 2 (right foot forward)
const PLAYER_WALK2: [u32; 256] = {
    let _ = 0;
    let S = 0xf5cc9a;
    let H = 0x523012;
    let W = 0xffffff;
    let E = 0x2a1800;
    let T = 0xe6ecff;
    let P = 0x385294;
    let B = 0x382414;
    let M = 0x9b5020;
    [
        0,0,0,0,0,H,H,H,H,H,H,0,0,0,0,0,
        0,0,0,0,H,H,H,H,H,H,H,H,0,0,0,0,
        0,0,0,0,H,H,H,H,H,H,H,H,0,0,0,0,
        0,0,0,0,H,S,S,S,S,S,S,H,0,0,0,0,
        0,0,0,0,S,S,W,E,S,W,E,S,0,0,0,0,
        0,0,0,0,S,S,S,S,S,S,S,S,0,0,0,0,
        0,0,0,0,0,S,S,M,M,S,S,0,0,0,0,0,
        0,0,0,0,0,0,S,S,S,S,0,0,0,0,0,0,
        0,0,T,T,T,T,T,T,T,T,T,T,T,T,0,0,
        0,S,S,T,T,T,T,T,T,T,T,T,T,S,S,0,
        0,0,0,T,T,T,T,T,T,T,T,T,T,0,0,0,
        0,0,0,0,0,T,T,T,T,T,T,0,0,0,0,0,
        0,0,0,0,P,P,P,P,P,P,P,P,0,0,0,0,
        0,0,0,0,P,P,0,0,0,0,P,P,0,0,0,0,
        0,0,0,0,0,B,B,0,0,B,B,B,0,0,0,0,
        0,0,0,0,0,0,B,B,B,B,B,0,0,0,0,0,
    ]
};

// Oak tree 16x32 (tall sprite)
const TREE_OAK: [u32; 512] = {
    let _ = 0;
    let L = 0x3d9a3d; // light leaf
    let G = 0x2d7a2d; // mid leaf
    let D = 0x1e6a1e; // dark leaf
    let T = 0x6b4226; // trunk
    let K = 0x543218; // bark detail
    [
        // Row 0-7: canopy
        0,0,0,0,0,0,G,G,G,G,0,0,0,0,0,0,
        0,0,0,0,G,G,L,L,L,L,G,G,0,0,0,0,
        0,0,0,G,G,L,L,L,L,L,L,G,G,0,0,0,
        0,0,G,G,L,L,L,G,G,L,L,L,G,G,0,0,
        0,G,G,L,L,L,G,D,D,G,L,L,L,G,G,0,
        0,G,L,L,G,L,L,G,G,L,L,G,L,L,G,0,
        G,G,L,G,D,G,L,L,L,L,G,D,G,L,G,G,
        G,L,G,D,D,D,G,L,L,G,D,D,D,G,L,G,
        // Row 8-15: lower canopy + top trunk
        G,G,D,D,G,D,D,G,G,D,D,G,D,D,G,G,
        0,G,G,D,D,D,G,G,G,G,D,D,D,G,G,0,
        0,0,G,G,G,D,D,G,G,D,D,G,G,G,0,0,
        0,0,0,G,G,G,G,T,T,G,G,G,G,0,0,0,
        0,0,0,0,0,0,T,T,T,T,0,0,0,0,0,0,
        0,0,0,0,0,0,T,K,K,T,0,0,0,0,0,0,
        0,0,0,0,0,0,T,T,T,T,0,0,0,0,0,0,
        0,0,0,0,0,0,T,K,T,T,0,0,0,0,0,0,
        // Row 16-23: trunk
        0,0,0,0,0,0,T,T,T,T,0,0,0,0,0,0,
        0,0,0,0,0,0,T,K,K,T,0,0,0,0,0,0,
        0,0,0,0,0,0,T,T,T,T,0,0,0,0,0,0,
        0,0,0,0,0,0,T,K,T,T,0,0,0,0,0,0,
        0,0,0,0,0,0,T,T,T,T,0,0,0,0,0,0,
        0,0,0,0,0,0,T,T,T,T,0,0,0,0,0,0,
        0,0,0,0,0,T,T,T,T,T,T,0,0,0,0,0,
        0,0,0,0,T,T,T,T,T,T,T,T,0,0,0,0,
        // Row 24-31: roots/base
        0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
        0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
        0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
        0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
        0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
        0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
        0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
        0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
    ]
};

// Pine tree 16x32
const TREE_PINE: [u32; 512] = {
    let _ = 0;
    let L = 0x2d8a2d; // light pine
    let G = 0x226b22; // mid pine
    let D = 0x1a5a1a; // dark pine
    let T = 0x5a3818; // trunk
    [
        0,0,0,0,0,0,0,L,L,0,0,0,0,0,0,0,
        0,0,0,0,0,0,L,L,L,L,0,0,0,0,0,0,
        0,0,0,0,0,L,G,L,L,G,L,0,0,0,0,0,
        0,0,0,0,L,G,G,L,L,G,G,L,0,0,0,0,
        0,0,0,L,G,G,D,G,G,D,G,G,L,0,0,0,
        0,0,0,0,0,L,G,L,L,G,L,0,0,0,0,0,
        0,0,0,0,L,G,G,L,L,G,G,L,0,0,0,0,
        0,0,0,L,G,G,D,G,G,D,G,G,L,0,0,0,
        0,0,L,G,G,D,D,G,G,D,D,G,G,L,0,0,
        0,0,0,0,L,G,G,L,L,G,G,L,0,0,0,0,
        0,0,0,L,G,G,D,G,G,D,G,G,L,0,0,0,
        0,0,L,G,G,D,D,G,G,D,D,G,G,L,0,0,
        0,L,G,G,D,D,D,G,G,D,D,D,G,G,L,0,
        L,G,G,D,D,D,D,G,G,D,D,D,D,G,G,L,
        0,0,0,0,0,0,T,T,T,T,0,0,0,0,0,0,
        0,0,0,0,0,0,T,T,T,T,0,0,0,0,0,0,
        0,0,0,0,0,0,T,T,T,T,0,0,0,0,0,0,
        0,0,0,0,0,0,T,T,T,T,0,0,0,0,0,0,
        0,0,0,0,0,0,T,T,T,T,0,0,0,0,0,0,
        0,0,0,0,0,0,T,T,T,T,0,0,0,0,0,0,
        0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
        0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
        0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
        0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
        0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
        0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
        0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
        0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
        0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
        0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
        0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
        0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
    ]
};

// Crop seed 16x16
const CROP_SEED: [u32; 256] = {
    let _ = 0;
    let D = 0x8b5e3c; // dirt
    let S = 0x9b7040; // seed
    [
        D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,
        D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,
        D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,
        D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,
        D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,
        D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,
        D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,
        D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,
        D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,
        D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,
        D,D,D,D,D,D,S,S,S,S,D,D,D,D,D,D,
        D,D,D,D,D,S,S,S,S,S,S,D,D,D,D,D,
        D,D,D,D,D,D,S,S,S,S,D,D,D,D,D,D,
        D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,
        D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,
        D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,
    ]
};

// Crop sprout 16x16
const CROP_SPROUT: [u32; 256] = {
    let _ = 0;
    let D = 0x8b5e3c;
    let G = 0x3a8a2a; // green stem
    let L = 0x5aaa3a; // leaf
    [
        D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,
        D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,
        D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,
        D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,
        D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,
        D,D,D,D,D,D,D,G,G,D,D,D,D,D,D,D,
        D,D,D,D,D,D,D,G,G,D,D,D,D,D,D,D,
        D,D,D,D,D,L,L,G,G,L,L,D,D,D,D,D,
        D,D,D,D,L,L,L,G,G,L,L,L,D,D,D,D,
        D,D,D,D,D,L,L,G,G,L,L,D,D,D,D,D,
        D,D,D,D,D,D,D,G,G,D,D,D,D,D,D,D,
        D,D,D,D,D,D,D,G,G,D,D,D,D,D,D,D,
        D,D,D,D,D,D,D,G,G,D,D,D,D,D,D,D,
        D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,
        D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,
        D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,
    ]
};

// Crop growing 16x16
const CROP_GROW: [u32; 256] = {
    let _ = 0;
    let D = 0x8b5e3c;
    let G = 0x3a8a2a;
    let L = 0x5aaa3a;
    [
        D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,
        D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,
        D,D,D,D,D,D,D,G,G,D,D,D,D,D,D,D,
        D,D,D,D,D,D,D,G,G,D,D,D,D,D,D,D,
        D,D,D,D,L,L,L,G,G,L,L,L,D,D,D,D,
        D,D,D,L,L,L,L,G,G,L,L,L,L,D,D,D,
        D,D,D,D,L,L,L,G,G,L,L,L,D,D,D,D,
        D,D,D,D,D,D,D,G,G,D,D,D,D,D,D,D,
        D,D,D,D,D,L,L,G,G,L,L,D,D,D,D,D,
        D,D,D,D,L,L,L,G,G,L,L,L,D,D,D,D,
        D,D,D,D,D,L,L,G,G,L,L,D,D,D,D,D,
        D,D,D,D,D,D,D,G,G,D,D,D,D,D,D,D,
        D,D,D,D,D,D,D,G,G,D,D,D,D,D,D,D,
        D,D,D,D,D,D,D,G,G,D,D,D,D,D,D,D,
        D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,
        D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,
    ]
};

// Crop mature 16x16 (with red produce)
const CROP_MATURE: [u32; 256] = {
    let _ = 0;
    let D = 0x8b5e3c;
    let G = 0x3a8a2a;
    let L = 0x5aaa3a;
    let R = 0xe74c3c; // red produce
    let Y = 0xf1c40f; // yellow highlight
    [
        D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,
        D,D,D,D,D,D,R,R,R,R,D,D,D,D,D,D,
        D,D,D,D,D,R,R,Y,R,R,R,D,D,D,D,D,
        D,D,D,D,D,R,R,R,R,R,R,D,D,D,D,D,
        D,D,D,D,L,L,R,R,R,R,L,L,D,D,D,D,
        D,D,D,L,L,L,L,G,G,L,L,L,L,D,D,D,
        D,D,D,D,L,L,L,G,G,L,L,L,D,D,D,D,
        D,D,D,D,D,D,D,G,G,D,D,D,D,D,D,D,
        D,D,D,D,D,L,L,G,G,L,L,D,D,D,D,D,
        D,D,D,D,L,L,L,G,G,L,L,L,D,D,D,D,
        D,D,D,D,D,L,L,G,G,L,L,D,D,D,D,D,
        D,D,D,D,D,D,D,G,G,D,D,D,D,D,D,D,
        D,D,D,D,D,D,D,G,G,D,D,D,D,D,D,D,
        D,D,D,D,D,D,D,G,G,D,D,D,D,D,D,D,
        D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,
        D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,D,
    ]
};
