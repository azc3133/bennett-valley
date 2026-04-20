use macroquad::prelude::*;
use crate::game::npc::NPC;
use std::collections::HashMap;

const MAX_HEARTS: u8 = 10; // 250 friendship / 25

pub fn draw(npcs: &[NPC]) {
    draw_labeled(npcs, None, "Relationships");
}

/// Draw relationships overlay. If `p2_friendships` is Some, show P2's friendship
/// values instead of the NPC's own friendship field.
pub fn draw_labeled(npcs: &[NPC], p2_friendships: Option<&HashMap<u8, u8>>, title_text: &str) {
    let sw = screen_width();
    let sh = screen_height();

    // Semi-transparent full-screen backdrop
    draw_rectangle(0.0, 0.0, sw, sh, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.82 });

    let panel_x = 30.0;
    let panel_y = 36.0;
    let panel_w = sw - 60.0;
    let panel_h = sh - 60.0;

    draw_rectangle(panel_x, panel_y, panel_w, panel_h, Color::from_hex(0x12122a));
    draw_rectangle_lines(panel_x, panel_y, panel_w, panel_h, 2.0, Color::from_hex(0x4444aa));

    // Title
    let tw = measure_text(title_text, None, 22, 1.0).width;
    draw_text(title_text, panel_x + panel_w / 2.0 - tw / 2.0, panel_y + 26.0, 22.0, Color::from_hex(0xf1c40f));

    // Close hint
    draw_text("I: Close", panel_x + panel_w - 70.0, panel_y + 26.0, 13.0, Color::from_hex(0x666688));

    // Column layout: 3 columns
    let cols = 3;
    let rows_per_col = (npcs.len() + cols - 1) / cols;
    let col_w = (panel_w - 20.0) / cols as f32;
    let row_h = 22.0;
    let start_y = panel_y + 42.0;

    for (i, npc) in npcs.iter().enumerate() {
        let col = i / rows_per_col;
        let row = i % rows_per_col;
        let x = panel_x + 10.0 + col as f32 * col_w;
        let y = start_y + row as f32 * row_h;

        let friendship = match p2_friendships {
            Some(map) => *map.get(&npc.id).unwrap_or(&0),
            None => npc.friendship,
        };
        let hearts = (friendship / 25).min(MAX_HEARTS);
        let cap_hearts = if npc.loved_gift_given { 8u8 } else { 4u8 };

        // Name
        let name_color = if npc.marriageable {
            Color::from_hex(0xff88cc)
        } else {
            WHITE
        };
        draw_text(&npc.name, x, y + 14.0, 13.0, name_color);

        // Hearts
        let hx_start = x + 56.0;
        for h in 0..MAX_HEARTS {
            let hx = hx_start + h as f32 * 13.0;
            let color = if h < hearts {
                Color::from_hex(0xe74c3c) // filled — earned
            } else if h < cap_hearts {
                Color::from_hex(0x552222) // reachable but not yet earned
            } else {
                Color::from_hex(0x333333) // locked (needs loved gift)
            };
            draw_heart(hx, y + 4.0, color);
        }

        // Gifted-today indicator
        if npc.gifted_today {
            draw_text("✓", hx_start + MAX_HEARTS as f32 * 13.0 + 2.0, y + 14.0, 12.0,
                      Color::from_hex(0x2ecc71));
        }

        // Loved gift hint (greyed, small)
        let loved = npc.gift_preferences.iter()
            .filter(|(_, &v)| v >= 2)
            .max_by_key(|(_, &v)| v)
            .map(|(k, _)| k.as_str())
            .unwrap_or("");
        if !loved.is_empty() {
            let lx = hx_start + MAX_HEARTS as f32 * 13.0 + 16.0;
            let color = if npc.loved_gift_given {
                Color::from_hex(0x2ecc71)
            } else {
                Color::from_hex(0x666666)
            };
            draw_text(loved, lx, y + 14.0, 11.0, color);
        }
    }

    // Legend
    let ly = panel_y + panel_h - 18.0;
    draw_heart(panel_x + 12.0, ly - 10.0, Color::from_hex(0xe74c3c));
    draw_text("= earned", panel_x + 28.0, ly, 11.0, Color::from_hex(0x888888));
    draw_heart(panel_x + 100.0, ly - 10.0, Color::from_hex(0x552222));
    draw_text("= reachable", panel_x + 116.0, ly, 11.0, Color::from_hex(0x888888));
    draw_heart(panel_x + 200.0, ly - 10.0, Color::from_hex(0x333333));
    draw_text("= locked (give loved gift)", panel_x + 216.0, ly, 11.0, Color::from_hex(0x888888));
    draw_text("Pink = marriageable  |  ✓ = gifted today  |  green gift name = loved gift given",
              panel_x + 380.0, ly, 11.0, Color::from_hex(0x666688));
}

fn draw_heart(x: f32, y: f32, color: Color) {
    draw_circle(x + 3.0, y + 3.5, 3.5, color);
    draw_circle(x + 8.0, y + 3.5, 3.5, color);
    draw_triangle(
        Vec2::new(x, y + 5.0),
        Vec2::new(x + 11.0, y + 5.0),
        Vec2::new(x + 5.5, y + 11.0),
        color,
    );
}
