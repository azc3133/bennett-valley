use macroquad::prelude::*;
use crate::game::state::{AnimalKind, GameState};
use crate::render::camera::{Camera, TILE_SIZE};

/// Draw the animal shop overlay UI.
pub fn draw_shop(state: &GameState) {
    let sw = screen_width();
    let sh = screen_height();

    let row_h = 36.0;
    let header_h = 60.0;
    let footer_h = 40.0;
    let items = AnimalKind::ALL;
    let total_rows = state.animal_shop_count();
    let box_h = (header_h + total_rows as f32 * row_h + footer_h).min(sh - 40.0);
    let box_w = 400.0f32.min(sw - 40.0);
    let box_x = sw / 2.0 - box_w / 2.0;
    let box_y = sh / 2.0 - box_h / 2.0;

    draw_rectangle(box_x, box_y, box_w, box_h, Color::from_hex(0x3a1a0a));
    draw_rectangle_lines(box_x, box_y, box_w, box_h, 2.0, Color::from_hex(0xb03030));

    let title = "Animal Shop";
    let tw = measure_text(title, None, 22, 1.0).width;
    draw_text(title, box_x + box_w / 2.0 - tw / 2.0, box_y + 28.0, 22.0, Color::from_hex(0xf1c40f));

    let gold_text = format!("Gold: {}g", state.player.gold);
    let gw = measure_text(&gold_text, None, 16, 1.0).width;
    draw_text(&gold_text, box_x + box_w - gw - 14.0, box_y + 50.0, 16.0, Color::from_hex(0xf1c40f));

    let list_y = box_y + header_h;

    // Animals
    for (i, item) in items.iter().enumerate() {
        let ry = list_y + i as f32 * row_h;
        let selected = i == state.animal_cursor;
        let owned = state.owned_animals.contains(item);

        if selected {
            draw_rectangle(box_x + 4.0, ry, box_w - 8.0, row_h - 2.0, Color::from_hex(0x5a2a1a));
        }

        draw_animal_icon(*item, box_x + 14.0, ry + 6.0);

        let name_color = if owned { Color::from_hex(0x888888) } else if selected { WHITE } else { Color::from_hex(0xcccccc) };
        draw_text(item.name(), box_x + 50.0, ry + 22.0, 16.0, name_color);

        if owned {
            let ow = measure_text("Owned", None, 14, 1.0).width;
            draw_text("Owned", box_x + box_w - ow - 14.0, ry + 22.0, 14.0, Color::from_hex(0x27ae60));
        } else {
            let price_text = format!("{}g", item.price());
            let pw = measure_text(&price_text, None, 16, 1.0).width;
            let affordable = state.player.gold >= item.price();
            let price_color = if affordable { Color::from_hex(0xf1c40f) } else { Color::from_hex(0xe74c3c) };
            draw_text(&price_text, box_x + box_w - pw - 14.0, ry + 22.0, 16.0, price_color);
        }
    }

    // Equestrian center upgrade (after animals)
    if !state.has_equestrian_center {
        let idx = items.len();
        let ry = list_y + idx as f32 * row_h;
        let selected = idx == state.animal_cursor;

        if selected {
            draw_rectangle(box_x + 4.0, ry, box_w - 8.0, row_h - 2.0, Color::from_hex(0x5a2a1a));
        }

        // Icon — small arena
        let ix = box_x + 14.0;
        let iy = ry + 6.0;
        draw_rectangle(ix, iy + 4.0, 24.0, 16.0, Color::from_hex(0xd4a050));
        draw_rectangle_lines(ix, iy + 4.0, 24.0, 16.0, 1.5, Color::from_hex(0x8b5e3c));
        draw_rectangle(ix + 4.0, iy + 8.0, 16.0, 8.0, Color::from_hex(0xc8a030));
        // Tiny fence posts
        for j in 0..5 {
            draw_rectangle(ix + 1.0 + j as f32 * 5.5, iy + 3.0, 2.0, 4.0, Color::from_hex(0x6b3a2a));
        }

        let name_color = if selected { WHITE } else { Color::from_hex(0xcccccc) };
        draw_text("Equestrian Center", box_x + 50.0, ry + 16.0, 14.0, name_color);
        let desc_color = Color::from_hex(0x999999);
        draw_text("Arena + Crossties", box_x + 50.0, ry + 30.0, 11.0, desc_color);

        let price_text = "10000g";
        let pw = measure_text(price_text, None, 16, 1.0).width;
        let affordable = state.player.gold >= 10000;
        let price_color = if affordable { Color::from_hex(0xf1c40f) } else { Color::from_hex(0xe74c3c) };
        draw_text(price_text, box_x + box_w - pw - 14.0, ry + 22.0, 16.0, price_color);
    }

    draw_text(
        "↑↓: Browse  |  E: Buy  |  Esc: Leave",
        box_x + 14.0, box_y + box_h - 12.0, 13.0, Color::from_hex(0xaaaaaa),
    );
}

fn draw_animal_icon(kind: AnimalKind, x: f32, y: f32) {
    match kind {
        AnimalKind::Chicken => {
            draw_circle(x + 14.0, y + 14.0, 8.0, WHITE);
            draw_circle(x + 20.0, y + 10.0, 5.0, WHITE);
            draw_circle(x + 20.0, y + 7.0, 3.0, Color::from_hex(0xe74c3c));
            draw_line(x + 23.0, y + 10.0, x + 26.0, y + 10.0, 2.0, Color::from_hex(0xf39c12));
        }
        AnimalKind::Cat => {
            draw_circle(x + 14.0, y + 14.0, 8.0, Color::from_hex(0xf39c12));
            draw_circle(x + 14.0, y + 8.0, 6.0, Color::from_hex(0xf39c12));
            // Ears
            draw_triangle(Vec2::new(x + 8.0, y + 8.0), Vec2::new(x + 12.0, y + 4.0), Vec2::new(x + 12.0, y + 10.0), Color::from_hex(0xf39c12));
            draw_triangle(Vec2::new(x + 16.0, y + 10.0), Vec2::new(x + 16.0, y + 4.0), Vec2::new(x + 20.0, y + 8.0), Color::from_hex(0xf39c12));
            draw_circle(x + 12.0, y + 7.0, 1.5, Color::from_hex(0x2ecc71));
            draw_circle(x + 16.0, y + 7.0, 1.5, Color::from_hex(0x2ecc71));
        }
        AnimalKind::Pig => {
            draw_circle(x + 14.0, y + 14.0, 10.0, Color::from_hex(0xf4a4b0));
            draw_circle(x + 14.0, y + 8.0, 6.0, Color::from_hex(0xf4a4b0));
            draw_circle(x + 14.0, y + 10.0, 4.0, Color::from_hex(0xe88a96));
            draw_circle(x + 12.0, y + 9.0, 1.0, Color::from_hex(0x333333));
            draw_circle(x + 16.0, y + 9.0, 1.0, Color::from_hex(0x333333));
        }
        AnimalKind::Sheep => {
            draw_circle(x + 14.0, y + 14.0, 10.0, Color::from_hex(0xf0ece0));
            draw_circle(x + 10.0, y + 12.0, 6.0, Color::from_hex(0xf0ece0));
            draw_circle(x + 18.0, y + 12.0, 6.0, Color::from_hex(0xf0ece0));
            draw_circle(x + 14.0, y + 8.0, 5.0, Color::from_hex(0x333333));
            draw_circle(x + 12.0, y + 7.0, 1.5, WHITE);
            draw_circle(x + 16.0, y + 7.0, 1.5, WHITE);
        }
        AnimalKind::Cow => {
            draw_circle(x + 14.0, y + 14.0, 10.0, WHITE);
            draw_circle(x + 10.0, y + 12.0, 4.0, Color::from_hex(0x333333));
            draw_circle(x + 18.0, y + 16.0, 3.0, Color::from_hex(0x333333));
            draw_circle(x + 14.0, y + 8.0, 5.0, WHITE);
            draw_circle(x + 12.0, y + 7.0, 1.5, Color::from_hex(0x333333));
            draw_circle(x + 16.0, y + 7.0, 1.5, Color::from_hex(0x333333));
        }
        AnimalKind::Horse => {
            let white = Color::from_hex(0xf0ece4);
            // Sleek body
            draw_rectangle(x + 4.0, y + 12.0, 16.0, 8.0, white);
            draw_circle(x + 4.0, y + 16.0, 4.0, white);
            draw_circle(x + 20.0, y + 16.0, 4.0, white);
            // Neck + head
            draw_line(x + 20.0, y + 12.0, x + 24.0, y + 6.0, 4.0, white);
            draw_rectangle(x + 22.0, y + 4.0, 8.0, 5.0, white);
            draw_circle(x + 24.0, y + 6.0, 1.5, Color::from_hex(0x333333));
            // Mane
            draw_line(x + 18.0, y + 6.0, x + 16.0, y + 4.0, 2.0, Color::from_hex(0xd0d0d0));
        }
    }
}

/// Draw owned animals and their pens on the farm overworld.
pub fn draw_farm_animals(state: &GameState, camera: &Camera) {
    // Draw pens first (behind animals)
    if state.owned_animals.contains(&AnimalKind::Chicken) {
        draw_chicken_coop(camera);
    }
    if state.owned_animals.contains(&AnimalKind::Pig) {
        draw_pig_pen(camera);
    }
    if state.owned_animals.contains(&AnimalKind::Sheep) {
        draw_sheep_pasture(camera);
    }
    if state.owned_animals.contains(&AnimalKind::Cow) {
        draw_cow_barn(camera);
    }
    if state.owned_animals.contains(&AnimalKind::Horse) {
        draw_horse_stable(camera);
    }

    // Equestrian center (riding arena + crossties)
    if state.has_equestrian_center {
        draw_equestrian_center(camera, &state.arena_jumps);
    }

    // Draw animals on top (skip horse if player is riding it)
    for animal in &state.owned_animals {
        if *animal == AnimalKind::Horse && state.riding_horse {
            continue; // horse is with the player
        }
        let (col, row) = animal.farm_tile();
        let (sx, sy) = camera.world_to_screen(col, row);
        if sx < -TILE_SIZE || sx > screen_width() + TILE_SIZE || sy < -TILE_SIZE || sy > screen_height() + TILE_SIZE {
            continue;
        }
        draw_farm_animal(*animal, sx, sy);
    }
}

// ── Pen drawings ─────────────────────────────────────────────────────────────

fn draw_fence_h(x: f32, y: f32, tiles: f32) {
    let w = tiles * TILE_SIZE;
    draw_rectangle(x, y, w, 3.0, Color::from_hex(0x8b5e3c));
    // Posts
    for i in 0..=(tiles as i32) {
        draw_rectangle(x + i as f32 * TILE_SIZE - 2.0, y - 4.0, 4.0, 10.0, Color::from_hex(0x6b3a2a));
    }
}

fn draw_fence_v(x: f32, y: f32, tiles: f32) {
    let h = tiles * TILE_SIZE;
    draw_rectangle(x, y, 3.0, h, Color::from_hex(0x8b5e3c));
    for i in 0..=(tiles as i32) {
        draw_rectangle(x - 2.0, y + i as f32 * TILE_SIZE - 2.0, 6.0, 4.0, Color::from_hex(0x6b3a2a));
    }
}

/// Chicken coop: cols 5-7, rows 14-16
fn draw_chicken_coop(camera: &Camera) {
    let (x, y) = camera.world_to_screen(5, 14);
    let w = 3.0 * TILE_SIZE;
    let h = 3.0 * TILE_SIZE;
    // Coop structure (top row)
    draw_rectangle(x, y, w, TILE_SIZE, Color::from_hex(0xd4a050));
    draw_rectangle(x + 2.0, y + 2.0, w - 4.0, TILE_SIZE - 4.0, Color::from_hex(0xc09040));
    // Tiny roof
    draw_triangle(
        Vec2::new(x - 4.0, y + TILE_SIZE),
        Vec2::new(x + w + 4.0, y + TILE_SIZE),
        Vec2::new(x + w / 2.0, y - 6.0),
        Color::from_hex(0xb03030),
    );
    draw_text("COOP", x + w / 2.0 - 16.0, y + 10.0, 11.0, WHITE);
    // Fence around bottom area
    draw_fence_h(x, y + h, 3.0);
    draw_fence_v(x, y + TILE_SIZE, 2.0);
    draw_fence_v(x + w, y + TILE_SIZE, 2.0);
    // Hay floor
    draw_rectangle(x + 2.0, y + TILE_SIZE + 2.0, w - 4.0, h - TILE_SIZE - 4.0, Color::from_hex(0xd4b060));
}

/// Pig pen: cols 9-12, rows 15-17
fn draw_pig_pen(camera: &Camera) {
    let (x, y) = camera.world_to_screen(9, 15);
    let w = 4.0 * TILE_SIZE;
    let h = 3.0 * TILE_SIZE;
    // Mud floor
    draw_rectangle(x + 2.0, y + 2.0, w - 4.0, h - 4.0, Color::from_hex(0x8a6a40));
    // Mud puddle
    draw_circle(x + w / 2.0 + 10.0, y + h / 2.0 + 8.0, 16.0, Color::from_hex(0x705030));
    // Fence
    draw_fence_h(x, y, 4.0);
    draw_fence_h(x, y + h, 4.0);
    draw_fence_v(x, y, 3.0);
    draw_fence_v(x + w, y, 3.0);
    // Trough
    draw_rectangle(x + 8.0, y + 6.0, 32.0, 10.0, Color::from_hex(0x6b3a2a));
    draw_rectangle(x + 10.0, y + 8.0, 28.0, 6.0, Color::from_hex(0xd4a030));
    // Sign
    draw_text("PIG PEN", x + w / 2.0 - 22.0, y - 4.0, 10.0, Color::from_hex(0x6b3a2a));
}

/// Sheep pasture: cols 14-18, rows 15-17
fn draw_sheep_pasture(camera: &Camera) {
    let (x, y) = camera.world_to_screen(14, 15);
    let w = 5.0 * TILE_SIZE;
    let h = 3.0 * TILE_SIZE;
    // Lush grass floor
    draw_rectangle(x + 2.0, y + 2.0, w - 4.0, h - 4.0, Color::from_hex(0x68a044));
    // Fence
    draw_fence_h(x, y, 5.0);
    draw_fence_h(x, y + h, 5.0);
    draw_fence_v(x, y, 3.0);
    draw_fence_v(x + w, y, 3.0);
    // Grass tufts
    for &(dx, dy) in &[(20.0, 30.0), (60.0, 50.0), (100.0, 20.0), (130.0, 60.0)] {
        draw_circle(x + dx, y + dy, 4.0, Color::from_hex(0x7ab854));
    }
    draw_text("PASTURE", x + w / 2.0 - 24.0, y - 4.0, 10.0, Color::from_hex(0x6b3a2a));
}

/// Cow barn: cols 5-8, rows 18-20
fn draw_cow_barn(camera: &Camera) {
    let (x, y) = camera.world_to_screen(5, 18);
    let w = 4.0 * TILE_SIZE;
    let h = 3.0 * TILE_SIZE;
    // Barn walls (top row is Farmhouse tile)
    draw_rectangle(x, y, w, TILE_SIZE, Color::from_hex(0xb03030));
    // Roof
    draw_triangle(
        Vec2::new(x - 4.0, y + TILE_SIZE),
        Vec2::new(x + w + 4.0, y + TILE_SIZE),
        Vec2::new(x + w / 2.0, y - 10.0),
        Color::from_hex(0x6a3a2a),
    );
    draw_text("BARN", x + w / 2.0 - 14.0, y + 8.0, 11.0, WHITE);
    // Fence around paddock
    draw_fence_h(x, y + h, 4.0);
    draw_fence_v(x, y + TILE_SIZE, 2.0);
    draw_fence_v(x + w, y + TILE_SIZE, 2.0);
    // Hay on floor
    draw_rectangle(x + 2.0, y + TILE_SIZE + 2.0, w - 4.0, h - TILE_SIZE - 4.0, Color::from_hex(0xd4b060));
    // Water bucket
    draw_rectangle(x + w - 20.0, y + h - 18.0, 14.0, 12.0, Color::from_hex(0x555555));
    draw_rectangle(x + w - 18.0, y + h - 14.0, 10.0, 8.0, Color::from_hex(0x4a9adf));
}

/// Horse stable: cols 10-14, rows 18-20
fn draw_horse_stable(camera: &Camera) {
    let (x, y) = camera.world_to_screen(10, 18);
    let w = 5.0 * TILE_SIZE;
    let h = 3.0 * TILE_SIZE;
    // Stable walls (top row is Farmhouse tile)
    draw_rectangle(x, y, w, TILE_SIZE, Color::from_hex(0x8b5e3c));
    draw_rectangle(x + 2.0, y + 2.0, w - 4.0, TILE_SIZE - 4.0, Color::from_hex(0xa0714a));
    // Roof
    draw_rectangle(x - 4.0, y - 4.0, w + 8.0, 8.0, Color::from_hex(0x6a3a2a));
    draw_text("STABLE", x + w / 2.0 - 20.0, y + 14.0, 11.0, WHITE);
    // Stall dividers
    draw_rectangle(x + w / 2.0 - 2.0, y + TILE_SIZE, 4.0, h - TILE_SIZE, Color::from_hex(0x6b3a2a));
    // Fence
    draw_fence_h(x, y + h, 5.0);
    draw_fence_v(x, y + TILE_SIZE, 2.0);
    draw_fence_v(x + w, y + TILE_SIZE, 2.0);
    // Hay floor
    draw_rectangle(x + 2.0, y + TILE_SIZE + 2.0, w - 4.0, h - TILE_SIZE - 4.0, Color::from_hex(0xd4b060));
    // Hay bale
    draw_rectangle(x + 8.0, y + h - 20.0, 20.0, 14.0, Color::from_hex(0xc8a030));
    draw_line(x + 18.0, y + h - 20.0, x + 18.0, y + h - 6.0, 1.0, Color::from_hex(0xa08020));
}

fn draw_farm_animal(kind: AnimalKind, x: f32, y: f32) {
    match kind {
        AnimalKind::Chicken => {
            // White body
            draw_circle(x + 16.0, y + 20.0, 8.0, WHITE);
            // Head
            draw_circle(x + 22.0, y + 14.0, 5.0, WHITE);
            // Comb
            draw_circle(x + 22.0, y + 10.0, 3.0, Color::from_hex(0xe74c3c));
            // Beak
            draw_line(x + 26.0, y + 14.0, x + 30.0, y + 14.0, 2.0, Color::from_hex(0xf39c12));
            // Eye
            draw_circle(x + 24.0, y + 13.0, 1.5, Color::from_hex(0x1a1a1a));
            // Legs
            draw_line(x + 14.0, y + 28.0, x + 12.0, y + 32.0, 1.5, Color::from_hex(0xf39c12));
            draw_line(x + 18.0, y + 28.0, x + 20.0, y + 32.0, 1.5, Color::from_hex(0xf39c12));
        }
        AnimalKind::Cat => {
            // Body
            draw_circle(x + 16.0, y + 20.0, 8.0, Color::from_hex(0xf39c12));
            // Head
            draw_circle(x + 16.0, y + 12.0, 6.0, Color::from_hex(0xf39c12));
            // Ears
            draw_triangle(Vec2::new(x + 10.0, y + 12.0), Vec2::new(x + 12.0, y + 4.0), Vec2::new(x + 14.0, y + 10.0), Color::from_hex(0xf39c12));
            draw_triangle(Vec2::new(x + 18.0, y + 10.0), Vec2::new(x + 20.0, y + 4.0), Vec2::new(x + 22.0, y + 12.0), Color::from_hex(0xf39c12));
            // Eyes
            draw_circle(x + 14.0, y + 11.0, 2.0, Color::from_hex(0x2ecc71));
            draw_circle(x + 18.0, y + 11.0, 2.0, Color::from_hex(0x2ecc71));
            draw_circle(x + 14.0, y + 11.0, 1.0, Color::from_hex(0x1a1a1a));
            draw_circle(x + 18.0, y + 11.0, 1.0, Color::from_hex(0x1a1a1a));
            // Tail
            draw_line(x + 8.0, y + 20.0, x + 2.0, y + 14.0, 2.5, Color::from_hex(0xf39c12));
        }
        AnimalKind::Pig => {
            // Body — big pink oval
            draw_circle(x + 16.0, y + 20.0, 11.0, Color::from_hex(0xf4a4b0));
            // Head
            draw_circle(x + 24.0, y + 16.0, 7.0, Color::from_hex(0xf4a4b0));
            // Snout
            draw_circle(x + 28.0, y + 17.0, 4.0, Color::from_hex(0xe88a96));
            draw_circle(x + 27.0, y + 16.0, 1.0, Color::from_hex(0x333333));
            draw_circle(x + 29.0, y + 16.0, 1.0, Color::from_hex(0x333333));
            // Eye
            draw_circle(x + 24.0, y + 14.0, 1.5, Color::from_hex(0x1a1a1a));
            // Ears
            draw_circle(x + 22.0, y + 11.0, 3.0, Color::from_hex(0xe88a96));
            // Curly tail
            draw_line(x + 5.0, y + 18.0, x + 2.0, y + 14.0, 2.0, Color::from_hex(0xf4a4b0));
            draw_line(x + 2.0, y + 14.0, x + 4.0, y + 12.0, 2.0, Color::from_hex(0xf4a4b0));
        }
        AnimalKind::Sheep => {
            // Fluffy body
            for &(dx, dy) in &[(12.0,18.0),(20.0,18.0),(16.0,14.0),(10.0,22.0),(22.0,22.0),(16.0,24.0)] {
                draw_circle(x + dx, y + dy, 6.0, Color::from_hex(0xf0ece0));
            }
            // Head — dark
            draw_circle(x + 16.0, y + 10.0, 5.0, Color::from_hex(0x444444));
            draw_circle(x + 14.0, y + 9.0, 1.5, WHITE);
            draw_circle(x + 18.0, y + 9.0, 1.5, WHITE);
            // Legs
            for &lx in &[12.0, 20.0] {
                draw_rectangle(x + lx - 1.0, y + 28.0, 3.0, 6.0, Color::from_hex(0x444444));
            }
        }
        AnimalKind::Cow => {
            // Body — white with spots
            draw_circle(x + 16.0, y + 20.0, 12.0, WHITE);
            draw_circle(x + 12.0, y + 18.0, 4.0, Color::from_hex(0x333333));
            draw_circle(x + 20.0, y + 22.0, 3.0, Color::from_hex(0x333333));
            // Head
            draw_circle(x + 26.0, y + 14.0, 6.0, WHITE);
            draw_circle(x + 28.0, y + 16.0, 3.0, Color::from_hex(0xf4a4b0));
            draw_circle(x + 26.0, y + 12.0, 1.5, Color::from_hex(0x1a1a1a));
            // Horns
            draw_line(x + 24.0, y + 9.0, x + 22.0, y + 6.0, 2.0, Color::from_hex(0xd4a050));
            draw_line(x + 28.0, y + 9.0, x + 30.0, y + 6.0, 2.0, Color::from_hex(0xd4a050));
            // Legs
            for &lx in &[10.0, 22.0] {
                draw_rectangle(x + lx - 1.0, y + 30.0, 3.0, 5.0, Color::from_hex(0x333333));
            }
        }
        AnimalKind::Horse => {
            let white = Color::from_hex(0xf0ece4);
            let silver = Color::from_hex(0xd0d0d0);
            // Body — elongated oval (rectangle + rounded ends)
            draw_rectangle(x + 6.0, y + 16.0, 20.0, 12.0, white);
            draw_circle(x + 6.0, y + 22.0, 6.0, white);
            draw_circle(x + 26.0, y + 22.0, 6.0, white);
            // Chest (front)
            draw_circle(x + 26.0, y + 18.0, 5.0, white);
            // Neck — angled upward
            draw_line(x + 26.0, y + 16.0, x + 30.0, y + 6.0, 6.0, white);
            // Head — elongated
            draw_rectangle(x + 27.0, y + 3.0, 10.0, 7.0, white);
            draw_circle(x + 37.0, y + 6.0, 3.5, white);
            // Nostril
            draw_circle(x + 37.0, y + 7.0, 1.0, Color::from_hex(0xcccccc));
            // Eye
            draw_circle(x + 33.0, y + 4.0, 1.5, Color::from_hex(0x1a1a1a));
            // Ear
            draw_rectangle(x + 29.0, y, 3.0, 5.0, white);
            // Silver mane — flowing
            for i in 0..5 {
                draw_line(x + 27.0 - i as f32 * 1.5, y + 4.0 + i as f32 * 3.0,
                          x + 24.0 - i as f32 * 1.5, y + 2.0 + i as f32 * 3.0,
                          2.0, silver);
            }
            // Legs — four slender legs
            for &lx in &[8.0, 14.0, 20.0, 25.0] {
                draw_rectangle(x + lx, y + 28.0, 2.5, 8.0, Color::from_hex(0xe0dcd4));
                // Hooves
                draw_rectangle(x + lx - 0.5, y + 35.0, 3.5, 2.0, Color::from_hex(0x888888));
            }
            // Tail — flowing
            draw_line(x + 4.0, y + 18.0, x + 0.0, y + 22.0, 2.5, silver);
            draw_line(x + 0.0, y + 22.0, x - 2.0, y + 28.0, 2.0, silver);
            draw_line(x + 1.0, y + 22.0, x + 2.0, y + 28.0, 1.5, silver);
        }
    }
}

/// Draw the equestrian center: riding arena + crossties area.
/// Placed at cols 20-30, rows 16-20 (east of the animal pens).
fn draw_equestrian_center(camera: &Camera, arena_jumps: &[(u8, u8, u8)]) {
    let fence_col = Color::from_hex(0x8b5e3c);
    let post_col = Color::from_hex(0x6b3a2a);
    let sand = Color::from_hex(0xd4b880);

    // ── Riding arena (cols 20-28, rows 16-19) ────────────────────────
    let (ax, ay) = camera.world_to_screen(20, 16);
    let aw = 9.0 * TILE_SIZE;
    let ah = 4.0 * TILE_SIZE;

    // Arena floor (sand/dirt)
    draw_rectangle(ax + 2.0, ay + 2.0, aw - 4.0, ah - 4.0, sand);
    // Track marks
    for i in 0..3 {
        let ry = ay + 20.0 + i as f32 * 36.0;
        draw_line(ax + 16.0, ry, ax + aw - 16.0, ry, 1.0, Color::from_hex(0xc0a060));
    }

    // Arena fence — all four sides
    // Top fence
    for i in 0..=9 {
        draw_rectangle(ax + i as f32 * TILE_SIZE - 2.0, ay - 4.0, 4.0, 10.0, post_col);
    }
    draw_rectangle(ax, ay, aw, 3.0, fence_col);
    draw_rectangle(ax, ay + 6.0, aw, 2.0, fence_col);
    // Bottom fence
    for i in 0..=9 {
        draw_rectangle(ax + i as f32 * TILE_SIZE - 2.0, ay + ah - 4.0, 4.0, 10.0, post_col);
    }
    draw_rectangle(ax, ay + ah - 2.0, aw, 3.0, fence_col);
    draw_rectangle(ax, ay + ah - 8.0, aw, 2.0, fence_col);
    // Left fence
    for i in 0..=4 {
        draw_rectangle(ax - 4.0, ay + i as f32 * TILE_SIZE - 2.0, 10.0, 4.0, post_col);
    }
    draw_rectangle(ax, ay, 3.0, ah, fence_col);
    draw_rectangle(ax + 6.0, ay, 2.0, ah, fence_col);
    // Right fence
    for i in 0..=4 {
        draw_rectangle(ax + aw - 4.0, ay + i as f32 * TILE_SIZE - 2.0, 10.0, 4.0, post_col);
    }
    draw_rectangle(ax + aw - 2.0, ay, 3.0, ah, fence_col);
    draw_rectangle(ax + aw - 8.0, ay, 2.0, ah, fence_col);

    // Arena gate (opening in bottom fence)
    draw_rectangle(ax + aw / 2.0 - 16.0, ay + ah - 8.0, 32.0, 10.0, sand);

    // Editor sign (right of gate)
    let sign_post_x = ax + aw + 8.0;
    let sign_post_y = ay + ah - 24.0;
    draw_rectangle(sign_post_x + 8.0, sign_post_y, 4.0, 28.0, Color::from_hex(0x6b3a2a));
    draw_rectangle(sign_post_x, sign_post_y - 4.0, 20.0, 14.0, Color::from_hex(0xf5f0e0));
    draw_rectangle_lines(sign_post_x, sign_post_y - 4.0, 20.0, 14.0, 1.0, Color::from_hex(0x8b5e3c));
    draw_text("E", sign_post_x + 6.0, sign_post_y + 7.0, 10.0, Color::from_hex(0x333333));

    // Sign
    let sign_x = ax + aw / 2.0 - 40.0;
    draw_rectangle(sign_x, ay - 18.0, 80.0, 14.0, Color::from_hex(0x6b3a2a));
    draw_text("RIDING ARENA", sign_x + 4.0, ay - 7.0, 11.0, Color::from_hex(0xf1c40f));

    // Jump obstacles from stored positions
    let jump_colors = [0xe74c3c, 0x2980b9, 0x27ae60, 0xf39c12, 0x8e44ad, 0x1abc9c];
    for (i, &(jc, jr, orient)) in arena_jumps.iter().enumerate() {
        let jx = ax + 10.0 + jc as f32 * TILE_SIZE;
        let jy = ay + 8.0 + jr as f32 * TILE_SIZE;
        let color = Color::from_hex(jump_colors[i % jump_colors.len()]);
        let white_post = Color::from_hex(0xeeeeee);
        match orient {
            0 => {
                // Horizontal
                draw_rectangle(jx, jy + 8.0, 4.0, 16.0, white_post);
                draw_rectangle(jx + 24.0, jy + 8.0, 4.0, 16.0, white_post);
                draw_rectangle(jx, jy + 10.0, 28.0, 3.0, color);
                draw_rectangle(jx, jy + 17.0, 28.0, 3.0, color);
                draw_rectangle(jx + 10.0, jy + 10.0, 8.0, 3.0, WHITE);
            }
            1 => {
                // Vertical
                draw_rectangle(jx + 4.0, jy, 16.0, 4.0, white_post);
                draw_rectangle(jx + 4.0, jy + 24.0, 16.0, 4.0, white_post);
                draw_rectangle(jx + 8.0, jy, 3.0, 28.0, color);
                draw_rectangle(jx + 15.0, jy, 3.0, 28.0, color);
                draw_rectangle(jx + 8.0, jy + 10.0, 3.0, 8.0, WHITE);
            }
            2 => {
                // Diagonal /
                draw_circle(jx + 2.0, jy + 26.0, 3.0, white_post);
                draw_circle(jx + 26.0, jy + 2.0, 3.0, white_post);
                draw_line(jx + 2.0, jy + 26.0, jx + 26.0, jy + 2.0, 3.5, color);
                draw_line(jx + 6.0, jy + 28.0, jx + 28.0, jy + 4.0, 3.0, color);
            }
            _ => {
                // Diagonal \
                draw_circle(jx + 2.0, jy + 2.0, 3.0, white_post);
                draw_circle(jx + 26.0, jy + 26.0, 3.0, white_post);
                draw_line(jx + 2.0, jy + 2.0, jx + 26.0, jy + 26.0, 3.5, color);
                draw_line(jx + 6.0, jy, jx + 28.0, jy + 24.0, 3.0, color);
            }
        }
    }

    // ── Crossties area (cols 20-23, rows 20-21) ──────────────────────
    let (cx, cy) = camera.world_to_screen(20, 20);
    let cw = 4.0 * TILE_SIZE;
    let ch = 2.0 * TILE_SIZE;

    // Floor
    draw_rectangle(cx, cy, cw, ch, Color::from_hex(0xc0b090));

    // Crosstie posts (vertical posts with chains between them)
    for i in 0..3 {
        let px = cx + 10.0 + i as f32 * (cw - 20.0) / 2.0;
        // Post
        draw_rectangle(px - 2.0, cy, 4.0, ch, Color::from_hex(0x555555));
        draw_circle(px, cy, 4.0, Color::from_hex(0x666666));
        // Cross chains between posts (horizontal lines at tie height)
        if i < 2 {
            let next_px = cx + 10.0 + (i + 1) as f32 * (cw - 20.0) / 2.0;
            draw_line(px + 2.0, cy + 16.0, next_px - 2.0, cy + 16.0, 1.5, Color::from_hex(0xaaaaaa));
            draw_line(px + 2.0, cy + 20.0, next_px - 2.0, cy + 20.0, 1.5, Color::from_hex(0xaaaaaa));
            // Chain droop (catenary)
            let mid = (px + next_px) / 2.0;
            draw_line(px + 2.0, cy + 16.0, mid, cy + 22.0, 1.0, Color::from_hex(0x999999));
            draw_line(mid, cy + 22.0, next_px - 2.0, cy + 16.0, 1.0, Color::from_hex(0x999999));
        }
    }
    // Rubber mat on floor
    draw_rectangle(cx + 20.0, cy + ch - 14.0, cw - 40.0, 10.0, Color::from_hex(0x333333));
    // Sign
    draw_text("CROSSTIES", cx + 14.0, cy - 4.0, 10.0, Color::from_hex(0x6b3a2a));
}
