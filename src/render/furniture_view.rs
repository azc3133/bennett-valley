use macroquad::prelude::*;
use crate::game::state::{FurnitureKind, GameState};

pub fn draw(state: &GameState) {
    let sw = screen_width();
    let sh = screen_height();

    let row_h = 32.0;
    let header_h = 60.0;
    let footer_h = 40.0;
    let items = FurnitureKind::ALL;
    let box_h = (header_h + items.len() as f32 * row_h + footer_h).min(sh - 40.0);
    let box_w = 400.0f32.min(sw - 40.0);
    let box_x = sw / 2.0 - box_w / 2.0;
    let box_y = sh / 2.0 - box_h / 2.0;

    // Background
    draw_rectangle(box_x, box_y, box_w, box_h, Color::from_hex(0x2a1a3e));
    draw_rectangle_lines(box_x, box_y, box_w, box_h, 2.0, Color::from_hex(0x6a4c93));

    // Title
    let title = "Furniture Shop";
    let tw = measure_text(title, None, 22, 1.0).width;
    draw_text(title, box_x + box_w / 2.0 - tw / 2.0, box_y + 28.0, 22.0, Color::from_hex(0xf1c40f));

    // Gold
    let gold_text = format!("Gold: {}g", state.player.gold);
    let gw = measure_text(&gold_text, None, 16, 1.0).width;
    draw_text(&gold_text, box_x + box_w - gw - 14.0, box_y + 50.0, 16.0, Color::from_hex(0xf1c40f));

    // Items
    let list_y = box_y + header_h;
    for (i, item) in items.iter().enumerate() {
        let ry = list_y + i as f32 * row_h;
        let selected = i == state.furniture_cursor;
        let owned = state.owned_furniture.contains(item);

        if selected {
            draw_rectangle(box_x + 4.0, ry, box_w - 8.0, row_h - 2.0, Color::from_hex(0x3a2a5e));
        }

        // Icon
        let ix = box_x + 14.0;
        let iy = ry + 4.0;
        draw_furniture_icon(*item, ix, iy);

        // Name
        let name_color = if owned { Color::from_hex(0x888888) } else if selected { WHITE } else { Color::from_hex(0xcccccc) };
        draw_text(item.name(), box_x + 46.0, ry + 20.0, 16.0, name_color);

        // Price or "Owned"
        if owned {
            let ow = measure_text("Owned", None, 14, 1.0).width;
            draw_text("Owned", box_x + box_w - ow - 14.0, ry + 20.0, 14.0, Color::from_hex(0x27ae60));
        } else {
            let price_text = format!("{}g", item.price());
            let pw = measure_text(&price_text, None, 16, 1.0).width;
            let affordable = state.player.gold >= item.price();
            let price_color = if affordable { Color::from_hex(0xf1c40f) } else { Color::from_hex(0xe74c3c) };
            draw_text(&price_text, box_x + box_w - pw - 14.0, ry + 20.0, 16.0, price_color);
        }
    }

    // Footer
    draw_text(
        "↑↓: Browse  |  E: Buy  |  Esc: Leave",
        box_x + 14.0, box_y + box_h - 12.0, 13.0, Color::from_hex(0xaaaaaa),
    );
}

fn draw_furniture_icon(kind: FurnitureKind, x: f32, y: f32) {
    match kind {
        FurnitureKind::TV => {
            draw_rectangle(x, y + 2.0, 24.0, 16.0, Color::from_hex(0x333333));
            draw_rectangle(x + 2.0, y + 4.0, 20.0, 12.0, Color::from_hex(0x4a9adf));
            draw_rectangle(x + 8.0, y + 18.0, 8.0, 4.0, Color::from_hex(0x555555));
        }
        FurnitureKind::Couch => {
            draw_rectangle(x, y + 10.0, 24.0, 12.0, Color::from_hex(0x8b4513));
            draw_rectangle(x + 2.0, y + 6.0, 20.0, 6.0, Color::from_hex(0xa0522d));
        }
        FurnitureKind::Lamp => {
            draw_rectangle(x + 10.0, y + 8.0, 4.0, 14.0, Color::from_hex(0x888888));
            draw_triangle(
                Vec2::new(x + 4.0, y + 10.0),
                Vec2::new(x + 20.0, y + 10.0),
                Vec2::new(x + 12.0, y + 2.0),
                Color::from_hex(0xf39c12),
            );
        }
        FurnitureKind::FishTank => {
            draw_rectangle(x, y + 4.0, 24.0, 18.0, Color::from_hex(0x1a6faa));
            draw_rectangle_lines(x, y + 4.0, 24.0, 18.0, 2.0, Color::from_hex(0x888888));
            draw_circle(x + 8.0, y + 14.0, 3.0, Color::from_hex(0xe74c3c));
            draw_circle(x + 18.0, y + 10.0, 2.0, Color::from_hex(0xf39c12));
        }
        FurnitureKind::Rug => {
            draw_rectangle(x + 2.0, y + 8.0, 20.0, 14.0, Color::from_hex(0xc0392b));
            draw_rectangle(x + 6.0, y + 12.0, 12.0, 6.0, Color::from_hex(0xe74c3c));
        }
        FurnitureKind::PottedPlant => {
            draw_rectangle(x + 6.0, y + 14.0, 12.0, 8.0, Color::from_hex(0xd4a050));
            draw_circle(x + 12.0, y + 10.0, 6.0, Color::from_hex(0x27ae60));
            draw_circle(x + 8.0, y + 8.0, 4.0, Color::from_hex(0x2ecc71));
        }
    }
}
