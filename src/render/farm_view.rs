use macroquad::prelude::*;
use crate::game::world::{FarmMap, TileKind};
use crate::game::crop::{CropKind, CropState};
use crate::render::camera::{Camera, TILE_SIZE};

const TS: f32 = TILE_SIZE;

pub fn draw(map: &FarmMap, camera: &Camera, house_upgraded: bool, hour: u8) {
    draw_full(map, camera, house_upgraded, hour, false, false);
}

pub fn draw_rain(map: &FarmMap, camera: &Camera, house_upgraded: bool, hour: u8) {
    draw_full(map, camera, house_upgraded, hour, true, false);
}

pub fn draw_rainbow(map: &FarmMap, camera: &Camera, house_upgraded: bool, hour: u8) {
    draw_full(map, camera, house_upgraded, hour, false, true);
}

fn draw_full(map: &FarmMap, camera: &Camera, house_upgraded: bool, hour: u8, raining: bool, rainbow: bool) {
    // Pass 1: base tiles
    for row in 0..map.height {
        for col in 0..map.width {
            let tile = &map.tiles[row][col];
            let (x, y) = camera.world_to_screen(col, row);

            // Buildings drawn as overlays — show grass underneath them
            if matches!(tile.kind, TileKind::Farmhouse | TileKind::Shop) {
                draw_grass(x, y, col, row);
                continue;
            }

            draw_tile(&tile.kind, x, y, col, row);

            if let Some(crop) = &tile.crop {
                draw_crop(crop, x, y);
            }
        }
    }

    // Pass 2: building overlays (drawn after tiles so they sit on top cleanly)
    let (fx, fy) = camera.world_to_screen(1, 1);
    draw_farmhouse(fx, fy, house_upgraded);

    let (sx, sy) = camera.world_to_screen(36, 1);
    draw_shop(sx, sy);

    // Town buildings
    let (ix, iy) = camera.world_to_screen(41, 2);
    draw_inn(ix, iy);

    let (mx, my) = camera.world_to_screen(49, 2);
    draw_market(mx, my);

    let (tx, ty) = camera.world_to_screen(57, 2);
    draw_tavern(tx, ty);

    let (cx, cy) = camera.world_to_screen(65, 2);
    draw_clinic(cx, cy);

    let (lx, ly) = camera.world_to_screen(41, 7);
    draw_library(lx, ly);

    let (hx, hy) = camera.world_to_screen(65, 7);
    draw_town_hall(hx, hy);

    // Furniture Shop
    let (fsx, fsy) = camera.world_to_screen(49, 7);
    draw_furniture_shop(fsx, fsy);

    // NPC cottages
    let cottage_positions: [(usize, usize, u32); 8] = [
        (42, 15, 0x2980b9), // blue roof
        (45, 15, 0x27ae60), // green roof
        (42, 18, 0xc0392b), // red roof
        (45, 18, 0xf39c12), // orange roof
        (55, 15, 0x8e44ad), // purple roof
        (55, 18, 0x2c3e50), // dark blue roof
        (71, 15, 0xd35400), // burnt orange roof
        (71, 18, 0x16a085), // teal roof
    ];
    for (col, row, roof_color) in cottage_positions {
        let (cx, cy) = camera.world_to_screen(col, row);
        draw_npc_cottage(cx, cy, Color::from_hex(roof_color));
    }

    // Street lamps
    let lamp_coords: &[(usize, usize)] = &[
        // Town main street (row 6)
        (43, 6), (47, 6), (51, 6), (55, 6), (59, 6), (63, 6), (67, 6), (71, 6),
        // Town north street (row 1)
        (45, 1), (53, 1), (61, 1), (69, 1),
        // Town mid street (row 13)
        (43, 13), (47, 13), (51, 13), (59, 13), (67, 13), (71, 13),
        // Town south exit (row 20)
        (44, 20), (52, 20), (60, 20), (68, 20),
        // Cross streets
        (48, 4), (48, 10), (48, 17),
        (56, 4), (56, 10), (56, 17),
        (64, 4),
        (72, 4), (72, 17),
        // Farm area
        (4, 4), (23, 4), (23, 20),
        // South road
        (10, 29), (20, 29), (30, 29),
    ];
    let is_night = hour >= 19 || hour < 6;
    for &(col, row) in lamp_coords {
        let (lx, ly) = camera.world_to_screen(col, row);
        draw_street_lamp(lx, ly, is_night);
    }

    // Restaurant
    let (rx, ry) = camera.world_to_screen(41, 21);
    draw_restaurant(rx, ry);

    // Arcade
    let (arx, ary) = camera.world_to_screen(49, 21);
    draw_arcade(arx, ary);

    // Swimming Pool
    let (spx, spy) = camera.world_to_screen(57, 21);
    draw_pool(spx, spy);

    // Ice Cream Shop
    let (icx, icy) = camera.world_to_screen(65, 21);
    draw_icecream_shop(icx, icy);

    // Beach decorations
    draw_beach(camera);

    // Animal Shop
    let (asx, asy) = camera.world_to_screen(63, 14);
    draw_animal_shop(asx, asy);

    // Playground
    let (px, py) = camera.world_to_screen(50, 14);
    draw_playground(px, py);

    // Rainbow arc
    if rainbow {
        let sw = screen_width();
        let sh = screen_height();
        let cx = sw / 2.0;
        let cy = sh * 0.7;
        let rainbow_colors: [(f32,f32,f32); 7] = [
            (1.0, 0.0, 0.0),   // red
            (1.0, 0.5, 0.0),   // orange
            (1.0, 1.0, 0.0),   // yellow
            (0.0, 0.8, 0.0),   // green
            (0.0, 0.5, 1.0),   // blue
            (0.3, 0.0, 0.8),   // indigo
            (0.6, 0.0, 1.0),   // violet
        ];
        for (i, &(r, g, b)) in rainbow_colors.iter().enumerate() {
            let radius = 300.0 + i as f32 * 12.0;
            let color = Color { r, g, b, a: 0.25 };
            // Draw arc using line segments
            for seg in 0..40 {
                let a1 = std::f32::consts::PI * (0.15 + seg as f32 * 0.0175);
                let a2 = std::f32::consts::PI * (0.15 + (seg + 1) as f32 * 0.0175);
                draw_line(
                    cx + radius * a1.cos(), cy - radius * a1.sin(),
                    cx + radius * a2.cos(), cy - radius * a2.sin(),
                    6.0, color,
                );
            }
        }
    }

    // Rain overlay
    if raining {
        let sw = screen_width();
        let sh = screen_height();
        // Grey overcast tint
        draw_rectangle(0.0, 0.0, sw, sh, Color { r: 0.3, g: 0.35, b: 0.4, a: 0.15 });
        // Rain drops — use time-based animation
        let time = macroquad::time::get_time() as f32;
        for i in 0..80 {
            let x = ((i as f32 * 137.3 + time * 40.0) % sw) as f32;
            let y = ((i as f32 * 97.7 + time * 280.0) % sh) as f32;
            let len = 8.0 + (i % 3) as f32 * 4.0;
            draw_line(x, y, x - 2.0, y + len, 1.0,
                      Color { r: 0.6, g: 0.7, b: 0.85, a: 0.35 });
        }
    }

    // Night overlay — darken the world after sunset
    if hour >= 19 {
        let darkness = match hour {
            19 => 0.1,  // dusk
            20 => 0.2,  // evening
            21 => 0.3,  // night
            _ =>  0.35, // deep night (22-26)
        };
        draw_rectangle(0.0, 0.0, screen_width(), screen_height(),
                       Color { r: 0.05, g: 0.05, b: 0.15, a: darkness });
    }
}

// ── Tile rendering ────────────────────────────────────────────────────────────

fn draw_tile(kind: &TileKind, x: f32, y: f32, col: usize, row: usize) {
    match kind {
        TileKind::Grass            => draw_grass(x, y, col, row),
        TileKind::Tilled           => draw_tilled(x, y),
        TileKind::Watered          => draw_watered(x, y),
        TileKind::Path             => draw_path(x, y, col, row),
        TileKind::Water            => draw_water(x, y, col, row),
        TileKind::ShipBox          => draw_ship_box(x, y),
        TileKind::ForagePatch      => draw_forage(x, y, col, row, true),
        TileKind::ForagePatchEmpty => draw_forage(x, y, col, row, false),
        TileKind::FishingSpot      => draw_fishing_spot(x, y),
        TileKind::Rock(hp)         => draw_rock(x, y, *hp),
        TileKind::Bench            => draw_bench(x, y),
        TileKind::OakTree          => draw_oak_tree(x, y, col, row, true),
        TileKind::OakTreeEmpty     => draw_oak_tree(x, y, col, row, false),
        TileKind::LongGrass        => draw_long_grass(x, y, col, row),
        TileKind::Farmhouse | TileKind::Shop => {} // handled in pass 2
    }
}

fn draw_grass(x: f32, y: f32, col: usize, row: usize) {
    // Base grass
    draw_rectangle(x, y, TS, TS, Color::from_hex(0x5a8a3c));
    // Subtle texture variation using position hash
    let h = (col.wrapping_mul(7).wrapping_add(row.wrapping_mul(13))) % 7;
    match h {
        0 => draw_rectangle(x + 3.0,  y + 3.0,  6.0, 5.0, Color::from_hex(0x68a044)),
        1 => draw_rectangle(x + 18.0, y + 14.0, 7.0, 4.0, Color::from_hex(0x4e7830)),
        2 => draw_rectangle(x + 8.0,  y + 20.0, 5.0, 6.0, Color::from_hex(0x68a044)),
        3 => {
            draw_rectangle(x + 2.0,  y + 16.0, 4.0, 3.0, Color::from_hex(0x4e7830));
            draw_rectangle(x + 22.0, y + 6.0,  4.0, 3.0, Color::from_hex(0x68a044));
        }
        _ => {}
    }
}

fn draw_long_grass(x: f32, y: f32, col: usize, row: usize) {
    // Base — darker green than normal grass
    draw_rectangle(x, y, TS, TS, Color::from_hex(0x4a7a2c));
    // Tall grass blades
    let h = (col.wrapping_mul(7).wrapping_add(row.wrapping_mul(13))) % 5;
    let blade_color = Color::from_hex(0x3a6a1c);
    let tip_color = Color::from_hex(0x6aaa3c);
    // Draw 4-6 grass blades with varying heights
    let blades: &[(f32, f32, f32)] = match h {
        0 => &[(3.0, 2.0, 14.0), (9.0, 4.0, 12.0), (16.0, 1.0, 15.0), (22.0, 3.0, 13.0), (12.0, 2.0, 11.0)],
        1 => &[(2.0, 3.0, 13.0), (7.0, 1.0, 15.0), (14.0, 2.0, 14.0), (20.0, 4.0, 11.0), (25.0, 2.0, 12.0)],
        2 => &[(4.0, 1.0, 15.0), (10.0, 3.0, 12.0), (17.0, 2.0, 14.0), (23.0, 1.0, 13.0)],
        3 => &[(1.0, 2.0, 14.0), (8.0, 1.0, 16.0), (13.0, 3.0, 11.0), (19.0, 2.0, 13.0), (24.0, 4.0, 12.0), (6.0, 2.0, 10.0)],
        _ => &[(5.0, 3.0, 13.0), (11.0, 1.0, 14.0), (18.0, 2.0, 12.0), (24.0, 3.0, 15.0)],
    };
    for &(bx, _by, bh) in blades {
        let base_y = y + TS - 2.0;
        // Blade body
        draw_rectangle(x + bx, base_y - bh, 2.0, bh, blade_color);
        // Blade tip (lighter)
        draw_rectangle(x + bx - 0.5, base_y - bh - 2.0, 3.0, 3.0, tip_color);
    }
}

fn draw_tilled(x: f32, y: f32) {
    // Soil base
    draw_rectangle(x, y, TS, TS, Color::from_hex(0x8b5e3c));
    // Furrow lines
    for i in 0..4u32 {
        let ly = y + 4.0 + i as f32 * 7.0;
        draw_line(x + 2.0, ly, x + TS - 3.0, ly, 1.5, Color::from_hex(0x6b4020));
    }
    // Edge shadow
    draw_rectangle(x, y + TS - 3.0, TS, 3.0, Color::from_hex(0x6b4020));
}

fn draw_watered(x: f32, y: f32) {
    // Dark wet soil
    draw_rectangle(x, y, TS, TS, Color::from_hex(0x5c3a1e));
    // Furrow lines with moisture sheen
    for i in 0..4u32 {
        let ly = y + 4.0 + i as f32 * 7.0;
        draw_line(x + 2.0, ly, x + TS - 3.0, ly, 1.5, Color::from_hex(0x3d2410));
    }
    // Moisture shimmer
    draw_rectangle(x + 6.0,  y + 2.0,  8.0, 1.5, Color { r: 0.4, g: 0.6, b: 0.9, a: 0.35 });
    draw_rectangle(x + 18.0, y + 12.0, 6.0, 1.5, Color { r: 0.4, g: 0.6, b: 0.9, a: 0.35 });
}

fn draw_path(x: f32, y: f32, col: usize, row: usize) {
    draw_rectangle(x, y, TS, TS, Color::from_hex(0xc8a96e));
    // Gravel/pebble texture
    let h = (col.wrapping_mul(11).wrapping_add(row.wrapping_mul(17))) % 5;
    let stone = Color::from_hex(0xa08050);
    match h {
        0 => { draw_rectangle(x + 4.0,  y + 5.0,  4.0, 3.0, stone);
               draw_rectangle(x + 20.0, y + 18.0, 3.0, 3.0, stone); }
        1 => { draw_rectangle(x + 14.0, y + 8.0,  3.0, 4.0, stone); }
        2 => { draw_rectangle(x + 6.0,  y + 20.0, 5.0, 3.0, stone);
               draw_rectangle(x + 20.0, y + 6.0,  3.0, 3.0, stone); }
        3 => { draw_rectangle(x + 10.0, y + 14.0, 4.0, 3.0, stone); }
        _ => {}
    }
}

fn draw_water(x: f32, y: f32, col: usize, row: usize) {
    // Deep water base
    draw_rectangle(x, y, TS, TS, Color::from_hex(0x1a6faa));
    // Wave shimmer — alternates by position
    let phase = (col + row) % 2;
    let shimmer = Color::from_hex(0x4a9fdf);
    if phase == 0 {
        draw_rectangle(x + 3.0,  y + 8.0,  14.0, 2.0, shimmer);
        draw_rectangle(x + 18.0, y + 18.0, 9.0,  2.0, shimmer);
    } else {
        draw_rectangle(x + 8.0,  y + 4.0,  9.0,  2.0, shimmer);
        draw_rectangle(x + 2.0,  y + 20.0, 14.0, 2.0, shimmer);
    }
    // Foam edge — top of tile
    draw_rectangle(x, y, TS, 2.0, Color { r: 0.7, g: 0.88, b: 1.0, a: 0.4 });
}

fn draw_ship_box(x: f32, y: f32) {
    // Crate body
    draw_rectangle(x + 2.0, y + 6.0, TS - 4.0, TS - 10.0, Color::from_hex(0xd4a017));
    // Lid
    draw_rectangle(x + 1.0, y + 3.0, TS - 2.0, 5.0, Color::from_hex(0xe8b828));
    // Wood grain lines
    draw_line(x + 10.0, y + 6.0, x + 10.0, y + TS - 4.0, 1.0, Color::from_hex(0xb88010));
    draw_line(x + 20.0, y + 6.0, x + 20.0, y + TS - 4.0, 1.0, Color::from_hex(0xb88010));
    draw_line(x + 2.0,  y + 14.0, x + TS - 2.0, y + 14.0, 1.0, Color::from_hex(0xb88010));
    // Lid handle
    draw_rectangle(x + 12.0, y + 1.0, 8.0, 3.0, Color::from_hex(0xa06010));
    // Gold star / stamp
    draw_circle(x + TS / 2.0, y + 18.0, 3.5, Color::from_hex(0xf1c40f));
}

fn draw_forage(x: f32, y: f32, col: usize, row: usize, has_item: bool) {
    // Grass base
    draw_grass(x, y, col, row);
    // Bush shape (three overlapping circles)
    let bush = if has_item { Color::from_hex(0x2d6e2d) } else { Color::from_hex(0x4a7a4a) };
    draw_circle(x + 16.0, y + 18.0, 8.0, bush);
    draw_circle(x + 10.0, y + 20.0, 6.0, bush);
    draw_circle(x + 22.0, y + 20.0, 6.0, bush);
    // Item indicator dots
    if has_item {
        let h = (col.wrapping_mul(5).wrapping_add(row.wrapping_mul(9))) % 3;
        let dot_color = match h {
            0 => Color::from_hex(0xe74c3c), // red berry
            1 => Color::from_hex(0xf39c12), // orange mushroom
            _ => Color::from_hex(0xecf0f1), // white flower
        };
        draw_circle(x + 14.0, y + 16.0, 2.5, dot_color);
        draw_circle(x + 20.0, y + 17.0, 2.5, dot_color);
        draw_circle(x + 17.0, y + 22.0, 2.0, dot_color);
    }
}

fn draw_bench(x: f32, y: f32) {
    // Grass base
    draw_rectangle(x, y, TS, TS, Color::from_hex(0x5a8a3c));
    // Bench seat
    draw_rectangle(x + 2.0, y + 14.0, TS - 4.0, 8.0, Color::from_hex(0xa0714a));
    // Back rest
    draw_rectangle(x + 2.0, y + 8.0, TS - 4.0, 4.0, Color::from_hex(0x8b5e3c));
    // Slats on seat
    draw_line(x + 4.0, y + 16.0, x + TS - 4.0, y + 16.0, 1.0, Color::from_hex(0x8b5e3c));
    draw_line(x + 4.0, y + 19.0, x + TS - 4.0, y + 19.0, 1.0, Color::from_hex(0x8b5e3c));
    // Legs
    draw_rectangle(x + 4.0, y + 22.0, 4.0, 8.0, Color::from_hex(0x6b3a2a));
    draw_rectangle(x + TS - 8.0, y + 22.0, 4.0, 8.0, Color::from_hex(0x6b3a2a));
    // Armrests
    draw_rectangle(x + 2.0, y + 12.0, 6.0, 3.0, Color::from_hex(0x8b5e3c));
    draw_rectangle(x + TS - 8.0, y + 12.0, 6.0, 3.0, Color::from_hex(0x8b5e3c));
}

fn draw_oak_tree(x: f32, y: f32, col: usize, row: usize, has_acorns: bool) {
    // Grass base
    draw_grass(x, y, col, row);
    // Trunk — thicker, taller
    draw_rectangle(x + 10.0, y + 12.0, 12.0, 22.0, Color::from_hex(0x6b4226));
    // Bark detail
    draw_line(x + 13.0, y + 14.0, x + 13.0, y + 32.0, 1.0, Color::from_hex(0x543218));
    draw_line(x + 18.0, y + 16.0, x + 18.0, y + 30.0, 1.0, Color::from_hex(0x543218));
    // Roots
    draw_line(x + 8.0, y + 32.0, x + 10.0, y + 30.0, 2.0, Color::from_hex(0x5a3a1a));
    draw_line(x + 22.0, y + 30.0, x + 24.0, y + 32.0, 2.0, Color::from_hex(0x5a3a1a));
    // Canopy — bigger, fuller crown that overflows the tile
    let canopy = Color::from_hex(0x2d7a2d);
    let canopy_light = Color::from_hex(0x3d9a3d);
    let canopy_dark = Color::from_hex(0x1e6a1e);
    draw_circle(x + 16.0, y + 4.0, 16.0, canopy);
    draw_circle(x + 6.0, y + 10.0, 12.0, canopy);
    draw_circle(x + 26.0, y + 10.0, 12.0, canopy);
    draw_circle(x + 16.0, y - 2.0, 10.0, canopy_light);
    draw_circle(x + 8.0, y + 16.0, 8.0, canopy_dark);
    draw_circle(x + 24.0, y + 16.0, 8.0, canopy_dark);
    // Acorn indicators (small brown dots) when harvestable
    if has_acorns {
        draw_circle(x + 5.0,  y + 18.0, 2.5, Color::from_hex(0x8b5e3c));
        draw_circle(x + 27.0, y + 17.0, 2.5, Color::from_hex(0x8b5e3c));
        draw_circle(x + 16.0, y + 22.0, 2.0, Color::from_hex(0x8b5e3c));
        // Tiny acorn caps
        draw_circle(x + 5.0,  y + 17.0, 1.5, Color::from_hex(0x5a3a1a));
        draw_circle(x + 27.0, y + 16.0, 1.5, Color::from_hex(0x5a3a1a));
        draw_circle(x + 16.0, y + 21.0, 1.2, Color::from_hex(0x5a3a1a));
    }
}

fn draw_fishing_spot(x: f32, y: f32) {
    // Water base (same as regular water)
    draw_rectangle(x, y, TS, TS, Color::from_hex(0x1a6faa));
    // Wave shimmer
    draw_rectangle(x + 3.0,  y + 8.0,  14.0, 2.0, Color::from_hex(0x4a9fdf));
    draw_rectangle(x + 18.0, y + 18.0, 9.0,  2.0, Color::from_hex(0x4a9fdf));
    // Concentric ripple rings — fishing float indicator
    draw_circle_lines(x + 16.0, y + 16.0, 9.0, 1.5, Color { r: 0.7, g: 0.9, b: 1.0, a: 0.55 });
    draw_circle_lines(x + 16.0, y + 16.0, 5.5, 1.5, Color { r: 0.7, g: 0.9, b: 1.0, a: 0.75 });
    // Float (red bob on water)
    draw_circle(x + 16.0, y + 16.0, 3.5, Color::from_hex(0xe74c3c));
    draw_circle(x + 16.0, y + 14.0, 2.0, Color::from_hex(0xf5f5f5)); // white top of float
    // Fishing line going up toward the bank
    draw_line(x + 16.0, y + 12.0, x + 16.0, y + 2.0, 1.0, Color::from_hex(0xdddddd));
}

fn draw_rock(x: f32, y: f32, hp: u8) {
    // Grass behind rock
    draw_rectangle(x, y, TS, TS, Color::from_hex(0x5a8a3c));
    let (rock_col, crack_col) = match hp {
        3 => (Color::from_hex(0x888888), Color::from_hex(0x555555)),
        2 => (Color::from_hex(0x999966), Color::from_hex(0x666633)),
        _ => (Color::from_hex(0xbbbb99), Color::from_hex(0x888866)),
    };
    // Rock body — smaller, more pebble-like
    draw_circle(x + 16.0, y + 20.0, 8.0, rock_col);
    draw_circle(x + 11.0, y + 22.0, 5.0, rock_col);
    draw_circle(x + 21.0, y + 22.0, 5.0, rock_col);
    // Highlight
    draw_circle(x + 14.0, y + 18.0, 3.0, Color { r: 1.0, g: 1.0, b: 1.0, a: 0.2 });
    // Crack lines for damaged rocks
    if hp < 3 {
        draw_line(x + 14.0, y + 16.0, x + 18.0, y + 24.0, 1.5, crack_col);
        if hp < 2 {
            draw_line(x + 11.0, y + 20.0, x + 21.0, y + 19.0, 1.5, crack_col);
        }
    }
}

// ── Crop rendering ────────────────────────────────────────────────────────────

fn draw_crop(crop: &CropState, x: f32, y: f32) {
    let (stem, produce) = crop_colors(&crop.kind);
    let days = crop.days_grown;

    // Watered tint on soil
    if crop.watered_today {
        draw_rectangle(x, y + TS - 6.0, TS, 6.0,
                       Color { r: 0.3, g: 0.5, b: 0.9, a: 0.2 });
    }

    if days == 0 {
        // Seed — small mound
        draw_circle(x + 16.0, y + 22.0, 3.5, Color::from_hex(0x9b7040));
    } else if days == 1 {
        // Sprout — tiny stem + two leaves
        draw_line(x + 16.0, y + 24.0, x + 16.0, y + 16.0, 2.0, stem);
        draw_circle(x + 12.0, y + 18.0, 4.0, stem);
        draw_circle(x + 20.0, y + 17.0, 4.0, stem);
    } else if days <= 3 {
        // Growing — taller stem + bigger leaves
        draw_line(x + 16.0, y + 26.0, x + 16.0, y + 10.0, 2.5, stem);
        draw_circle(x + 10.0, y + 16.0, 5.0, stem);
        draw_circle(x + 22.0, y + 14.0, 5.0, stem);
        draw_circle(x + 14.0, y + 11.0, 4.0, stem);
    } else {
        // Mature — full plant with produce color on top
        draw_line(x + 16.0, y + 28.0, x + 16.0, y + 8.0, 3.0, stem);
        draw_circle(x + 9.0,  y + 18.0, 6.0, stem);
        draw_circle(x + 23.0, y + 16.0, 6.0, stem);
        draw_circle(x + 14.0, y + 11.0, 5.0, stem);
        // Produce
        draw_circle(x + 16.0, y + 8.0, 6.0, produce);
        // Shine on produce
        draw_circle(x + 14.0, y + 6.0, 2.0, Color { r: 1.0, g: 1.0, b: 1.0, a: 0.4 });
    }
}

fn crop_colors(kind: &CropKind) -> (Color, Color) {
    match kind {
        CropKind::Parsnip    => (Color::from_hex(0x4a8a2a), Color::from_hex(0xf0e060)),
        CropKind::Potato     => (Color::from_hex(0x5a8a3a), Color::from_hex(0xc8a050)),
        CropKind::Cauliflower => (Color::from_hex(0x5a9a3a), Color::from_hex(0xf0f0e0)),
        CropKind::Melon      => (Color::from_hex(0x4a8a2a), Color::from_hex(0xa8d060)),
        CropKind::Blueberry  => (Color::from_hex(0x4a7a3a), Color::from_hex(0x6060d0)),
        CropKind::Tomato     => (Color::from_hex(0x4a9a2a), Color::from_hex(0xe03020)),
        CropKind::Pumpkin    => (Color::from_hex(0x5a8a2a), Color::from_hex(0xe07010)),
        CropKind::Yam        => (Color::from_hex(0x5a7a3a), Color::from_hex(0xa05090)),
        CropKind::Cranberry  => (Color::from_hex(0x4a8a3a), Color::from_hex(0xc02020)),
    }
}

// ── Building overlays ─────────────────────────────────────────────────────────

fn draw_farmhouse(x: f32, y: f32, upgraded: bool) {
    let w = if upgraded { 5.0 * TS } else { 3.0 * TS };
    let h = if upgraded { 4.0 * TS } else { 3.0 * TS };

    // Foundation shadow
    draw_rectangle(x + 4.0, y + h - 4.0, w, 8.0,
                   Color { r: 0.0, g: 0.0, b: 0.0, a: 0.18 });

    // Walls — cream
    let wall = Color::from_hex(0xf5e6c8);
    draw_rectangle(x, y + 24.0, w, h - 24.0, wall);

    // Horizontal siding lines
    let siding = Color::from_hex(0xe0c898);
    for i in 0..5u32 {
        draw_line(x, y + 24.0 + i as f32 * 14.0,
                  x + w, y + 24.0 + i as f32 * 14.0,
                  1.0, siding);
    }

    // Roof — dark red tiles, stepped
    let roof_dark = Color::from_hex(0x8b2020);
    let roof_mid  = Color::from_hex(0xa83030);
    draw_rectangle(x - 6.0,  y + 14.0, w + 12.0, 12.0, roof_dark);
    draw_rectangle(x + 6.0,  y + 7.0,  w - 12.0, 10.0, roof_mid);
    draw_rectangle(x + 16.0, y + 2.0,  w - 32.0, 8.0,  roof_dark);
    // Roof edge overhang shadow
    draw_rectangle(x - 6.0,  y + 24.0, w + 12.0, 4.0,
                   Color { r: 0.0, g: 0.0, b: 0.0, a: 0.25 });

    // Chimney (top-right)
    draw_rectangle(x + w - 22.0, y - 10.0, 14.0, 26.0, Color::from_hex(0xa0522d));
    // Chimney top cap
    draw_rectangle(x + w - 24.0, y - 12.0, 18.0, 4.0, Color::from_hex(0x8b3a1c));
    // Smoke puffs
    draw_circle(x + w - 15.0, y - 18.0, 5.0, Color { r: 0.85, g: 0.85, b: 0.85, a: 0.55 });
    draw_circle(x + w - 10.0, y - 26.0, 4.0, Color { r: 0.85, g: 0.85, b: 0.85, a: 0.35 });

    // Left window
    draw_window(x + 6.0, y + 30.0);
    // Right window
    draw_window(x + w - 26.0, y + 30.0);

    // Door
    let door_x = x + w / 2.0 - 11.0;
    let door_y = y + h - 30.0;
    draw_rectangle(door_x, door_y, 22.0, 30.0, Color::from_hex(0x8b5e3c));
    draw_rectangle_lines(door_x, door_y, 22.0, 30.0, 2.0, Color::from_hex(0x6b3a1c));
    // Door panels
    draw_rectangle(door_x + 3.0, door_y + 3.0, 7.0, 10.0, Color::from_hex(0x7a5030));
    draw_rectangle(door_x + 12.0, door_y + 3.0, 7.0, 10.0, Color::from_hex(0x7a5030));
    // Door knob
    draw_circle(door_x + 17.0, door_y + 18.0, 2.5, Color::from_hex(0xf1c40f));

    // "HOME" flower box under windows
    let flower_box = Color::from_hex(0x8b5e3c);
    draw_rectangle(x + 3.0, y + 49.0, 22.0, 6.0, flower_box);
    draw_rectangle(x + w - 26.0, y + 49.0, 22.0, 6.0, flower_box);
    // Flowers
    for (bx, by, fc) in &[
        (x + 6.0, y + 46.0, Color::from_hex(0xe74c3c)),
        (x + 12.0, y + 44.0, Color::from_hex(0xf39c12)),
        (x + 18.0, y + 46.0, Color::from_hex(0x9b59b6)),
        (x + w - 23.0, y + 46.0, Color::from_hex(0xe74c3c)),
        (x + w - 17.0, y + 44.0, Color::from_hex(0xf1c40f)),
        (x + w - 11.0, y + 46.0, Color::from_hex(0x2ecc71)),
    ] {
        draw_circle(*bx, *by, 3.0, *fc);
    }
}

fn draw_window(x: f32, y: f32) {
    // Frame
    draw_rectangle(x, y, 20.0, 16.0, Color::from_hex(0xd4b896));
    // Glass (sky blue tint)
    draw_rectangle(x + 2.0, y + 2.0, 7.0, 12.0, Color::from_hex(0xa8d8f0));
    draw_rectangle(x + 11.0, y + 2.0, 7.0, 12.0, Color::from_hex(0xa8d8f0));
    // Divider
    draw_line(x + 10.0, y, x + 10.0, y + 16.0, 1.5, Color::from_hex(0xd4b896));
    draw_line(x, y + 8.0, x + 20.0, y + 8.0, 1.5, Color::from_hex(0xd4b896));
    // Shine
    draw_rectangle(x + 3.0, y + 3.0, 3.0, 4.0, Color { r: 1.0, g: 1.0, b: 1.0, a: 0.35 });
}

fn draw_shop(x: f32, y: f32) {
    let w = 4.0 * TS; // 128
    let h = 3.0 * TS; // 96

    // Foundation shadow
    draw_rectangle(x + 4.0, y + h - 4.0, w, 8.0,
                   Color { r: 0.0, g: 0.0, b: 0.0, a: 0.18 });

    // Walls — warm off-white
    draw_rectangle(x, y + 20.0, w, h - 20.0, Color::from_hex(0xfaf0e0));
    // Siding
    let siding = Color::from_hex(0xe0d4b0);
    for i in 0..5u32 {
        draw_line(x, y + 20.0 + i as f32 * 14.0,
                  x + w, y + 20.0 + i as f32 * 14.0, 1.0, siding);
    }

    // Roof — teal/dark blue
    draw_rectangle(x - 4.0,  y + 12.0, w + 8.0, 10.0, Color::from_hex(0x2c5f6e));
    draw_rectangle(x + 8.0,  y + 5.0,  w - 16.0, 9.0, Color::from_hex(0x3a7a8e));
    draw_rectangle(x + 18.0, y,         w - 36.0, 7.0, Color::from_hex(0x2c5f6e));
    // Overhang shadow
    draw_rectangle(x - 4.0, y + 20.0, w + 8.0, 4.0,
                   Color { r: 0.0, g: 0.0, b: 0.0, a: 0.22 });

    // Awning — striped
    let aw_y = y + 28.0;
    let aw_h = 12.0;
    draw_rectangle(x, aw_y, w, aw_h, Color::from_hex(0x2c5f6e));
    for i in 0..8u32 {
        if i % 2 == 0 {
            draw_rectangle(x + i as f32 * 16.0, aw_y, 16.0, aw_h, Color::from_hex(0x4a9faf));
        }
    }
    // Awning fringe
    for i in 0..10u32 {
        draw_rectangle(x + i as f32 * 13.0 + 2.0, aw_y + aw_h, 5.0, 5.0,
                       Color::from_hex(0x2c5f6e));
    }

    // Shop sign
    let sign_x = x + w / 2.0 - 30.0;
    draw_rectangle(sign_x, y + 7.0, 60.0, 14.0, Color::from_hex(0xf1c40f));
    draw_rectangle_lines(sign_x, y + 7.0, 60.0, 14.0, 2.0, Color::from_hex(0xb8860b));
    draw_text("SHOP", sign_x + 10.0, y + 19.0, 14.0, Color::from_hex(0x2c3e50));

    // Large display window
    draw_rectangle(x + 6.0, y + 44.0, w - 12.0, 28.0, Color::from_hex(0xa8d8f0));
    draw_rectangle_lines(x + 6.0, y + 44.0, w - 12.0, 28.0, 3.0, Color::from_hex(0xd4b896));
    // Window shine
    draw_rectangle(x + 8.0, y + 46.0, 14.0, 8.0,
                   Color { r: 1.0, g: 1.0, b: 1.0, a: 0.3 });
    // Window dividers
    draw_line(x + w / 2.0, y + 44.0, x + w / 2.0, y + 72.0, 2.0, Color::from_hex(0xd4b896));
    draw_line(x + 6.0, y + 58.0, x + w - 6.0, y + 58.0, 2.0, Color::from_hex(0xd4b896));

    // Door (right side)
    let door_x = x + w - 28.0;
    let door_y = y + h - 28.0;
    draw_rectangle(door_x, door_y, 22.0, 28.0, Color::from_hex(0x8b5e3c));
    draw_rectangle_lines(door_x, door_y, 22.0, 28.0, 2.0, Color::from_hex(0x6b3a1c));
    draw_circle(door_x + 5.0, door_y + 14.0, 2.5, Color::from_hex(0xf1c40f));
}

// ── Town buildings ────────────────────────────────────────────────────────────

/// Inn — warm-lit building with a hanging sign, 7×4 tiles
fn draw_inn(x: f32, y: f32) {
    let w = 7.0 * TS;
    let h = 4.0 * TS;
    draw_rectangle(x + 4.0, y + h, w, 6.0, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.15 }); // shadow
    draw_rectangle(x, y + 22.0, w, h - 22.0, Color::from_hex(0xf0ddb0)); // walls — warm cream
    for i in 0..5u32 {
        draw_line(x, y + 22.0 + i as f32 * 13.0, x + w, y + 22.0 + i as f32 * 13.0,
                  1.0, Color::from_hex(0xd8c090));
    }
    // Roof — mossy green
    draw_rectangle(x - 6.0, y + 12.0, w + 12.0, 12.0, Color::from_hex(0x3a7a3a));
    draw_rectangle(x + 6.0, y + 5.0,  w - 12.0, 9.0,  Color::from_hex(0x4a9a4a));
    draw_rectangle(x + 16.0, y,        w - 32.0, 7.0,  Color::from_hex(0x3a7a3a));
    draw_rectangle(x - 6.0, y + 22.0, w + 12.0, 4.0,
                   Color { r: 0.0, g: 0.0, b: 0.0, a: 0.2 }); // overhang shadow
    // Hanging sign
    let sign_x = x + w / 2.0 - 28.0;
    draw_rectangle(sign_x, y + 28.0, 56.0, 14.0, Color::from_hex(0xc8860a));
    draw_rectangle_lines(sign_x, y + 28.0, 56.0, 14.0, 2.0, Color::from_hex(0x8b5e00));
    draw_text("INN", sign_x + 16.0, y + 40.0, 13.0, Color::from_hex(0xfff0cc));
    // Windows
    draw_window(x + 8.0, y + 48.0);
    draw_window(x + w - 28.0, y + 48.0);
    // Door
    let dx = x + w / 2.0 - 11.0;
    let dy = y + h - 28.0;
    draw_rectangle(dx, dy, 22.0, 28.0, Color::from_hex(0x7a4a28));
    draw_rectangle_lines(dx, dy, 22.0, 28.0, 2.0, Color::from_hex(0x5a2e10));
    draw_circle(dx + 17.0, dy + 14.0, 2.5, Color::from_hex(0xf1c40f));
    // Lanterns
    draw_circle(x + 8.0, y + 26.0, 5.0, Color { r: 1.0, g: 0.85, b: 0.3, a: 0.8 });
    draw_circle(x + w - 8.0, y + 26.0, 5.0, Color { r: 1.0, g: 0.85, b: 0.3, a: 0.8 });
}

/// Market — bright striped stall, 7×4 tiles
fn draw_market(x: f32, y: f32) {
    let w = 7.0 * TS;
    let h = 4.0 * TS;
    draw_rectangle(x + 4.0, y + h, w, 6.0, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.15 });
    draw_rectangle(x, y + 20.0, w, h - 20.0, Color::from_hex(0xfaf0e0));
    for i in 0..5u32 {
        draw_line(x, y + 20.0 + i as f32 * 13.0, x + w, y + 20.0 + i as f32 * 13.0,
                  1.0, Color::from_hex(0xe0d4b0));
    }
    // Roof — orange
    draw_rectangle(x - 4.0, y + 12.0, w + 8.0, 10.0, Color::from_hex(0xc85010));
    draw_rectangle(x + 8.0, y + 5.0,  w - 16.0, 9.0, Color::from_hex(0xe06020));
    draw_rectangle(x + 18.0, y,        w - 36.0, 7.0, Color::from_hex(0xc85010));
    draw_rectangle(x - 4.0, y + 20.0, w + 8.0, 4.0,
                   Color { r: 0.0, g: 0.0, b: 0.0, a: 0.2 });
    // Striped awning
    let aw_y = y + 28.0;
    draw_rectangle(x, aw_y, w, 10.0, Color::from_hex(0xe06020));
    for i in 0..8u32 {
        if i % 2 == 0 {
            draw_rectangle(x + i as f32 * 14.0, aw_y, 14.0, 10.0, Color::from_hex(0xfff0cc));
        }
    }
    let sign_x = x + w / 2.0 - 36.0;
    draw_rectangle(sign_x, y + 7.0, 72.0, 14.0, Color::from_hex(0xf1c40f));
    draw_rectangle_lines(sign_x, y + 7.0, 72.0, 14.0, 2.0, Color::from_hex(0xb8860b));
    draw_text("MARKET", sign_x + 8.0, y + 19.0, 13.0, Color::from_hex(0x2c3e50));
    draw_rectangle(x + 6.0, y + 44.0, w - 12.0, 28.0, Color::from_hex(0xa8d8f0));
    draw_rectangle_lines(x + 6.0, y + 44.0, w - 12.0, 28.0, 2.0, Color::from_hex(0xd4b896));
    let door_x = x + w - 28.0;
    let door_y = y + h - 28.0;
    draw_rectangle(door_x, door_y, 22.0, 28.0, Color::from_hex(0x8b5e3c));
    draw_rectangle_lines(door_x, door_y, 22.0, 28.0, 2.0, Color::from_hex(0x6b3a1c));
    draw_circle(door_x + 5.0, door_y + 14.0, 2.5, Color::from_hex(0xf1c40f));
}

/// Tavern — dark wood with neon-lit sign, 7×4 tiles
fn draw_tavern(x: f32, y: f32) {
    let w = 7.0 * TS;
    let h = 4.0 * TS;
    draw_rectangle(x + 4.0, y + h, w, 6.0, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.18 });
    draw_rectangle(x, y + 22.0, w, h - 22.0, Color::from_hex(0xd4a878)); // warm wood walls
    for i in 0..5u32 {
        draw_line(x, y + 22.0 + i as f32 * 13.0, x + w, y + 22.0 + i as f32 * 13.0,
                  1.0, Color::from_hex(0xb88858));
    }
    // Roof — dark
    draw_rectangle(x - 6.0, y + 12.0, w + 12.0, 12.0, Color::from_hex(0x4a3020));
    draw_rectangle(x + 6.0, y + 5.0,  w - 12.0, 9.0,  Color::from_hex(0x5a4030));
    draw_rectangle(x + 16.0, y,        w - 32.0, 7.0,  Color::from_hex(0x4a3020));
    draw_rectangle(x - 6.0, y + 22.0, w + 12.0, 4.0,
                   Color { r: 0.0, g: 0.0, b: 0.0, a: 0.28 });
    let sign_x = x + w / 2.0 - 36.0;
    draw_rectangle(sign_x, y + 28.0, 72.0, 14.0, Color::from_hex(0x1a1a2e));
    draw_rectangle_lines(sign_x, y + 28.0, 72.0, 14.0, 2.0, Color::from_hex(0xe74c3c));
    draw_text("TAVERN", sign_x + 8.0, y + 40.0, 13.0, Color::from_hex(0xe74c3c));
    draw_window(x + 8.0, y + 50.0);
    draw_window(x + w - 28.0, y + 50.0);
    let dx = x + w / 2.0 - 11.0;
    let dy = y + h - 28.0;
    draw_rectangle(dx, dy, 22.0, 28.0, Color::from_hex(0x5a3010));
    draw_rectangle_lines(dx, dy, 22.0, 28.0, 2.0, Color::from_hex(0x3a1800));
    draw_circle(dx + 17.0, dy + 14.0, 2.5, Color::from_hex(0xf1c40f));
    // Warm window glow
    draw_rectangle(x + 10.0, y + 52.0, 16.0, 10.0,
                   Color { r: 1.0, g: 0.75, b: 0.2, a: 0.3 });
    draw_rectangle(x + w - 26.0, y + 52.0, 16.0, 10.0,
                   Color { r: 1.0, g: 0.75, b: 0.2, a: 0.3 });
}

/// Clinic — clean white building with a red cross, 7×4 tiles
fn draw_clinic(x: f32, y: f32) {
    let w = 7.0 * TS;
    let h = 4.0 * TS;
    draw_rectangle(x + 4.0, y + h, w, 6.0, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.12 });
    draw_rectangle(x, y + 22.0, w, h - 22.0, Color::from_hex(0xf8f8f0)); // white walls
    for i in 0..5u32 {
        draw_line(x, y + 22.0 + i as f32 * 13.0, x + w, y + 22.0 + i as f32 * 13.0,
                  1.0, Color::from_hex(0xdde8ee));
    }
    // Roof — light blue
    draw_rectangle(x - 6.0, y + 12.0, w + 12.0, 12.0, Color::from_hex(0x4a80b0));
    draw_rectangle(x + 6.0, y + 5.0,  w - 12.0, 9.0,  Color::from_hex(0x5a98cc));
    draw_rectangle(x + 16.0, y,        w - 32.0, 7.0,  Color::from_hex(0x4a80b0));
    draw_rectangle(x - 6.0, y + 22.0, w + 12.0, 4.0,
                   Color { r: 0.0, g: 0.0, b: 0.0, a: 0.15 });
    // Red cross sign
    let cx2 = x + w / 2.0;
    draw_rectangle(cx2 - 6.0, y + 27.0, 12.0, 20.0, Color::from_hex(0xe74c3c));
    draw_rectangle(cx2 - 12.0, y + 33.0, 24.0, 8.0,  Color::from_hex(0xe74c3c));
    draw_window(x + 8.0, y + 50.0);
    draw_window(x + w - 28.0, y + 50.0);
    let dx = x + w / 2.0 - 11.0;
    let dy = y + h - 28.0;
    draw_rectangle(dx, dy, 22.0, 28.0, Color::from_hex(0x8898a8));
    draw_rectangle_lines(dx, dy, 22.0, 28.0, 2.0, Color::from_hex(0x6678a0));
    draw_circle(dx + 17.0, dy + 14.0, 2.5, Color::from_hex(0xaaaacc));
}

/// Library — stone building with arched windows, 7×6 tiles
fn draw_library(x: f32, y: f32) {
    let w = 7.0 * TS;
    let h = 6.0 * TS;
    draw_rectangle(x + 4.0, y + h, w, 8.0, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.18 });
    draw_rectangle(x, y + 24.0, w, h - 24.0, Color::from_hex(0xd4c8a8)); // stone walls
    // Stone texture
    for i in 0..6u32 {
        draw_line(x, y + 24.0 + i as f32 * 18.0, x + w, y + 24.0 + i as f32 * 18.0,
                  1.0, Color::from_hex(0xb8a888));
    }
    // Flat roof with battlement trim
    draw_rectangle(x - 4.0, y + 12.0, w + 8.0, 14.0, Color::from_hex(0x9a8a6a));
    draw_rectangle(x + 6.0, y + 5.0,  w - 12.0, 10.0, Color::from_hex(0xb0a07a));
    draw_rectangle(x + 16.0, y,        w - 32.0, 7.0, Color::from_hex(0x9a8a6a));
    // Battlement notches
    for i in 0..5u32 {
        draw_rectangle(x + 6.0 + i as f32 * 26.0, y - 6.0, 14.0, 10.0, Color::from_hex(0xb0a07a));
    }
    draw_rectangle(x - 4.0, y + 24.0, w + 8.0, 4.0, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.22 });
    // Library sign
    let sign_x = x + w / 2.0 - 40.0;
    draw_rectangle(sign_x, y + 16.0, 80.0, 14.0, Color::from_hex(0x8b6914));
    draw_rectangle_lines(sign_x, y + 16.0, 80.0, 14.0, 2.0, Color::from_hex(0x5a3a00));
    draw_text("LIBRARY", sign_x + 8.0, y + 28.0, 13.0, Color::from_hex(0xf5e6c8));
    // Arched windows
    for &wx in &[x + 8.0, x + w / 2.0 - 10.0, x + w - 28.0] {
        draw_rectangle(wx, y + 50.0, 20.0, 20.0, Color::from_hex(0xa8d8f0));
        draw_circle(wx + 10.0, y + 50.0, 10.0, Color::from_hex(0xa8d8f0));
        draw_rectangle_lines(wx, y + 50.0, 20.0, 20.0, 2.0, Color::from_hex(0xd4c8a8));
    }
    // Door
    let dx = x + w / 2.0 - 13.0;
    let dy = y + h - 32.0;
    draw_rectangle(dx, dy, 26.0, 32.0, Color::from_hex(0x7a5a30));
    draw_circle(dx + 13.0, dy, 13.0, Color::from_hex(0x7a5a30)); // arched top
    draw_rectangle_lines(dx, dy, 26.0, 32.0, 2.0, Color::from_hex(0x5a3a10));
    draw_circle(dx + 20.0, dy + 18.0, 2.5, Color::from_hex(0xf1c40f));
}

/// Town Hall — grand building with a clock tower, 11×6 tiles
fn draw_town_hall(x: f32, y: f32) {
    let w = 11.0 * TS;
    let h = 6.0 * TS;
    draw_rectangle(x + 6.0, y + h, w, 10.0, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.2 });
    draw_rectangle(x, y + 28.0, w, h - 28.0, Color::from_hex(0xe8dcc8)); // cream stone
    for i in 0..5u32 {
        draw_line(x, y + 28.0 + i as f32 * 19.0, x + w, y + 28.0 + i as f32 * 19.0,
                  1.0, Color::from_hex(0xd0c0a0));
    }
    // Main roof — slate grey
    draw_rectangle(x - 8.0, y + 14.0, w + 16.0, 16.0, Color::from_hex(0x5a5a6a));
    draw_rectangle(x + 8.0, y + 5.0,  w - 16.0, 11.0, Color::from_hex(0x6a6a7a));
    draw_rectangle(x + 20.0, y,        w - 40.0, 7.0,  Color::from_hex(0x5a5a6a));
    draw_rectangle(x - 8.0, y + 28.0, w + 16.0, 5.0, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.25 });
    // Central clock tower
    let tower_x = x + w / 2.0 - 20.0;
    draw_rectangle(tower_x, y - 28.0, 40.0, 44.0, Color::from_hex(0xd0c0a0));
    draw_rectangle_lines(tower_x, y - 28.0, 40.0, 44.0, 2.0, Color::from_hex(0xa09070));
    // Clock face
    draw_circle(x + w / 2.0, y - 6.0, 14.0, Color::from_hex(0xf5f0e8));
    draw_circle_lines(x + w / 2.0, y - 6.0, 14.0, 2.0, Color::from_hex(0x4a3a20));
    // Clock hands
    draw_line(x + w / 2.0, y - 6.0, x + w / 2.0, y - 17.0, 2.0, Color::from_hex(0x2a1a00));
    draw_line(x + w / 2.0, y - 6.0, x + w / 2.0 + 8.0, y - 6.0, 2.0, Color::from_hex(0x2a1a00));
    // Tower spire
    draw_triangle(
        Vec2::new(tower_x, y - 28.0),
        Vec2::new(tower_x + 40.0, y - 28.0),
        Vec2::new(x + w / 2.0, y - 52.0),
        Color::from_hex(0x4a4a5a),
    );
    // Columns (decorative)
    for &cx3 in &[x + 14.0, x + w - 22.0] {
        draw_rectangle(cx3, y + 28.0, 8.0, h - 28.0, Color::from_hex(0xf0e8d0));
        draw_circle(cx3 + 4.0, y + 28.0, 6.0, Color::from_hex(0xf0e8d0));
    }
    // Banner
    draw_rectangle(x + w / 2.0 - 18.0, y + 36.0, 36.0, 18.0, Color::from_hex(0x2255aa));
    draw_rectangle_lines(x + w / 2.0 - 18.0, y + 36.0, 36.0, 18.0, 2.0, Color::from_hex(0x1a3a80));
    // Star on banner
    draw_circle(x + w / 2.0, y + 45.0, 5.0, Color::from_hex(0xf1c40f));
    // Main door (double)
    let dx = x + w / 2.0 - 18.0;
    let dy = y + h - 34.0;
    draw_rectangle(dx, dy, 36.0, 34.0, Color::from_hex(0x7a5830));
    draw_line(dx + 18.0, dy, dx + 18.0, dy + 34.0, 2.0, Color::from_hex(0x5a3810));
    draw_rectangle_lines(dx, dy, 36.0, 34.0, 2.0, Color::from_hex(0x5a3810));
    draw_circle(dx + 12.0, dy + 18.0, 2.5, Color::from_hex(0xf1c40f));
    draw_circle(dx + 24.0, dy + 18.0, 2.5, Color::from_hex(0xf1c40f));
    // Side windows
    for &wx in &[x + 28.0, x + w - 48.0] {
        draw_window(wx, y + 70.0);
        draw_window(wx, y + 110.0);
    }
}

/// Street lamp — tall pole with lantern, glows at night
fn draw_street_lamp(x: f32, y: f32, is_night: bool) {
    let cx = x + TS / 2.0;
    // Pole
    draw_rectangle(cx - 2.0, y + 4.0, 4.0, TS - 4.0, Color::from_hex(0x555555));
    // Lamp housing
    draw_rectangle(cx - 6.0, y, 12.0, 8.0, Color::from_hex(0x444444));
    draw_rectangle(cx - 5.0, y + 1.0, 10.0, 6.0, Color::from_hex(0x666666));

    if is_night {
        // Warm glow
        draw_circle(cx, y + 4.0, 16.0, Color { r: 1.0, g: 0.85, b: 0.4, a: 0.12 });
        draw_circle(cx, y + 4.0, 10.0, Color { r: 1.0, g: 0.9, b: 0.5, a: 0.18 });
        // Lit lantern
        draw_rectangle(cx - 4.0, y + 1.0, 8.0, 5.0, Color::from_hex(0xffe066));
        draw_circle(cx, y + 3.5, 3.0, Color { r: 1.0, g: 0.95, b: 0.7, a: 0.9 });
    }
}

/// Small NPC cottage — 3×2 tile house with colored roof
fn draw_npc_cottage(x: f32, y: f32, roof_color: Color) {
    let w = 3.0 * TS;
    let h = 2.0 * TS;
    // Shadow
    draw_rectangle(x + 3.0, y + h, w, 4.0, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.15 });
    // Walls
    draw_rectangle(x, y + 16.0, w, h - 16.0, Color::from_hex(0xf0e8d0));
    // Siding lines
    for i in 0..3 {
        draw_line(x, y + 20.0 + i as f32 * 12.0, x + w, y + 20.0 + i as f32 * 12.0,
                  1.0, Color::from_hex(0xd8d0b8));
    }
    // Roof — triangular
    draw_triangle(
        Vec2::new(x - 4.0, y + 18.0),
        Vec2::new(x + w + 4.0, y + 18.0),
        Vec2::new(x + w / 2.0, y - 2.0),
        roof_color,
    );
    // Lighter roof highlight
    let lighter = Color { r: roof_color.r + 0.1, g: roof_color.g + 0.1, b: roof_color.b + 0.1, a: 1.0 };
    draw_triangle(
        Vec2::new(x + 4.0, y + 18.0),
        Vec2::new(x + w / 2.0, y + 18.0),
        Vec2::new(x + w / 2.0, y + 2.0),
        lighter,
    );
    // Chimney
    draw_rectangle(x + w - 22.0, y - 4.0, 10.0, 14.0, Color::from_hex(0x8a7a6a));
    draw_rectangle(x + w - 24.0, y - 6.0, 14.0, 4.0, Color::from_hex(0x7a6a5a));
    // Door
    let dx = x + w / 2.0 - 8.0;
    draw_rectangle(dx, y + h - 22.0, 16.0, 22.0, Color::from_hex(0x7a5030));
    draw_circle(dx + 12.0, y + h - 10.0, 2.0, Color::from_hex(0xf1c40f));
    // Window
    draw_rectangle(x + 6.0, y + 24.0, 16.0, 14.0, Color::from_hex(0xa8d8f0));
    draw_rectangle_lines(x + 6.0, y + 24.0, 16.0, 14.0, 1.5, Color::from_hex(0xd4b896));
    draw_line(x + 14.0, y + 24.0, x + 14.0, y + 38.0, 1.0, Color::from_hex(0xd4b896));
    // Flower box under window
    draw_rectangle(x + 4.0, y + 38.0, 20.0, 4.0, Color::from_hex(0x8b5e3c));
    draw_circle(x + 9.0, y + 37.0, 3.0, Color::from_hex(0xe74c3c));
    draw_circle(x + 14.0, y + 36.0, 3.0, Color::from_hex(0xf1c40f));
    draw_circle(x + 19.0, y + 37.0, 3.0, Color::from_hex(0xe74c3c));
}

/// Italian Restaurant — warm terracotta building, 7×3 tiles
fn draw_restaurant(x: f32, y: f32) {
    let w = 7.0 * TS;
    let h = 3.0 * TS;
    // Shadow
    draw_rectangle(x + 6.0, y + h, w, 10.0, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.2 });
    // Walls — warm terracotta/cream
    draw_rectangle(x, y + 24.0, w, h - 24.0, Color::from_hex(0xf5e0c0));
    for i in 0..6 {
        draw_line(x, y + 28.0 + i as f32 * 16.0, x + w, y + 28.0 + i as f32 * 16.0, 1.0, Color::from_hex(0xe8d0a8));
    }
    // Roof — red tile
    draw_rectangle(x - 6.0, y + 14.0, w + 12.0, 12.0, Color::from_hex(0xc04020));
    draw_rectangle(x + 4.0, y + 6.0, w - 8.0, 10.0, Color::from_hex(0xd05030));
    draw_rectangle(x + 14.0, y, w - 28.0, 8.0, Color::from_hex(0xc04020));
    // Overhang shadow
    draw_rectangle(x - 6.0, y + 24.0, w + 12.0, 4.0, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.2 });
    // Sign — Italian flag colors
    let sign_x = x + w / 2.0 - 48.0;
    draw_rectangle(sign_x, y + 6.0, 96.0, 16.0, Color::from_hex(0xf5f0e0));
    draw_rectangle_lines(sign_x, y + 6.0, 96.0, 16.0, 2.0, Color::from_hex(0x8b5e3c));
    // Italian flag stripe
    draw_rectangle(sign_x + 2.0, y + 8.0, 10.0, 12.0, Color::from_hex(0x009246)); // green
    draw_rectangle(sign_x + 12.0, y + 8.0, 10.0, 12.0, WHITE);
    draw_rectangle(sign_x + 22.0, y + 8.0, 10.0, 12.0, Color::from_hex(0xce2b37)); // red
    draw_text("RISTORANTE", sign_x + 36.0, y + 19.0, 11.0, Color::from_hex(0x6a3a2a));
    // Awning — green and white stripes
    let aw_y = y + 30.0;
    draw_rectangle(x - 4.0, aw_y, w + 8.0, 12.0, Color::from_hex(0x009246));
    for i in 0..9 {
        if i % 2 == 0 {
            draw_rectangle(x - 4.0 + i as f32 * ((w + 8.0) / 9.0), aw_y, (w + 8.0) / 9.0, 12.0, WHITE);
        }
    }
    // Windows
    for &wx in &[x + 10.0, x + w - 38.0] {
        draw_rectangle(wx, y + 48.0, 28.0, 24.0, Color::from_hex(0xa8d8f0));
        draw_rectangle_lines(wx, y + 48.0, 28.0, 24.0, 2.0, Color::from_hex(0x8b5e3c));
        draw_line(wx + 14.0, y + 48.0, wx + 14.0, y + 72.0, 1.0, Color::from_hex(0x8b5e3c));
        // Flower box
        draw_rectangle(wx - 2.0, y + 72.0, 32.0, 5.0, Color::from_hex(0x8b5e3c));
        draw_circle(wx + 6.0, y + 71.0, 3.0, Color::from_hex(0xe74c3c));
        draw_circle(wx + 14.0, y + 70.0, 3.0, Color::from_hex(0xf1c40f));
        draw_circle(wx + 22.0, y + 71.0, 3.0, Color::from_hex(0xe74c3c));
    }
    // Door
    let dx = x + w / 2.0 - 13.0;
    let dy = y + h - 34.0;
    draw_rectangle(dx, dy, 26.0, 34.0, Color::from_hex(0x6b3a2a));
    draw_rectangle_lines(dx, dy, 26.0, 34.0, 2.0, Color::from_hex(0x4a2a1a));
    draw_circle(dx + 20.0, dy + 18.0, 3.0, Color::from_hex(0xf1c40f));
    // Pizza icon above door
    draw_circle(dx + 13.0, dy - 10.0, 8.0, Color::from_hex(0xf1c40f));
    draw_circle(dx + 13.0, dy - 10.0, 6.0, Color::from_hex(0xe74c3c));
    draw_circle(dx + 11.0, dy - 12.0, 2.0, Color::from_hex(0xf1c40f));
    draw_circle(dx + 15.0, dy - 9.0, 2.0, Color::from_hex(0xf1c40f));
}

/// Swimming Pool — 7×5 tiles (cols 57-63, rows 21-25)
fn draw_pool(x: f32, y: f32) {
    let ts = TS;
    let w = 7.0 * ts;
    let h = 5.0 * ts;

    // Concrete deck (light gray border)
    draw_rectangle(x, y, w, h, Color::from_hex(0xccccbb));

    // Pool water (interior: cols 58-62, rows 22-24 → offset 1 tile in, 1 tile down)
    let pw = 5.0 * ts;
    let ph = 3.0 * ts;
    let px = x + ts;
    let py = y + ts;
    draw_rectangle(px, py, pw, ph, Color::from_hex(0x3498db));

    // Lighter pool bottom sheen
    draw_rectangle(px + 4.0, py + 4.0, pw - 8.0, ph - 8.0, Color::from_hex(0x5dade2));

    // Lane lines (white dashed)
    for lane in 1..5 {
        let lx = px + lane as f32 * ts;
        for dash in 0..6 {
            let dy = py + 4.0 + dash as f32 * (ph / 6.0);
            draw_rectangle(lx - 0.5, dy, 1.0, ph / 12.0, Color { r: 1.0, g: 1.0, b: 1.0, a: 0.5 });
        }
    }

    // Lane rope floats (colored dots along lane lines)
    for lane in 1..5 {
        let lx = px + lane as f32 * ts;
        for dot in 0..4 {
            let dy = py + 8.0 + dot as f32 * (ph / 4.0);
            let color = if dot % 2 == 0 { Color::from_hex(0xe74c3c) } else { Color::from_hex(0xf1c40f) };
            draw_circle(lx, dy, 1.5, color);
        }
    }

    // Diving board (right side, extending over pool)
    let dbx = x + w - ts * 0.3;
    let dby = py + ph * 0.4;
    draw_rectangle(dbx - 2.0, dby, ts * 0.8, 4.0, Color::from_hex(0xeeeeee)); // board
    draw_rectangle(dbx - 2.0, dby + 4.0, 3.0, 6.0, Color::from_hex(0x999999)); // support

    // Ladder (left side)
    let lax = x + ts * 0.2;
    let lay = py + ph * 0.3;
    draw_rectangle(lax, lay, 2.0, 12.0, Color::from_hex(0xaaaaaa)); // left rail
    draw_rectangle(lax + 5.0, lay, 2.0, 12.0, Color::from_hex(0xaaaaaa)); // right rail
    for rung in 0..3 {
        draw_rectangle(lax, lay + 2.0 + rung as f32 * 4.0, 7.0, 1.5, Color::from_hex(0xbbbbbb));
    }

    // "POOL" text on north deck
    draw_text("POOL", x + w * 0.35, y + ts * 0.7, 12.0, Color::from_hex(0x2980b9));
}

/// Beach decorations — umbrellas, towels, lifeguard chair, surfboard, shells
fn draw_beach(camera: &Camera) {
    let ts = TS;

    // Beach umbrella helper
    let draw_umbrella = |col: usize, row: usize, color1: u32, color2: u32| {
        let (x, y) = camera.world_to_screen(col, row);
        // Pole
        draw_rectangle(x + ts * 0.45, y + 2.0, 2.0, ts * 0.8, Color::from_hex(0x8b7355));
        // Canopy — alternating colored triangles
        let cx = x + ts * 0.46;
        let cy = y - 2.0;
        let r = ts * 0.6;
        draw_circle(cx, cy, r, Color::from_hex(color1));
        // Stripes
        for i in 0..4 {
            let angle = i as f32 * std::f32::consts::PI / 2.0;
            let sx = cx + angle.cos() * r * 0.3;
            let sy = cy + angle.sin() * r * 0.3;
            draw_circle(sx, sy, r * 0.35, Color::from_hex(color2));
        }
    };

    // Beach towel helper
    let draw_towel = |col: usize, row: usize, color: u32| {
        let (x, y) = camera.world_to_screen(col, row);
        draw_rectangle(x + 2.0, y + ts * 0.3, ts * 0.8, ts * 0.5, Color::from_hex(color));
        // Fringe
        draw_rectangle(x + 2.0, y + ts * 0.3, ts * 0.8, 2.0, Color::from_hex(0xffffff));
        draw_rectangle(x + 2.0, y + ts * 0.75, ts * 0.8, 2.0, Color::from_hex(0xffffff));
    };

    // Umbrellas
    draw_umbrella(88, 59, 0xe74c3c, 0xffffff);  // red/white
    draw_umbrella(96, 60, 0x3498db, 0xf1c40f);  // blue/yellow
    draw_umbrella(104, 59, 0x2ecc71, 0xffffff); // green/white
    draw_umbrella(110, 60, 0xff69b4, 0xffffff); // pink/white

    // Towels
    draw_towel(89, 60, 0xff6347);  // coral
    draw_towel(97, 61, 0x4169e1);  // royal blue
    draw_towel(105, 60, 0xffd700); // gold
    draw_towel(111, 61, 0xff1493); // deep pink

    // Lifeguard chair (col 100, row 58)
    {
        let (x, y) = camera.world_to_screen(100, 58);
        // Legs
        draw_rectangle(x + 4.0, y + 4.0, 3.0, ts - 4.0, Color::from_hex(0xdeb887));
        draw_rectangle(x + ts - 8.0, y + 4.0, 3.0, ts - 4.0, Color::from_hex(0xdeb887));
        // Cross brace
        draw_line(x + 5.0, y + ts * 0.7, x + ts - 7.0, y + ts * 0.4, 2.0, Color::from_hex(0xdeb887));
        // Seat platform
        draw_rectangle(x + 1.0, y + 2.0, ts - 3.0, 5.0, Color::from_hex(0xc19a6b));
        // Back
        draw_rectangle(x + ts - 8.0, y - 6.0, 3.0, 10.0, Color::from_hex(0xc19a6b));
        // Flag
        draw_rectangle(x + ts - 6.0, y - 12.0, 8.0, 6.0, Color::from_hex(0xe74c3c));
    }

    // Surfboard (col 84, row 60)
    {
        let (x, y) = camera.world_to_screen(84, 60);
        // Board leaning at angle
        draw_rectangle(x + 4.0, y - 4.0, 5.0, ts + 4.0, Color::from_hex(0x00bfff));
        // Stripe
        draw_rectangle(x + 4.0, y + ts * 0.3, 5.0, 4.0, Color::from_hex(0xffffff));
        // Fin
        draw_triangle(
            Vec2::new(x + 6.5, y + ts - 1.0),
            Vec2::new(x + 3.0, y + ts + 3.0),
            Vec2::new(x + 10.0, y + ts + 3.0),
            Color::from_hex(0x0088cc),
        );
    }

    // Sandcastle (col 93, row 62)
    {
        let (x, y) = camera.world_to_screen(93, 62);
        // Base mound
        draw_rectangle(x + 4.0, y + ts * 0.4, ts * 0.6, ts * 0.5, Color::from_hex(0xf4d03f));
        // Towers
        draw_rectangle(x + 5.0, y + ts * 0.15, 5.0, ts * 0.35, Color::from_hex(0xf0c040));
        draw_rectangle(x + ts * 0.45, y + ts * 0.2, 5.0, ts * 0.3, Color::from_hex(0xf0c040));
        // Battlement notches
        draw_rectangle(x + 6.0, y + ts * 0.12, 2.0, 3.0, Color::from_hex(0xe8b830));
        draw_rectangle(x + ts * 0.47, y + ts * 0.17, 2.0, 3.0, Color::from_hex(0xe8b830));
        // Flag
        draw_line(x + 7.0, y + ts * 0.12, x + 7.0, y, 1.0, Color::from_hex(0x8b7355));
        draw_triangle(
            Vec2::new(x + 7.0, y),
            Vec2::new(x + 7.0, y + 5.0),
            Vec2::new(x + 13.0, y + 2.5),
            Color::from_hex(0xe74c3c),
        );
    }

    // Shells scattered on sand
    let shell_spots: &[(usize, usize)] = &[
        (87, 61), (91, 63), (99, 62), (103, 63), (108, 62), (113, 63),
        (86, 63), (95, 63), (101, 61), (107, 61),
    ];
    for &(col, row) in shell_spots {
        let (x, y) = camera.world_to_screen(col, row);
        let h = (col * 7 + row * 13) % 4;
        let color = match h {
            0 => Color::from_hex(0xfff5ee), // seashell white
            1 => Color::from_hex(0xffc0cb), // pink shell
            2 => Color::from_hex(0xf5deb3), // wheat/tan shell
            _ => Color::from_hex(0xffe4c4), // bisque
        };
        draw_circle(x + ts * 0.3, y + ts * 0.6, 2.5, color);
        draw_circle(x + ts * 0.5, y + ts * 0.5, 2.0, color);
    }

    // "BEACH" sign at entrance (col 96, row 57)
    {
        let (x, y) = camera.world_to_screen(96, 57);
        // Sign post
        draw_rectangle(x + ts * 0.4, y + 2.0, 3.0, ts - 2.0, Color::from_hex(0x8b7355));
        // Sign board
        draw_rectangle(x - 4.0, y - 2.0, ts + 8.0, 12.0, Color::from_hex(0xdeb887));
        draw_text("BEACH", x + 1.0, y + 8.0, 11.0, Color::from_hex(0x8b4513));
    }
}

/// Ice Cream Shop — cute pastel building, 5×3 tiles (cols 65-69, rows 21-23)
fn draw_icecream_shop(x: f32, y: f32) {
    let ts = TS;
    let w = 5.0 * ts;
    let h = 3.0 * ts;

    // Shadow
    draw_rectangle(x + 3.0, y + 3.0, w, h, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.2 });

    // Walls — pastel pink
    draw_rectangle(x, y, w, h, Color::from_hex(0xffb6c1));
    // White trim stripe
    draw_rectangle(x, y + h * 0.55, w, 3.0, WHITE);

    // Waffle cone roof — warm tan with scalloped edge
    let roof_h = ts * 1.2;
    draw_rectangle(x - 3.0, y - roof_h + 4.0, w + 6.0, roof_h, Color::from_hex(0xdeb887));
    // Scallop bumps on roof edge
    for i in 0..6 {
        let sx = x - 1.0 + i as f32 * (w + 4.0) / 6.0;
        draw_circle(sx + 4.0, y + 4.0, 4.0, Color::from_hex(0xdeb887));
    }

    // Ice cream cone sign (center of roof)
    let cx = x + w / 2.0;
    let cy = y - roof_h * 0.3;
    // Cone (triangle)
    draw_triangle(
        Vec2::new(cx - 5.0, cy + 2.0),
        Vec2::new(cx + 5.0, cy + 2.0),
        Vec2::new(cx, cy + 14.0),
        Color::from_hex(0xd2a05a),
    );
    // Scoops
    draw_circle(cx - 4.0, cy, 5.0, Color::from_hex(0xff69b4)); // strawberry
    draw_circle(cx + 4.0, cy, 5.0, Color::from_hex(0xfff8dc)); // vanilla
    draw_circle(cx, cy - 4.0, 5.0, Color::from_hex(0x8b4513));  // chocolate

    // Windows (two small squares)
    draw_rectangle(x + 4.0, y + 6.0, ts * 0.6, ts * 0.6, Color::from_hex(0xfff0f5));
    draw_rectangle_lines(x + 4.0, y + 6.0, ts * 0.6, ts * 0.6, 1.0, Color::from_hex(0xcc8899));
    draw_rectangle(x + w - ts * 0.6 - 4.0, y + 6.0, ts * 0.6, ts * 0.6, Color::from_hex(0xfff0f5));
    draw_rectangle_lines(x + w - ts * 0.6 - 4.0, y + 6.0, ts * 0.6, ts * 0.6, 1.0, Color::from_hex(0xcc8899));

    // Door (center bottom)
    let dw = ts * 0.5;
    let dh = ts * 0.8;
    draw_rectangle(x + w / 2.0 - dw / 2.0, y + h - dh, dw, dh, Color::from_hex(0x996666));
    // Doorknob
    draw_circle(x + w / 2.0 + dw * 0.25, y + h - dh * 0.4, 1.5, Color::from_hex(0xf1c40f));

    // "ICE CREAM" text
    draw_text("ICE CREAM", x + 3.0, y + h * 0.52, 10.0, Color::from_hex(0xcc3366));

    // Awning stripes (pink and white)
    let aw_h = 6.0;
    let aw_y = y + h * 0.58;
    for i in 0..10 {
        let stripe_x = x + i as f32 * (w / 10.0);
        let color = if i % 2 == 0 { Color::from_hex(0xff69b4) } else { WHITE };
        draw_rectangle(stripe_x, aw_y, w / 10.0, aw_h, color);
    }

    // "Summer Only!" tag
    draw_text("Summer Only!", x + 2.0, y + h + 10.0, 9.0, Color::from_hex(0xff6699));
}

/// Arcade — neon-lit entertainment building, 7×5 tiles
fn draw_arcade(x: f32, y: f32) {
    let w = 7.0 * TS;
    let h = 5.0 * TS;
    // Shadow
    draw_rectangle(x + 6.0, y + h, w, 10.0, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.2 });
    // Walls — dark purple/black
    draw_rectangle(x, y + 20.0, w, h - 20.0, Color::from_hex(0x1a1a2e));
    // Roof — flat with neon trim
    draw_rectangle(x - 4.0, y + 12.0, w + 8.0, 10.0, Color::from_hex(0x2a2a4e));
    draw_rectangle(x - 4.0, y + 20.0, w + 8.0, 3.0, Color::from_hex(0xff00ff)); // neon pink trim
    // Sign — "ARCADE" in neon
    let sign_x = x + w / 2.0 - 42.0;
    draw_rectangle(sign_x - 4.0, y - 2.0, 92.0, 18.0, Color::from_hex(0x1a1a1a));
    draw_text("ARCADE", sign_x + 2.0, y + 12.0, 16.0, Color::from_hex(0x00ffff));
    // Neon glow around sign
    draw_rectangle(sign_x - 6.0, y - 4.0, 96.0, 22.0, Color { r: 0.0, g: 1.0, b: 1.0, a: 0.08 });
    // Windows showing game screens
    let win_y = y + 32.0;
    for i in 0..3 {
        let wx = x + 12.0 + i as f32 * 70.0;
        draw_rectangle(wx, win_y, 28.0, 36.0, Color::from_hex(0x111122));
        // Screen glow
        let colors = [0x00ff00, 0xff4444, 0x4444ff];
        draw_rectangle(wx + 4.0, win_y + 4.0, 20.0, 20.0, Color::from_hex(colors[i as usize]));
        // Pixel art on screens
        draw_rectangle(wx + 8.0, win_y + 8.0, 4.0, 4.0, WHITE);
        draw_rectangle(wx + 14.0, win_y + 12.0, 6.0, 4.0, WHITE);
        // Joystick below screen
        draw_rectangle(wx + 10.0, win_y + 28.0, 8.0, 6.0, Color::from_hex(0x444444));
        draw_circle(wx + 14.0, win_y + 30.0, 3.0, Color::from_hex(0xff0000));
    }
    // Door
    let dx = x + w / 2.0 - 13.0;
    let dy = y + h - 32.0;
    draw_rectangle(dx, dy, 26.0, 32.0, Color::from_hex(0x333355));
    draw_rectangle_lines(dx, dy, 26.0, 32.0, 2.0, Color::from_hex(0xff00ff));
    draw_circle(dx + 20.0, dy + 18.0, 3.0, Color::from_hex(0x00ffff));
    // "OPEN" neon
    draw_text("OPEN", dx + 2.0, dy + 12.0, 10.0, Color::from_hex(0x00ff00));
    // Stars/sparkles decoration
    for &(sx, sy) in &[(x+8.0, y+28.0), (x+w-16.0, y+30.0), (x+w/2.0, y+24.0)] {
        draw_circle(sx, sy, 2.0, Color::from_hex(0xffff00));
    }
}

/// Animal Shop — barn-style building, 7×6 tiles
fn draw_animal_shop(x: f32, y: f32) {
    let w = 7.0 * TS;
    let h = 6.0 * TS;
    // Shadow
    draw_rectangle(x + 6.0, y + h, w, 10.0, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.2 });
    // Barn walls — red
    draw_rectangle(x, y + 28.0, w, h - 28.0, Color::from_hex(0xb03030));
    // Horizontal plank lines
    for i in 0..8u32 {
        draw_line(x, y + 28.0 + i as f32 * 14.0, x + w, y + 28.0 + i as f32 * 14.0,
                  1.0, Color::from_hex(0x8a2020));
    }
    // White trim
    draw_rectangle(x - 2.0, y + 26.0, w + 4.0, 4.0, Color::from_hex(0xf0e8d0));
    // Barn roof — triangular
    draw_triangle(
        Vec2::new(x - 8.0, y + 28.0),
        Vec2::new(x + w + 8.0, y + 28.0),
        Vec2::new(x + w / 2.0, y),
        Color::from_hex(0x6a3a2a),
    );
    draw_triangle(
        Vec2::new(x - 4.0, y + 28.0),
        Vec2::new(x + w + 4.0, y + 28.0),
        Vec2::new(x + w / 2.0, y + 4.0),
        Color::from_hex(0x7a4a3a),
    );
    // Sign
    let sign_x = x + w / 2.0 - 44.0;
    draw_rectangle(sign_x, y + 10.0, 88.0, 16.0, Color::from_hex(0xf5f0e0));
    draw_rectangle_lines(sign_x, y + 10.0, 88.0, 16.0, 2.0, Color::from_hex(0x8b5e3c));
    draw_text("ANIMALS", sign_x + 12.0, y + 23.0, 14.0, Color::from_hex(0x6a3a2a));
    // Hay loft opening (dark rectangle at top)
    draw_rectangle(x + w / 2.0 - 16.0, y + 16.0, 32.0, 12.0, Color::from_hex(0x3a1a0a));
    // Hay visible inside
    draw_rectangle(x + w / 2.0 - 12.0, y + 22.0, 24.0, 6.0, Color::from_hex(0xd4a030));
    // Barn doors (double, center bottom)
    let dx = x + w / 2.0 - 20.0;
    let dy = y + h - 40.0;
    draw_rectangle(dx, dy, 40.0, 40.0, Color::from_hex(0x8a2020));
    draw_line(dx + 20.0, dy, dx + 20.0, dy + 40.0, 2.0, Color::from_hex(0x6a1010));
    draw_rectangle_lines(dx, dy, 40.0, 40.0, 2.0, Color::from_hex(0xf0e8d0));
    // X brace on doors
    draw_line(dx + 2.0, dy + 2.0, dx + 18.0, dy + 38.0, 2.0, Color::from_hex(0xf0e8d0));
    draw_line(dx + 18.0, dy + 2.0, dx + 2.0, dy + 38.0, 2.0, Color::from_hex(0xf0e8d0));
    draw_line(dx + 22.0, dy + 2.0, dx + 38.0, dy + 38.0, 2.0, Color::from_hex(0xf0e8d0));
    draw_line(dx + 38.0, dy + 2.0, dx + 22.0, dy + 38.0, 2.0, Color::from_hex(0xf0e8d0));
    // Windows on sides
    for &wx in &[x + 12.0, x + w - 36.0] {
        draw_rectangle(wx, y + 48.0, 24.0, 20.0, Color::from_hex(0xa8d8f0));
        draw_rectangle_lines(wx, y + 48.0, 24.0, 20.0, 2.0, Color::from_hex(0xf0e8d0));
        draw_line(wx + 12.0, y + 48.0, wx + 12.0, y + 68.0, 1.0, Color::from_hex(0xf0e8d0));
    }
}

/// Furniture Shop — cozy storefront with display windows, 7×6 tiles
fn draw_furniture_shop(x: f32, y: f32) {
    let w = 7.0 * TS;
    let h = 6.0 * TS;
    // Shadow
    draw_rectangle(x + 6.0, y + h, w, 10.0, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.2 });
    // Walls — warm wood
    draw_rectangle(x, y + 20.0, w, h - 20.0, Color::from_hex(0xd4a050));
    for i in 0..7u32 {
        draw_line(x, y + 20.0 + i as f32 * 16.0, x + w, y + 20.0 + i as f32 * 16.0,
                  1.0, Color::from_hex(0xc09040));
    }
    // Roof — purple/teal
    draw_rectangle(x - 6.0, y + 10.0, w + 12.0, 12.0, Color::from_hex(0x6a4c93));
    draw_rectangle(x + 6.0, y + 3.0,  w - 12.0, 9.0,  Color::from_hex(0x7a5ca3));
    draw_rectangle(x + 16.0, y,        w - 32.0, 5.0,  Color::from_hex(0x6a4c93));
    draw_rectangle(x - 6.0, y + 20.0, w + 12.0, 4.0, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.2 });
    // Sign
    let sign_x = x + w / 2.0 - 50.0;
    draw_rectangle(sign_x, y + 5.0, 100.0, 16.0, Color::from_hex(0xf1c40f));
    draw_rectangle_lines(sign_x, y + 5.0, 100.0, 16.0, 2.0, Color::from_hex(0xb8860b));
    draw_text("FURNITURE", sign_x + 8.0, y + 18.0, 14.0, Color::from_hex(0x2c3e50));
    // Display windows — show furniture silhouettes
    // Left window (couch icon)
    let lw_x = x + 8.0;
    let lw_y = y + 36.0;
    draw_rectangle(lw_x, lw_y, 80.0, 50.0, Color::from_hex(0xa8d8f0));
    draw_rectangle_lines(lw_x, lw_y, 80.0, 50.0, 2.0, Color::from_hex(0xb8956a));
    // Mini couch in window
    draw_rectangle(lw_x + 14.0, lw_y + 24.0, 52.0, 20.0, Color::from_hex(0x8b4513));
    draw_rectangle(lw_x + 16.0, lw_y + 20.0, 48.0, 8.0, Color::from_hex(0xa0522d));
    // Right window (lamp icon)
    let rw_x = x + w - 88.0;
    let rw_y = y + 36.0;
    draw_rectangle(rw_x, rw_y, 80.0, 50.0, Color::from_hex(0xa8d8f0));
    draw_rectangle_lines(rw_x, rw_y, 80.0, 50.0, 2.0, Color::from_hex(0xb8956a));
    // Mini lamp in window
    draw_rectangle(rw_x + 34.0, rw_y + 18.0, 4.0, 28.0, Color::from_hex(0x888888));
    draw_triangle(
        Vec2::new(rw_x + 22.0, rw_y + 20.0),
        Vec2::new(rw_x + 50.0, rw_y + 20.0),
        Vec2::new(rw_x + 36.0, rw_y + 6.0),
        Color::from_hex(0xf39c12),
    );
    // Door (center)
    let dx = x + w / 2.0 - 13.0;
    let dy = y + h - 32.0;
    draw_rectangle(dx, dy, 26.0, 32.0, Color::from_hex(0x7a5030));
    draw_rectangle_lines(dx, dy, 26.0, 32.0, 2.0, Color::from_hex(0x5a3010));
    draw_circle(dx + 20.0, dy + 18.0, 3.0, Color::from_hex(0xf1c40f));
    // "OPEN" sign on door
    draw_rectangle(dx + 4.0, dy + 4.0, 18.0, 10.0, Color::from_hex(0x27ae60));
    draw_text("OPEN", dx + 5.0, dy + 12.0, 9.0, WHITE);
}

/// Playground — swing set, slide, sandbox, seesaw. 12×5 tiles
fn draw_playground(x: f32, y: f32) {
    let _w = 12.0 * TS;
    let _h = 5.0 * TS;

    // ── Ground surface (sand/dirt play area) ─────────────────────────────
    draw_rectangle(x, y, 12.0 * TS, 5.0 * TS, Color::from_hex(0xd2b48c));
    draw_rectangle_lines(x, y, 12.0 * TS, 5.0 * TS, 2.0, Color::from_hex(0xb8956a));
    // "PLAYGROUND" sign
    let sign_x = x + 6.0 * TS - 48.0;
    draw_rectangle(sign_x, y - 18.0, 96.0, 18.0, Color::from_hex(0x27ae60));
    draw_text("PLAYGROUND", sign_x + 6.0, y - 4.0, 14.0, WHITE);

    // ── Swing set (left, cols 1-3, rows 0-2) ────────────────────────────
    let sx = x + 1.0 * TS;
    let sy = y + 0.5 * TS;
    // A-frame posts
    draw_rectangle(sx, sy, 4.0, 2.5 * TS, Color::from_hex(0x8b4513));
    draw_rectangle(sx + 3.0 * TS - 4.0, sy, 4.0, 2.5 * TS, Color::from_hex(0x8b4513));
    // Crossbar
    draw_rectangle(sx, sy, 3.0 * TS, 4.0, Color::from_hex(0xa0522d));
    // Swing chains + seats
    for i in 0..2 {
        let cx = sx + 16.0 + i as f32 * 40.0;
        draw_line(cx, sy + 4.0, cx, sy + 50.0, 1.5, Color::from_hex(0x888888));
        draw_line(cx + 16.0, sy + 4.0, cx + 16.0, sy + 50.0, 1.5, Color::from_hex(0x888888));
        draw_rectangle(cx, sy + 50.0, 16.0, 4.0, Color::from_hex(0x2c3e50));
    }

    // ── Slide (center, cols 5-7, rows 0-3) ──────────────────────────────
    let sl_x = x + 5.0 * TS;
    let sl_y = y + 0.3 * TS;
    // Platform
    draw_rectangle(sl_x, sl_y, 2.0 * TS, 1.5 * TS, Color::from_hex(0xa0522d));
    draw_rectangle(sl_x + 4.0, sl_y + 4.0, 2.0 * TS - 8.0, 1.5 * TS - 8.0, Color::from_hex(0xc87030));
    // Ladder (left)
    draw_rectangle(sl_x - 2.0, sl_y, 4.0, 2.5 * TS, Color::from_hex(0x8b4513));
    for r in 0..5 {
        draw_rectangle(sl_x - 2.0, sl_y + 8.0 + r as f32 * 14.0, 16.0, 3.0, Color::from_hex(0xa0522d));
    }
    // Slide chute (right, angled)
    let chute_top_x = sl_x + 2.0 * TS;
    let chute_top_y = sl_y + 4.0;
    let chute_bot_x = sl_x + 3.0 * TS + 8.0;
    let chute_bot_y = sl_y + 3.0 * TS;
    // Slide surface (thick angled line approximated with a rotated rect)
    draw_line(chute_top_x, chute_top_y, chute_bot_x, chute_bot_y, 18.0, Color::from_hex(0xe74c3c));
    draw_line(chute_top_x, chute_top_y, chute_bot_x, chute_bot_y, 14.0, Color::from_hex(0xf05050));
    // Side rails
    draw_line(chute_top_x - 2.0, chute_top_y, chute_bot_x - 2.0, chute_bot_y, 2.0, Color::from_hex(0xcc3030));
    draw_line(chute_top_x + 14.0, chute_top_y, chute_bot_x + 14.0, chute_bot_y, 2.0, Color::from_hex(0xcc3030));

    // ── Sandbox (right, cols 9-11, rows 2-4) ────────────────────────────
    let bx = x + 9.0 * TS;
    let by = y + 2.0 * TS;
    draw_rectangle(bx, by, 3.0 * TS, 2.0 * TS, Color::from_hex(0xf5deb3));
    draw_rectangle_lines(bx, by, 3.0 * TS, 2.0 * TS, 3.0, Color::from_hex(0xa0522d));
    // Sand mounds
    draw_circle(bx + 20.0, by + 24.0, 10.0, Color::from_hex(0xe8d5a0));
    draw_circle(bx + 56.0, by + 36.0, 8.0, Color::from_hex(0xe8d5a0));
    draw_circle(bx + 40.0, by + 16.0, 6.0, Color::from_hex(0xe8d5a0));
    // Toy bucket
    draw_rectangle(bx + 68.0, by + 10.0, 12.0, 14.0, Color::from_hex(0x3498db));
    draw_rectangle(bx + 66.0, by + 8.0, 16.0, 4.0, Color::from_hex(0x2980b9));

    // ── Seesaw (bottom-left, cols 1-3, rows 3-4) ────────────────────────
    let sw_x = x + 1.0 * TS;
    let sw_y = y + 3.5 * TS;
    // Fulcrum (triangle)
    draw_triangle(
        Vec2::new(sw_x + 1.5 * TS - 8.0, sw_y + 20.0),
        Vec2::new(sw_x + 1.5 * TS + 8.0, sw_y + 20.0),
        Vec2::new(sw_x + 1.5 * TS, sw_y + 4.0),
        Color::from_hex(0x8b4513),
    );
    // Board (slightly tilted)
    draw_line(sw_x + 4.0, sw_y + 8.0, sw_x + 3.0 * TS - 4.0, sw_y + 14.0, 5.0, Color::from_hex(0x27ae60));
    // Handles
    draw_rectangle(sw_x + 6.0, sw_y + 2.0, 4.0, 10.0, Color::from_hex(0x888888));
    draw_rectangle(sw_x + 3.0 * TS - 10.0, sw_y + 8.0, 4.0, 10.0, Color::from_hex(0x888888));
}
