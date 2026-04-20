use macroquad::prelude::*;
use crate::game::state::{FestivalKind, GamePhase, GameState};

pub fn draw(state: &GameState) {
    match state.phase {
        GamePhase::FestivalAnnounce => draw_announce(state),
        GamePhase::FestivalPlaying  => draw_playing(state),
        GamePhase::FestivalResults  => draw_results(state),
        _ => {}
    }
}

fn draw_announce(state: &GameState) {
    let sw = screen_width();
    let sh = screen_height();
    let kind = match state.festival_kind { Some(k) => k, None => return };

    draw_rectangle(0.0, 0.0, sw, sh, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.7 });

    let bw = 420.0;
    let bh = 220.0;
    let bx = sw / 2.0 - bw / 2.0;
    let by = sh / 2.0 - bh / 2.0;

    draw_rectangle(bx, by, bw, bh, Color::from_hex(0x1a2a4e));
    draw_rectangle_lines(bx, by, bw, bh, 3.0, Color::from_hex(0xf1c40f));

    // Festival banner
    let banner_color = match kind {
        FestivalKind::EggHunt        => 0x27ae60,
        FestivalKind::FishingDerby   => 0x2980b9,
        FestivalKind::MushroomForage => 0x8e44ad,
        FestivalKind::TreasureHunt   => 0xf39c12,
    };
    draw_rectangle(bx + 20.0, by + 16.0, bw - 40.0, 36.0, Color::from_hex(banner_color));

    let title = format!("Festival Day: {}!", kind.name());
    let tw = measure_text(&title, None, 24, 1.0).width;
    draw_text(&title, bx + bw / 2.0 - tw / 2.0, by + 42.0, 24.0, WHITE);

    // Description
    let desc = match kind {
        FestivalKind::EggHunt        => "Search the field for hidden eggs!",
        FestivalKind::FishingDerby   => "Find the best fishing spots!",
        FestivalKind::MushroomForage => "Hunt for rare mushrooms in the forest!",
        FestivalKind::TreasureHunt   => "Dig for buried winter treasures!",
    };
    let dw = measure_text(desc, None, 16, 1.0).width;
    draw_text(desc, bx + bw / 2.0 - dw / 2.0, by + 80.0, 16.0, Color::from_hex(0xdddddd));

    // Rules
    let rules = format!(
        "{} {} hidden  |  {} searches allowed  |  {}g per find",
        kind.hidden_count(), kind.item_name(), kind.max_searches(), kind.prize_per_item()
    );
    let rw = measure_text(&rules, None, 14, 1.0).width;
    draw_text(&rules, bx + bw / 2.0 - rw / 2.0, by + 116.0, 14.0, Color::from_hex(0xaaaacc));

    // Decorative icons
    draw_festival_icons(kind, bx + 30.0, by + 140.0, bw - 60.0);

    let hint = "Press E to start!";
    let hw = measure_text(hint, None, 16, 1.0).width;
    draw_text(hint, bx + bw / 2.0 - hw / 2.0, by + bh - 20.0, 16.0, Color::from_hex(0xf1c40f));
}

fn draw_playing(state: &GameState) {
    let sw = screen_width();
    let sh = screen_height();
    let kind = match state.festival_kind { Some(k) => k, None => return };

    draw_rectangle(0.0, 0.0, sw, sh, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.75 });

    let cols = if state.festival_grid.is_empty() { 8 } else { state.festival_grid[0].len() };
    let rows = state.festival_grid.len();
    let cell = 48.0f32;
    let grid_w = cols as f32 * cell;
    let grid_h = rows as f32 * cell;
    let gx = sw / 2.0 - grid_w / 2.0;
    let gy = sh / 2.0 - grid_h / 2.0 + 20.0;

    // Header
    let title = kind.name();
    let tw = measure_text(title, None, 22, 1.0).width;
    draw_text(title, sw / 2.0 - tw / 2.0, gy - 36.0, 22.0, Color::from_hex(0xf1c40f));

    let info = format!(
        "Found: {}/{}   Searches left: {}",
        state.festival_found, kind.hidden_count(), state.festival_searches_left
    );
    let iw = measure_text(&info, None, 16, 1.0).width;
    draw_text(&info, sw / 2.0 - iw / 2.0, gy - 12.0, 16.0, WHITE);

    // Grid background colors per festival
    let (bg_color, searched_color, item_color) = match kind {
        FestivalKind::EggHunt => (0x5a8a3c, 0x8ab86c, 0xf1c40f),      // green field, yellow eggs
        FestivalKind::FishingDerby => (0x1a6faa, 0x4a9fdf, 0xe74c3c),  // blue water, red fish
        FestivalKind::MushroomForage => (0x3a5a2a, 0x6a8a5a, 0x8e44ad),// dark forest, purple mushrooms
        FestivalKind::TreasureHunt => (0xd0d8e0, 0xf0f0f0, 0xf39c12),  // snow, gold treasures
    };

    for r in 0..rows {
        for c in 0..cols {
            let x = gx + c as f32 * cell;
            let y = gy + r as f32 * cell;
            let revealed = state.festival_revealed[r][c];
            let has_item = state.festival_grid[r][c];
            let is_cursor = (c, r) == state.festival_cursor;

            // Cell background
            let bg = if revealed {
                Color::from_hex(searched_color)
            } else {
                // Subtle variation
                let h = (c.wrapping_mul(7).wrapping_add(r.wrapping_mul(13))) % 3;
                let base = Color::from_hex(bg_color);
                if h == 0 {
                    Color { r: base.r + 0.03, g: base.g + 0.03, b: base.b + 0.03, a: 1.0 }
                } else {
                    base
                }
            };
            draw_rectangle(x, y, cell - 1.0, cell - 1.0, bg);

            // Revealed item
            if revealed && has_item {
                draw_festival_item(kind, x + cell / 2.0, y + cell / 2.0);
            } else if revealed {
                // Empty — show X
                draw_text("X", x + cell / 2.0 - 6.0, y + cell / 2.0 + 5.0, 16.0,
                          Color { r: 0.5, g: 0.5, b: 0.5, a: 0.4 });
            }

            // Cursor highlight
            if is_cursor {
                draw_rectangle_lines(x - 1.0, y - 1.0, cell + 1.0, cell + 1.0, 3.0, Color::from_hex(0xf1c40f));
            }
        }
    }

    // Footer
    let hint = "WASD: Move cursor  |  E: Search  |  Esc: Finish early";
    let hw = measure_text(hint, None, 13, 1.0).width;
    draw_text(hint, sw / 2.0 - hw / 2.0, gy + grid_h + 24.0, 13.0, Color::from_hex(0xaaaaaa));
}

fn draw_results(state: &GameState) {
    let sw = screen_width();
    let sh = screen_height();
    let kind = match state.festival_kind { Some(k) => k, None => return };

    draw_rectangle(0.0, 0.0, sw, sh, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.7 });

    let bw = 380.0;
    let bh = 200.0;
    let bx = sw / 2.0 - bw / 2.0;
    let by = sh / 2.0 - bh / 2.0;

    draw_rectangle(bx, by, bw, bh, Color::from_hex(0x1a2a4e));
    draw_rectangle_lines(bx, by, bw, bh, 3.0, Color::from_hex(0xf1c40f));

    let title = format!("{} Results", kind.name());
    let tw = measure_text(&title, None, 24, 1.0).width;
    draw_text(&title, bx + bw / 2.0 - tw / 2.0, by + 38.0, 24.0, Color::from_hex(0xf1c40f));

    let found = format!("You found {} {}!", state.festival_found, kind.item_name());
    let fw = measure_text(&found, None, 18, 1.0).width;
    draw_text(&found, bx + bw / 2.0 - fw / 2.0, by + 76.0, 18.0, WHITE);

    let prize = format!("Prize: {}g", state.festival_prize);
    let pw = measure_text(&prize, None, 20, 1.0).width;
    let prize_color = if state.festival_prize > 0 { Color::from_hex(0x2ecc71) } else { Color::from_hex(0xaaaaaa) };
    draw_text(&prize, bx + bw / 2.0 - pw / 2.0, by + 112.0, 20.0, prize_color);

    if state.festival_found == kind.hidden_count() as u32 {
        let perfect = "PERFECT SCORE!";
        let pw2 = measure_text(perfect, None, 18, 1.0).width;
        draw_text(perfect, bx + bw / 2.0 - pw2 / 2.0, by + 142.0, 18.0, Color::from_hex(0xf1c40f));
    }

    let hint = "Press E to continue";
    let hw = measure_text(hint, None, 14, 1.0).width;
    draw_text(hint, bx + bw / 2.0 - hw / 2.0, by + bh - 18.0, 14.0, Color::from_hex(0x888888));
}

fn draw_festival_item(kind: FestivalKind, cx: f32, cy: f32) {
    match kind {
        FestivalKind::EggHunt => {
            // Colored egg
            draw_circle(cx, cy, 10.0, Color::from_hex(0xf1c40f));
            draw_circle(cx, cy - 2.0, 8.0, Color::from_hex(0xff6b6b));
            draw_circle(cx - 2.0, cy + 2.0, 3.0, Color::from_hex(0x3498db));
        }
        FestivalKind::FishingDerby => {
            // Fish
            draw_circle(cx, cy, 8.0, Color::from_hex(0xe74c3c));
            draw_triangle(
                Vec2::new(cx + 8.0, cy),
                Vec2::new(cx + 16.0, cy - 6.0),
                Vec2::new(cx + 16.0, cy + 6.0),
                Color::from_hex(0xc0392b),
            );
            draw_circle(cx - 3.0, cy - 2.0, 2.0, WHITE);
        }
        FestivalKind::MushroomForage => {
            // Mushroom
            draw_rectangle(cx - 3.0, cy, 6.0, 10.0, Color::from_hex(0xf5f5dc));
            draw_circle(cx, cy - 2.0, 10.0, Color::from_hex(0x8e44ad));
            draw_circle(cx - 4.0, cy - 4.0, 3.0, WHITE);
            draw_circle(cx + 5.0, cy - 1.0, 2.0, WHITE);
        }
        FestivalKind::TreasureHunt => {
            // Treasure chest
            draw_rectangle(cx - 10.0, cy - 4.0, 20.0, 14.0, Color::from_hex(0xd4a017));
            draw_rectangle(cx - 10.0, cy - 7.0, 20.0, 5.0, Color::from_hex(0xe8b828));
            draw_circle(cx, cy + 2.0, 3.0, Color::from_hex(0xf1c40f));
        }
    }
}

fn draw_festival_icons(kind: FestivalKind, x: f32, y: f32, w: f32) {
    let count = 5;
    let spacing = w / count as f32;
    for i in 0..count {
        draw_festival_item(kind, x + i as f32 * spacing + spacing / 2.0, y + 12.0);
    }
}
