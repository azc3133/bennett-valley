use macroquad::prelude::*;
use crate::game::state::{BuildingKind, GameState};
use crate::render::player_view::draw_character;
use crate::render::camera::TILE_SIZE;

// Interior grid: 12×8 base, 16×10 upgraded.
const BASE_COLS: f32 = 12.0;
const BASE_ROWS: f32 = 8.0;
const UPG_COLS: f32 = 16.0;
const UPG_ROWS: f32 = 10.0;

// Thread-local grid size for the current draw call.
static mut GRID_COLS: f32 = 12.0;
static mut GRID_ROWS: f32 = 8.0;

fn set_grid_size(upgraded: bool) {
    unsafe {
        if upgraded { GRID_COLS = UPG_COLS; GRID_ROWS = UPG_ROWS; }
        else { GRID_COLS = BASE_COLS; GRID_ROWS = BASE_ROWS; }
    }
}
fn cols() -> f32 { unsafe { GRID_COLS } }
fn rows() -> f32 { unsafe { GRID_ROWS } }

fn grid_origin(sw: f32, sh: f32) -> (f32, f32) {
    let floor_top = 102.0_f32;
    let floor_bot = sh - 60.0;
    let grid_h = rows() * TILE_SIZE;
    let gx = sw / 2.0 - (cols() * TILE_SIZE) / 2.0;
    let gy = floor_top + (floor_bot - floor_top - grid_h) / 2.0;
    (gx, gy)
}

fn tile_px(col: i32, row: i32, sw: f32, sh: f32) -> (f32, f32) {
    let (gx, gy) = grid_origin(sw, sh);
    (gx + col as f32 * TILE_SIZE, gy + row as f32 * TILE_SIZE)
}

pub fn draw(state: &GameState) {
    let sw = screen_width();
    let sh = screen_height();

    // Set grid size based on whether the farmhouse is upgraded
    set_grid_size(state.current_building == BuildingKind::Farmhouse && state.house_upgraded);

    match state.current_building {
        BuildingKind::Farmhouse => draw_farmhouse(state, sw, sh),
        BuildingKind::Inn       => draw_inn_interior(state, sw, sh),
        BuildingKind::Market    => draw_market_interior(state, sw, sh),
        BuildingKind::Tavern    => draw_tavern_interior(state, sw, sh),
        BuildingKind::Clinic    => draw_clinic_interior(state, sw, sh),
        BuildingKind::Library   => draw_library_interior(state, sw, sh),
        BuildingKind::TownHall  => draw_townhall_interior(state, sw, sh),
        BuildingKind::FurnitureShop => {}
        BuildingKind::AnimalShop => {}
        BuildingKind::Arcade => {}
        BuildingKind::Restaurant => {}
        BuildingKind::IceCreamShop => {}
    }

    // Player character (same for all interiors)
    draw_player(state, sw, sh);

    // Stats + controls
    draw_footer(state, sw, sh);
}

// ── Shared helpers ───────────────────────────────────────────────────────────

fn draw_room_bg(sw: f32, sh: f32, wall_color: u32, floor_color: u32) {
    draw_rectangle(0.0, 36.0, sw, sh - 60.0, Color::from_hex(floor_color));
    draw_rectangle(0.0, 36.0, sw, 60.0, Color::from_hex(wall_color));
    draw_rectangle(0.0, 96.0, sw, 6.0, Color::from_hex(0x5a3e28)); // baseboard
}

fn draw_window(sw: f32) {
    let win_x = sw / 2.0 - 50.0;
    let win_y = 44.0;
    draw_rectangle(win_x, win_y, 100.0, 52.0, Color::from_hex(0x7ec8e3));
    draw_rectangle_lines(win_x, win_y, 100.0, 52.0, 4.0, Color::from_hex(0x8b5e3c));
    draw_line(win_x + 50.0, win_y, win_x + 50.0, win_y + 52.0, 3.0, Color::from_hex(0x8b5e3c));
    draw_line(win_x, win_y + 26.0, win_x + 100.0, win_y + 26.0, 3.0, Color::from_hex(0x8b5e3c));
}

fn draw_door(sw: f32, sh: f32, building: BuildingKind) {
    let (door_x, door_y) = tile_px(5, 7, sw, sh);
    let door_w = 2.0 * TILE_SIZE;
    let door_h = TILE_SIZE + 6.0;
    let door_bottom = door_y + door_h;
    draw_rectangle(door_x, door_bottom - door_h, door_w, door_h, Color::from_hex(0x6b3a2a));
    draw_rectangle(door_x + 4.0, door_bottom - door_h + 4.0, door_w - 8.0, door_h - 4.0,
                   Color::from_hex(0xc8a870));
    draw_circle(door_x + door_w - 10.0, door_bottom - door_h / 2.0, 4.0, Color::from_hex(0xd4af37));
    let exit_hint = "S: go outside";
    let ew = measure_text(exit_hint, None, 12, 1.0).width;
    draw_text(exit_hint, door_x + door_w / 2.0 - ew / 2.0, door_bottom - door_h - 4.0,
              12.0, Color::from_hex(0x888888));
    // Sleep hint only in farmhouse
    if building == BuildingKind::Farmhouse {
        // shown near the bed, not the door
    }
}

fn draw_player(state: &GameState, sw: f32, sh: f32) {
    let (col, row) = state.farmhouse_tile;
    let (px, py) = tile_px(col, row, sw, sh);
    let skin  = Color { r: 0.96, g: 0.80, b: 0.62, a: 1.0 };
    let hair  = Color { r: 0.32, g: 0.18, b: 0.08, a: 1.0 };
    let shirt = Color { r: 0.90, g: 0.92, b: 1.00, a: 1.0 };
    let pants = Color { r: 0.22, g: 0.32, b: 0.62, a: 1.0 };
    let shoes = Color { r: 0.22, g: 0.14, b: 0.08, a: 1.0 };
    let hat   = Color { r: 0.15, g: 0.55, b: 0.25, a: 1.0 };
    draw_character(px, py, shirt, pants, shoes, skin, hair, &state.player.facing);
    draw_rectangle(px + 5.0,  py - 2.0, 22.0, 4.0, hat);
    draw_rectangle(px + 9.0,  py - 9.0, 14.0, 8.0, hat);
    draw_rectangle(px + 9.0,  py - 3.0, 14.0, 2.0, Color::from_hex(0xf1c40f));
}

fn draw_footer(state: &GameState, sw: f32, sh: f32) {
    let stats = format!("Gold: {}g   Energy: {}/{}", state.player.gold, state.player.energy, state.player.max_energy);
    let sw2 = measure_text(&stats, None, 16, 1.0).width;
    draw_text(&stats, sw / 2.0 - sw2 / 2.0, sh - 44.0, 16.0, Color::from_hex(0xddddcc));

    draw_rectangle(0.0, sh - 24.0, sw, 24.0, Color::from_hex(0x1a1a2e));
    let hint = if state.current_building == BuildingKind::Farmhouse {
        "WASD: Move  |  Z: Sleep until morning  |  S at door: Go outside  |  Esc: Go outside"
    } else {
        "WASD: Move  |  S at door: Go outside  |  Esc: Go outside"
    };
    let hw = measure_text(hint, None, 13, 1.0).width;
    draw_text(hint, sw / 2.0 - hw / 2.0, sh - 6.0, 13.0, Color::from_hex(0xaaaaaa));
}

fn draw_sign(text: &str, sw: f32) {
    let tw = measure_text(text, None, 18, 1.0).width;
    let x = sw / 2.0 - tw / 2.0 - 8.0;
    draw_rectangle(x, 42.0, tw + 16.0, 24.0, Color::from_hex(0x6b3a2a));
    draw_text(text, x + 8.0, 60.0, 18.0, Color::from_hex(0xf1c40f));
}

fn draw_table(col: i32, row: i32, w_tiles: f32, h_tiles: f32, sw: f32, sh: f32) {
    let (x, y) = tile_px(col, row, sw, sh);
    let tw = w_tiles * TILE_SIZE;
    let th = h_tiles * TILE_SIZE - 8.0;
    draw_rectangle(x, y, tw, th, Color::from_hex(0x8b5e3c));
    draw_rectangle(x + 3.0, y + 3.0, tw - 6.0, th - 6.0, Color::from_hex(0xa0714a));
    // Legs
    draw_rectangle(x + 4.0, y + th, 8.0, 16.0, Color::from_hex(0x6b3a2a));
    draw_rectangle(x + tw - 12.0, y + th, 8.0, 16.0, Color::from_hex(0x6b3a2a));
}

fn draw_chair(col: i32, row: i32, sw: f32, sh: f32) {
    let (x, y) = tile_px(col, row, sw, sh);
    draw_rectangle(x + 6.0, y + 4.0, 20.0, 20.0, Color::from_hex(0x6b3a2a));
    draw_rectangle(x + 8.0, y + 6.0, 16.0, 16.0, Color::from_hex(0x8b5e3c));
}

// ── Farmhouse ────────────────────────────────────────────────────────────────

fn draw_farmhouse(state: &GameState, sw: f32, sh: f32) {
    draw_room_bg(sw, sh, 0x3a2a4a, 0x2c1f14);
    draw_window(sw);

    let (gx, gy) = grid_origin(sw, sh);

    // Bookshelf (cols 0-1, rows 0-2)
    let shelf_w = 2.0 * TILE_SIZE;
    let shelf_h = 3.0 * TILE_SIZE;
    draw_rectangle(gx, gy, shelf_w, shelf_h, Color::from_hex(0x6b3a2a));
    for s in 0..2_u32 {
        let sy = gy + s as f32 * (shelf_h / 2.0) + 4.0;
        draw_rectangle(gx + 4.0, sy, shelf_w - 8.0, shelf_h / 2.0 - 8.0, Color::from_hex(0x1a1a2e));
        let colors: &[u32] = if s == 0 {
            &[0xc0392b, 0x2980b9, 0xf39c12, 0x27ae60, 0x8e44ad, 0xe74c3c]
        } else {
            &[0x16a085, 0xd35400, 0x2c3e50, 0x1abc9c, 0xe67e22, 0x8e44ad]
        };
        for (i, &c) in colors.iter().enumerate() {
            let bx = gx + 6.0 + i as f32 * (shelf_w - 12.0) / colors.len() as f32;
            draw_rectangle(bx, sy + 2.0, 10.0, shelf_h / 2.0 - 12.0, Color::from_hex(c));
        }
    }

    // Table (cols 0-2, rows 3-4)
    draw_table(0, 3, 3.0, 2.0, sw, sh);
    // Mug on table
    let (tbl_x, tbl_y) = tile_px(0, 3, sw, sh);
    let tbl_w = 3.0 * TILE_SIZE;
    draw_rectangle(tbl_x + tbl_w / 2.0 - 12.0, tbl_y - 22.0, 24.0, 22.0, Color::from_hex(0xf5f5dc));
    draw_circle(tbl_x + tbl_w / 2.0, tbl_y - 11.0, 7.0, Color::from_hex(0x6f3500));

    // Bed (cols 9-11, rows 2-4)
    let (bed_x, bed_y) = tile_px(9, 2, sw, sh);
    let bed_w = 3.0 * TILE_SIZE;
    let bed_h = 3.0 * TILE_SIZE;
    draw_rectangle(bed_x, bed_y, bed_w, bed_h, Color::from_hex(0x6b3a2a));
    draw_rectangle(bed_x + 6.0, bed_y + 6.0, bed_w - 12.0, bed_h - 12.0, Color::from_hex(0xe8dcc8));
    draw_rectangle(bed_x + 10.0, bed_y + 10.0, 48.0, 36.0, WHITE);
    draw_rectangle(bed_x + 6.0, bed_y + 50.0, bed_w - 12.0, bed_h - 56.0, Color::from_hex(0x4a7a5a));
    let hint = "Z: sleep";
    let hw = measure_text(hint, None, 12, 1.0).width;
    draw_text(hint, bed_x + bed_w / 2.0 - hw / 2.0, bed_y + bed_h + 14.0, 12.0, Color::from_hex(0x888888));

    // P2 bed (cols 9-11, rows 5-6) — only when co-op active
    if state.coop_active {
        let (b2x, b2y) = tile_px(9, 5, sw, sh);
        let b2w = 3.0 * TILE_SIZE;
        let b2h = 2.0 * TILE_SIZE;
        draw_rectangle(b2x, b2y, b2w, b2h, Color::from_hex(0x6b3a2a));
        draw_rectangle(b2x + 6.0, b2y + 6.0, b2w - 12.0, b2h - 12.0, Color::from_hex(0xe8dcc8));
        draw_rectangle(b2x + 10.0, b2y + 8.0, 48.0, 24.0, WHITE);
        draw_rectangle(b2x + 6.0, b2y + 36.0, b2w - 12.0, b2h - 42.0, Color::from_hex(0x4a5a7a));
        let hint2 = "P2 bed";
        let hw2 = measure_text(hint2, None, 10, 1.0).width;
        draw_text(hint2, b2x + b2w / 2.0 - hw2 / 2.0, b2y + b2h + 10.0, 10.0, Color::from_hex(0x3498db));
    }

    // ── Purchased furniture ─────────────────────────────────────────────
    draw_owned_furniture(state, sw, sh);

    // ── Upgraded extension — extra room on the right ─────────────────
    if state.house_upgraded {
        // Dining area (cols 12-15, rows 2-4)
        draw_table(12, 2, 3.0, 2.0, sw, sh);
        draw_chair(12, 4, sw, sh);
        draw_chair(15, 4, sw, sh);

        // Kitchen counter (cols 12-15, row 0-1)
        let (kx, ky) = tile_px(12, 0, sw, sh);
        draw_rectangle(kx, ky, 4.0 * TILE_SIZE, 2.0 * TILE_SIZE - 6.0, Color::from_hex(0x888888));
        draw_rectangle(kx + 4.0, ky + 4.0, 4.0 * TILE_SIZE - 8.0, 2.0 * TILE_SIZE - 14.0, Color::from_hex(0xa0a0a0));
        // Stove
        draw_rectangle(kx + 10.0, ky + 6.0, 28.0, 20.0, Color::from_hex(0x333333));
        draw_circle(kx + 18.0, ky + 14.0, 5.0, Color::from_hex(0x555555));
        draw_circle(kx + 32.0, ky + 14.0, 5.0, Color::from_hex(0x555555));
        // Sink
        draw_rectangle(kx + 60.0, ky + 8.0, 24.0, 16.0, Color::from_hex(0xcccccc));
        draw_rectangle(kx + 64.0, ky + 12.0, 16.0, 8.0, Color::from_hex(0x4a9adf));

        // Extra window on right wall
        let (wx, wy) = tile_px(15, 5, sw, sh);
        draw_rectangle(wx, wy, TILE_SIZE, TILE_SIZE + 8.0, Color::from_hex(0x7ec8e3));
        draw_rectangle_lines(wx, wy, TILE_SIZE, TILE_SIZE + 8.0, 3.0, Color::from_hex(0x8b5e3c));
        draw_line(wx + TILE_SIZE / 2.0, wy, wx + TILE_SIZE / 2.0, wy + TILE_SIZE + 8.0, 2.0, Color::from_hex(0x8b5e3c));

        // Bathroom area (cols 12-13, rows 7-9)
        let (bx, by) = tile_px(12, 7, sw, sh);
        // Bathtub
        draw_rectangle(bx, by, 2.0 * TILE_SIZE, TILE_SIZE + 4.0, Color::from_hex(0xeeeeee));
        draw_rectangle(bx + 4.0, by + 4.0, 2.0 * TILE_SIZE - 8.0, TILE_SIZE - 4.0, Color::from_hex(0x7ec8e3));
        // Toilet
        let (tx2, ty2) = tile_px(14, 8, sw, sh);
        draw_rectangle(tx2 + 4.0, ty2, 20.0, 22.0, Color::from_hex(0xeeeeee));
        draw_rectangle(tx2 + 6.0, ty2 + 2.0, 16.0, 10.0, Color::from_hex(0xdddddd));
    }

    draw_door(sw, sh, BuildingKind::Farmhouse);
}

fn draw_owned_furniture(state: &GameState, sw: f32, sh: f32) {
    use crate::game::state::FurnitureKind;

    // Lamp — left side, col 4, row 0
    if state.owned_furniture.contains(&FurnitureKind::Lamp) {
        let (lx, ly) = tile_px(4, 0, sw, sh);
        draw_rectangle(lx + 12.0, ly + 14.0, 8.0, 20.0, Color::from_hex(0x888888));
        draw_rectangle(lx + 8.0, ly + 32.0, 16.0, 4.0, Color::from_hex(0x666666));
        draw_triangle(
            Vec2::new(lx + 2.0, ly + 16.0),
            Vec2::new(lx + 30.0, ly + 16.0),
            Vec2::new(lx + 16.0, ly),
            Color::from_hex(0xf39c12),
        );
        draw_circle(lx + 16.0, ly + 8.0, 4.0, Color::from_hex(0xfff8dc));
    }

    // Fish Tank — near window, cols 6-7, row 0
    if state.owned_furniture.contains(&FurnitureKind::FishTank) {
        let (fx, fy) = tile_px(6, 0, sw, sh);
        let ftw = 2.0 * TILE_SIZE;
        let fth = TILE_SIZE + 4.0;
        // Stand
        draw_rectangle(fx + 4.0, fy + fth, ftw - 8.0, 8.0, Color::from_hex(0x555555));
        // Tank
        draw_rectangle(fx, fy, ftw, fth, Color::from_hex(0x1a6faa));
        draw_rectangle_lines(fx, fy, ftw, fth, 2.0, Color::from_hex(0x888888));
        // Fish
        draw_circle(fx + 14.0, fy + 16.0, 4.0, Color::from_hex(0xe74c3c));
        draw_circle(fx + 36.0, fy + 10.0, 3.0, Color::from_hex(0xf39c12));
        draw_circle(fx + 48.0, fy + 22.0, 3.5, Color::from_hex(0x2ecc71));
        // Bubbles
        draw_circle(fx + 24.0, fy + 6.0, 2.0, Color { r: 0.8, g: 0.9, b: 1.0, a: 0.5 });
        draw_circle(fx + 42.0, fy + 4.0, 1.5, Color { r: 0.8, g: 0.9, b: 1.0, a: 0.5 });
    }

    // TV — right wall above bed, cols 9-10, row 0
    if state.owned_furniture.contains(&FurnitureKind::TV) {
        let (tx, ty) = tile_px(9, 0, sw, sh);
        let tvw = 2.0 * TILE_SIZE;
        // TV body
        draw_rectangle(tx, ty + 4.0, tvw, TILE_SIZE + 8.0, Color::from_hex(0x222222));
        draw_rectangle(tx + 4.0, ty + 8.0, tvw - 8.0, TILE_SIZE, Color::from_hex(0x3498db));
        // Screen glow
        draw_rectangle(tx + 8.0, ty + 12.0, tvw - 16.0, TILE_SIZE - 8.0,
                       Color { r: 0.4, g: 0.7, b: 1.0, a: 0.3 });
        // Stand
        draw_rectangle(tx + tvw / 2.0 - 8.0, ty + TILE_SIZE + 12.0, 16.0, 6.0, Color::from_hex(0x444444));
    }

    // Couch — center, cols 4-6, rows 4-5
    if state.owned_furniture.contains(&FurnitureKind::Couch) {
        let (cx, cy) = tile_px(4, 4, sw, sh);
        let cw = 3.0 * TILE_SIZE;
        let ch = 2.0 * TILE_SIZE - 8.0;
        // Frame
        draw_rectangle(cx, cy, cw, ch, Color::from_hex(0x6b3a2a));
        // Cushions
        draw_rectangle(cx + 4.0, cy + 4.0, cw - 8.0, ch - 12.0, Color::from_hex(0xc0392b));
        // Back
        draw_rectangle(cx + 2.0, cy, cw - 4.0, 10.0, Color::from_hex(0x922b21));
        // Armrests
        draw_rectangle(cx, cy, 10.0, ch, Color::from_hex(0x8b4513));
        draw_rectangle(cx + cw - 10.0, cy, 10.0, ch, Color::from_hex(0x8b4513));
        // Pillows
        draw_rectangle(cx + 14.0, cy + 6.0, 20.0, 16.0, Color::from_hex(0xf39c12));
        draw_rectangle(cx + cw - 34.0, cy + 6.0, 20.0, 16.0, Color::from_hex(0x27ae60));
    }

    // Rug — floor center, cols 4-7, rows 5-6 (no collision)
    if state.owned_furniture.contains(&FurnitureKind::Rug) {
        let (rx, ry) = tile_px(4, 5, sw, sh);
        let rw = 4.0 * TILE_SIZE;
        let rh = 2.0 * TILE_SIZE;
        draw_rectangle(rx, ry, rw, rh, Color::from_hex(0x8e3030));
        draw_rectangle(rx + 8.0, ry + 8.0, rw - 16.0, rh - 16.0, Color::from_hex(0xa04040));
        draw_rectangle(rx + 16.0, ry + 16.0, rw - 32.0, rh - 32.0, Color::from_hex(0xb85050));
        // Corner details
        for &(dx2, dy2) in &[(12.0, 12.0), (rw-18.0, 12.0), (12.0, rh-18.0), (rw-18.0, rh-18.0)] {
            draw_circle(rx + dx2, ry + dy2, 3.0, Color::from_hex(0xf1c40f));
        }
    }

    // Potted Plant — right side, col 11, row 5
    if state.owned_furniture.contains(&FurnitureKind::PottedPlant) {
        let (px, py) = tile_px(11, 5, sw, sh);
        // Pot
        draw_rectangle(px + 4.0, py + 16.0, 24.0, 16.0, Color::from_hex(0xd4a050));
        draw_rectangle(px + 2.0, py + 14.0, 28.0, 4.0, Color::from_hex(0xc09040));
        // Plant
        draw_circle(px + 16.0, py + 8.0, 10.0, Color::from_hex(0x27ae60));
        draw_circle(px + 10.0, py + 4.0, 7.0, Color::from_hex(0x2ecc71));
        draw_circle(px + 22.0, py + 6.0, 6.0, Color::from_hex(0x27ae60));
        // Stem
        draw_rectangle(px + 14.0, py + 14.0, 4.0, 6.0, Color::from_hex(0x1a8a1a));
    }
}

// ── Inn ──────────────────────────────────────────────────────────────────────

fn draw_inn_interior(_state: &GameState, sw: f32, sh: f32) {
    draw_room_bg(sw, sh, 0x4a3228, 0x3a2618);
    draw_sign("INN", sw);

    // Fireplace (right wall, cols 10-11, rows 0-2)
    let (fx, fy) = tile_px(10, 0, sw, sh);
    draw_rectangle(fx, fy, 2.0 * TILE_SIZE, 3.0 * TILE_SIZE, Color::from_hex(0x555555));
    draw_rectangle(fx + 8.0, fy + TILE_SIZE, TILE_SIZE + 16.0, 2.0 * TILE_SIZE, Color::from_hex(0x222222));
    // Fire glow
    draw_circle(fx + TILE_SIZE, fy + 2.0 * TILE_SIZE, 12.0, Color { r: 1.0, g: 0.4, b: 0.1, a: 0.6 });
    draw_circle(fx + TILE_SIZE, fy + 2.0 * TILE_SIZE - 4.0, 8.0, Color { r: 1.0, g: 0.7, b: 0.2, a: 0.8 });

    // Front desk (cols 4-7, rows 1-2)
    let (dx, dy) = tile_px(4, 1, sw, sh);
    draw_rectangle(dx, dy, 4.0 * TILE_SIZE, 2.0 * TILE_SIZE - 8.0, Color::from_hex(0x6b3a2a));
    draw_rectangle(dx + 4.0, dy + 4.0, 4.0 * TILE_SIZE - 8.0, 2.0 * TILE_SIZE - 16.0, Color::from_hex(0x8b5e3c));
    // Guest book
    draw_rectangle(dx + 20.0, dy + 8.0, 24.0, 16.0, Color::from_hex(0xc8a870));

    // Coat rack (col 0, row 0)
    let (cx, cy) = tile_px(0, 0, sw, sh);
    draw_rectangle(cx + 14.0, cy, 4.0, TILE_SIZE * 2.0, Color::from_hex(0x6b3a2a));
    draw_rectangle(cx + 6.0, cy + 8.0, 20.0, 4.0, Color::from_hex(0x6b3a2a));

    // Tables with chairs
    draw_table(1, 4, 2.0, 2.0, sw, sh);
    draw_chair(0, 4, sw, sh);
    draw_chair(3, 4, sw, sh);
    draw_table(7, 4, 2.0, 2.0, sw, sh);
    draw_chair(6, 4, sw, sh);
    draw_chair(9, 4, sw, sh);

    // Rug in center
    let (rx, ry) = tile_px(4, 4, sw, sh);
    draw_rectangle(rx, ry, 3.0 * TILE_SIZE, 2.0 * TILE_SIZE, Color { r: 0.6, g: 0.15, b: 0.1, a: 0.4 });

    draw_door(sw, sh, BuildingKind::Inn);
}

// ── Market ───────────────────────────────────────────────────────────────────

fn draw_market_interior(_state: &GameState, sw: f32, sh: f32) {
    draw_room_bg(sw, sh, 0x3a4a3a, 0x2a2018);
    draw_sign("MARKET", sw);

    // Counter/register (cols 4-7, row 1-2)
    let (cx, cy) = tile_px(4, 1, sw, sh);
    draw_rectangle(cx, cy, 4.0 * TILE_SIZE, 2.0 * TILE_SIZE - 8.0, Color::from_hex(0x8b5e3c));
    // Register
    draw_rectangle(cx + 50.0, cy + 4.0, 28.0, 20.0, Color::from_hex(0x444444));
    draw_rectangle(cx + 54.0, cy + 6.0, 20.0, 10.0, Color::from_hex(0x88ff88));

    // Left shelves (cols 0-1, rows 0-4)
    let (sx, sy) = tile_px(0, 0, sw, sh);
    draw_rectangle(sx, sy, 2.0 * TILE_SIZE, 5.0 * TILE_SIZE, Color::from_hex(0x6b3a2a));
    for r in 0..4 {
        let ry = sy + r as f32 * TILE_SIZE + 4.0;
        draw_rectangle(sx + 4.0, ry, TILE_SIZE * 2.0 - 8.0, TILE_SIZE - 8.0, Color::from_hex(0x4a3a2a));
        // Colorful produce on shelves
        let colors = [0xe74c3c, 0xf39c12, 0x27ae60, 0x3498db];
        for (i, &c) in colors.iter().enumerate() {
            draw_circle(sx + 12.0 + i as f32 * 14.0, ry + 10.0, 5.0, Color::from_hex(c));
        }
    }

    // Right shelves (cols 10-11, rows 0-4)
    let (sx, sy) = tile_px(10, 0, sw, sh);
    draw_rectangle(sx, sy, 2.0 * TILE_SIZE, 5.0 * TILE_SIZE, Color::from_hex(0x6b3a2a));
    for r in 0..4 {
        let ry = sy + r as f32 * TILE_SIZE + 4.0;
        draw_rectangle(sx + 4.0, ry, TILE_SIZE * 2.0 - 8.0, TILE_SIZE - 8.0, Color::from_hex(0x4a3a2a));
        let colors = [0x8e44ad, 0xf1c40f, 0xe67e22, 0x1abc9c];
        for (i, &c) in colors.iter().enumerate() {
            draw_circle(sx + 12.0 + i as f32 * 14.0, ry + 10.0, 5.0, Color::from_hex(c));
        }
    }

    // Crates (cols 8-9, rows 4-5)
    let (bx, by) = tile_px(8, 4, sw, sh);
    for r in 0..2 {
        for c in 0..2 {
            let x = bx + c as f32 * TILE_SIZE + 2.0;
            let y = by + r as f32 * TILE_SIZE + 2.0;
            draw_rectangle(x, y, TILE_SIZE - 4.0, TILE_SIZE - 4.0, Color::from_hex(0xd4a017));
            draw_line(x + 14.0, y, x + 14.0, y + TILE_SIZE - 4.0, 1.0, Color::from_hex(0xb88010));
        }
    }

    draw_door(sw, sh, BuildingKind::Market);
}

// ── Tavern ───────────────────────────────────────────────────────────────────

fn draw_tavern_interior(_state: &GameState, sw: f32, sh: f32) {
    draw_room_bg(sw, sh, 0x2a1a14, 0x1e1410);
    draw_sign("TAVERN", sw);

    // Warm ambient glow
    draw_rectangle(0.0, 36.0, sw, sh - 60.0, Color { r: 0.9, g: 0.6, b: 0.2, a: 0.06 });

    // Bar counter (cols 1-8, row 1)
    let (bx, by) = tile_px(1, 1, sw, sh);
    draw_rectangle(bx, by, 8.0 * TILE_SIZE, TILE_SIZE, Color::from_hex(0x5a3218));
    draw_rectangle(bx + 2.0, by + 2.0, 8.0 * TILE_SIZE - 4.0, TILE_SIZE - 4.0, Color::from_hex(0x7a4a28));
    // Bar taps
    for i in 0..3 {
        let tx = bx + 40.0 + i as f32 * 60.0;
        draw_rectangle(tx, by - 16.0, 8.0, 20.0, Color::from_hex(0xd4af37));
        draw_circle(tx + 4.0, by - 18.0, 5.0, Color::from_hex(0xd4af37));
    }

    // Bar stools (row 2)
    for col in &[2, 4, 6] {
        draw_chair(*col, 2, sw, sh);
    }

    // Kegs (cols 9-11, rows 0-2)
    let (kx, ky) = tile_px(9, 0, sw, sh);
    for r in 0..3 {
        for c in 0..3 {
            let x = kx + c as f32 * TILE_SIZE + 2.0;
            let y = ky + r as f32 * TILE_SIZE + 4.0;
            draw_rectangle(x, y, TILE_SIZE - 4.0, TILE_SIZE - 8.0, Color::from_hex(0x6b3a2a));
            draw_circle(x + TILE_SIZE / 2.0 - 2.0, y + TILE_SIZE / 2.0 - 4.0, 4.0, Color::from_hex(0xd4af37));
        }
    }

    // Tables (bottom area)
    draw_table(1, 4, 3.0, 2.0, sw, sh);
    draw_chair(0, 4, sw, sh);
    draw_chair(0, 5, sw, sh);
    draw_chair(4, 4, sw, sh);
    draw_table(7, 4, 3.0, 2.0, sw, sh);
    draw_chair(6, 4, sw, sh);
    draw_chair(10, 4, sw, sh);

    draw_door(sw, sh, BuildingKind::Tavern);
}

// ── Clinic ───────────────────────────────────────────────────────────────────

fn draw_clinic_interior(_state: &GameState, sw: f32, sh: f32) {
    draw_room_bg(sw, sh, 0xe8e8e8, 0xd0d8d0);
    draw_sign("CLINIC", sw);

    // Medicine cabinet (cols 0-1, rows 0-2)
    let (mx, my) = tile_px(0, 0, sw, sh);
    draw_rectangle(mx, my, 2.0 * TILE_SIZE, 3.0 * TILE_SIZE, Color::from_hex(0xeeeeee));
    draw_rectangle_lines(mx, my, 2.0 * TILE_SIZE, 3.0 * TILE_SIZE, 2.0, Color::from_hex(0x4a9adf));
    // Shelves with bottles
    for r in 0..3 {
        let ry = my + r as f32 * TILE_SIZE + 4.0;
        draw_line(mx + 4.0, ry + TILE_SIZE - 4.0, mx + 2.0 * TILE_SIZE - 4.0, ry + TILE_SIZE - 4.0, 1.0, Color::from_hex(0xaaaaaa));
        for i in 0..4 {
            let bx = mx + 8.0 + i as f32 * 14.0;
            let colors = [0x3498db, 0xe74c3c, 0x27ae60, 0xf39c12];
            draw_rectangle(bx, ry + 4.0, 8.0, 18.0, Color::from_hex(colors[i]));
        }
    }

    // Examination table (cols 7-9, rows 2-3)
    let (ex, ey) = tile_px(7, 2, sw, sh);
    draw_rectangle(ex, ey, 3.0 * TILE_SIZE, 2.0 * TILE_SIZE, Color::from_hex(0xcccccc));
    draw_rectangle(ex + 4.0, ey + 4.0, 3.0 * TILE_SIZE - 8.0, 2.0 * TILE_SIZE - 8.0, WHITE);
    // Pillow
    draw_rectangle(ex + 8.0, ey + 8.0, 30.0, 20.0, Color::from_hex(0xaaddff));

    // Desk (cols 3-5, row 1)
    draw_table(3, 1, 3.0, 1.5, sw, sh);
    draw_chair(3, 2, sw, sh);

    // Red cross on wall
    let cross_x = sw / 2.0 + 60.0;
    draw_rectangle(cross_x, 50.0, 8.0, 28.0, Color::from_hex(0xe74c3c));
    draw_rectangle(cross_x - 10.0, 60.0, 28.0, 8.0, Color::from_hex(0xe74c3c));

    draw_door(sw, sh, BuildingKind::Clinic);
}

// ── Library ──────────────────────────────────────────────────────────────────

fn draw_library_interior(_state: &GameState, sw: f32, sh: f32) {
    draw_room_bg(sw, sh, 0x3a3028, 0x2a2018);
    draw_sign("LIBRARY", sw);

    // Bookshelves on left wall (cols 0-1, rows 0-5)
    draw_bookshelf(0, 0, 6, sw, sh);
    // Bookshelves on right wall (cols 10-11, rows 0-5)
    draw_bookshelf(10, 0, 6, sw, sh);
    // Center shelf (cols 5-6, rows 0-2)
    draw_bookshelf(5, 0, 3, sw, sh);

    // Reading tables
    draw_table(3, 3, 2.0, 2.0, sw, sh);
    draw_chair(2, 3, sw, sh);
    draw_chair(5, 3, sw, sh);
    draw_table(7, 3, 2.0, 2.0, sw, sh);
    draw_chair(6, 3, sw, sh);
    draw_chair(9, 3, sw, sh);

    // Globe (col 9, row 5)
    let (gx, gy) = tile_px(9, 5, sw, sh);
    draw_circle(gx + 16.0, gy + 10.0, 12.0, Color::from_hex(0x3498db));
    draw_circle(gx + 16.0, gy + 10.0, 12.0, Color { r: 0.2, g: 0.6, b: 0.3, a: 0.4 });
    draw_rectangle(gx + 12.0, gy + 22.0, 8.0, 8.0, Color::from_hex(0x6b3a2a));

    // Cavalier King Charles Spaniel — curled up near the globe
    draw_cavalier(8, 6, sw, sh);

    draw_door(sw, sh, BuildingKind::Library);
}

/// Draw a small Cavalier King Charles Spaniel (brown and white).
fn draw_cavalier(col: i32, row: i32, sw: f32, sh: f32) {
    let (x, y) = tile_px(col, row, sw, sh);
    let cx = x + 16.0; // center of tile
    let cy = y + 18.0;

    // Body — white oval
    draw_circle(cx, cy, 10.0, Color::from_hex(0xf5f0e8));
    draw_circle(cx + 2.0, cy + 1.0, 9.0, Color::from_hex(0xede5d8));

    // Brown patches on back
    draw_circle(cx + 4.0, cy - 3.0, 5.0, Color::from_hex(0x8b4513));
    draw_circle(cx - 2.0, cy - 1.0, 4.0, Color::from_hex(0xa0522d));

    // Head — round, turned slightly
    let hx = cx - 8.0;
    let hy = cy - 6.0;
    draw_circle(hx, hy, 7.0, Color::from_hex(0xf5f0e8)); // white base
    // Brown ear (left, drooping)
    draw_circle(hx - 6.0, hy + 2.0, 5.0, Color::from_hex(0x8b4513));
    draw_circle(hx - 5.0, hy + 5.0, 4.0, Color::from_hex(0x7a3a10));
    // Brown ear (right, behind head)
    draw_circle(hx + 4.0, hy + 3.0, 4.0, Color::from_hex(0x8b4513));
    // Face — white with brown markings
    draw_circle(hx, hy, 6.5, Color::from_hex(0xf5f0e8));
    // Brown cap on top of head
    draw_circle(hx, hy - 3.0, 4.5, Color::from_hex(0x8b4513));
    // Eyes — big, round, soulful
    draw_circle(hx - 2.5, hy - 1.0, 2.0, Color::from_hex(0x1a1a1a));
    draw_circle(hx + 2.5, hy - 1.0, 2.0, Color::from_hex(0x1a1a1a));
    // Eye shine
    draw_circle(hx - 2.0, hy - 1.5, 0.8, WHITE);
    draw_circle(hx + 3.0, hy - 1.5, 0.8, WHITE);
    // Nose — small black
    draw_circle(hx, hy + 2.0, 1.5, Color::from_hex(0x1a1a1a));
    // Tiny smile
    draw_line(hx - 1.5, hy + 3.5, hx, hy + 4.0, 1.0, Color::from_hex(0x333333));
    draw_line(hx + 1.5, hy + 3.5, hx, hy + 4.0, 1.0, Color::from_hex(0x333333));

    // Curled tail — brown, curved upward
    draw_line(cx + 10.0, cy - 2.0, cx + 14.0, cy - 6.0, 2.5, Color::from_hex(0x8b4513));
    draw_line(cx + 14.0, cy - 6.0, cx + 12.0, cy - 9.0, 2.0, Color::from_hex(0xa0522d));

    // Front paws — white, tucked
    draw_circle(cx - 6.0, cy + 6.0, 3.0, Color::from_hex(0xf5f0e8));
    draw_circle(cx - 2.0, cy + 7.0, 2.5, Color::from_hex(0xf5f0e8));
}

fn draw_bookshelf(col: i32, start_row: i32, rows: i32, sw: f32, sh: f32) {
    let (sx, sy) = tile_px(col, start_row, sw, sh);
    let shelf_w = 2.0 * TILE_SIZE;
    let shelf_h = rows as f32 * TILE_SIZE;
    draw_rectangle(sx, sy, shelf_w, shelf_h, Color::from_hex(0x6b3a2a));
    let book_colors = [
        0xc0392b, 0x2980b9, 0xf39c12, 0x27ae60, 0x8e44ad, 0xe74c3c,
        0x16a085, 0xd35400, 0x2c3e50, 0x1abc9c, 0xe67e22, 0x8e44ad,
    ];
    for r in 0..rows {
        let ry = sy + r as f32 * TILE_SIZE + 4.0;
        draw_rectangle(sx + 4.0, ry, shelf_w - 8.0, TILE_SIZE - 8.0, Color::from_hex(0x1a1a2e));
        for i in 0..5 {
            let ci = ((col as usize * 3 + r as usize * 5 + i) % book_colors.len()) as usize;
            let bx = sx + 6.0 + i as f32 * (shelf_w - 12.0) / 5.0;
            draw_rectangle(bx, ry + 2.0, 10.0, TILE_SIZE - 12.0, Color::from_hex(book_colors[ci]));
        }
    }
}

// ── Town Hall ────────────────────────────────────────────────────────────────

fn draw_townhall_interior(_state: &GameState, sw: f32, sh: f32) {
    draw_room_bg(sw, sh, 0x2a2a3e, 0x1e1a14);
    draw_sign("TOWN HALL", sw);

    // Large desk (cols 3-8, rows 1-2)
    let (dx, dy) = tile_px(3, 1, sw, sh);
    draw_rectangle(dx, dy, 6.0 * TILE_SIZE, 2.0 * TILE_SIZE - 8.0, Color::from_hex(0x5a3218));
    draw_rectangle(dx + 4.0, dy + 4.0, 6.0 * TILE_SIZE - 8.0, 2.0 * TILE_SIZE - 16.0, Color::from_hex(0x7a4a28));
    // Inkwell + papers
    draw_rectangle(dx + 30.0, dy + 8.0, 40.0, 24.0, Color::from_hex(0xe8dcc8));
    draw_circle(dx + 100.0, dy + 16.0, 6.0, Color::from_hex(0x222222));

    // Chair behind desk
    draw_chair(5, 0, sw, sh);

    // Flag/banner (cols 0-1, row 0)
    let (fx, fy) = tile_px(0, 0, sw, sh);
    draw_rectangle(fx + 14.0, fy, 4.0, TILE_SIZE * 3.0, Color::from_hex(0x6b3a2a));
    draw_rectangle(fx + 18.0, fy + 4.0, 36.0, 48.0, Color::from_hex(0x2c3e50));
    // Star on banner
    draw_circle(fx + 36.0, fy + 28.0, 8.0, Color::from_hex(0xf1c40f));

    // Bulletin board (cols 10-11, rows 0-1)
    let (bx, by) = tile_px(10, 0, sw, sh);
    draw_rectangle(bx, by, 2.0 * TILE_SIZE, 2.0 * TILE_SIZE, Color::from_hex(0x8b5e3c));
    draw_rectangle(bx + 4.0, by + 4.0, 2.0 * TILE_SIZE - 8.0, 2.0 * TILE_SIZE - 8.0, Color::from_hex(0xd2b48c));
    // Pinned notes
    for i in 0..4 {
        let nx = bx + 8.0 + (i % 2) as f32 * 26.0;
        let ny = by + 8.0 + (i / 2) as f32 * 24.0;
        let colors = [0xfffacd, 0xadd8e6, 0xffb6c1, 0x98fb98];
        draw_rectangle(nx, ny, 22.0, 18.0, Color::from_hex(colors[i]));
        draw_circle(nx + 11.0, ny + 2.0, 2.0, Color::from_hex(0xe74c3c));
    }

    // Benches (rows 4-5)
    let (b1x, b1y) = tile_px(2, 4, sw, sh);
    draw_rectangle(b1x, b1y, 3.0 * TILE_SIZE, TILE_SIZE, Color::from_hex(0x6b3a2a));
    let (b2x, b2y) = tile_px(7, 4, sw, sh);
    draw_rectangle(b2x, b2y, 3.0 * TILE_SIZE, TILE_SIZE, Color::from_hex(0x6b3a2a));

    draw_door(sw, sh, BuildingKind::TownHall);
}
