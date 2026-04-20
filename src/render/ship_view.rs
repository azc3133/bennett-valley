use macroquad::prelude::*;
use crate::game::inventory::ItemKind;
use crate::game::state::GameState;

pub fn draw(state: &GameState) {
    let sw = screen_width();
    let sh = screen_height();

    let row_h = 26.0;
    let header_h = 60.0;
    let footer_h = 50.0;
    let manifest_len = state.ship_manifest.len();
    // +1 for the "Ship Selected" button row
    let content_h = (manifest_len + 1) as f32 * row_h;
    let box_h = (header_h + content_h + footer_h).min(sh - 40.0);
    let box_w = 380.0f32.min(sw - 40.0);
    let box_x = sw / 2.0 - box_w / 2.0;
    let box_y = sh / 2.0 - box_h / 2.0;

    // Background
    draw_rectangle(box_x, box_y, box_w, box_h, Color::from_hex(0x1a1a2e));
    draw_rectangle_lines(box_x, box_y, box_w, box_h, 2.0, WHITE);

    // Title
    draw_text("Shipping Box", box_x + box_w / 2.0 - 60.0, box_y + 28.0, 22.0, Color::from_hex(0xf1c40f));

    // Total value of selected items
    let total_value: u32 = state.ship_manifest.iter()
        .filter(|(_, _, _, sel)| *sel)
        .map(|(_, qty, price, _)| qty * price)
        .sum();
    let total_text = format!("Total: {}g", total_value);
    let tw = measure_text(&total_text, None, 16, 1.0).width;
    draw_text(&total_text, box_x + box_w - tw - 14.0, box_y + 50.0, 16.0, Color::from_hex(0x2ecc71));

    // Items list
    let list_y = box_y + header_h;
    let max_visible = ((box_h - header_h - footer_h) / row_h) as usize;

    // Scroll offset: keep cursor visible
    let total_rows = manifest_len + 1;
    let scroll_offset = if state.ship_cursor >= max_visible {
        (state.ship_cursor + 1).saturating_sub(max_visible)
    } else {
        0
    };

    for vi in 0..max_visible.min(total_rows) {
        let idx = scroll_offset + vi;
        if idx >= total_rows { break; }
        let ry = list_y + vi as f32 * row_h;
        let selected_row = idx == state.ship_cursor;

        if idx < manifest_len {
            // Item row
            let (ref item, qty, price, checked) = state.ship_manifest[idx];

            if selected_row {
                draw_rectangle(box_x + 4.0, ry - 2.0, box_w - 8.0, row_h - 2.0, Color::from_hex(0x2a3a5e));
            }

            // Checkbox
            let check_x = box_x + 12.0;
            let check_y = ry + 4.0;
            draw_rectangle_lines(check_x, check_y, 14.0, 14.0, 1.0, Color::from_hex(0x888888));
            if checked {
                draw_text("✓", check_x + 1.0, check_y + 12.0, 14.0, Color::from_hex(0x2ecc71));
            }

            // Item name
            let label = item_label(item);
            let name_color = if selected_row { WHITE } else { Color::from_hex(0xcccccc) };
            draw_text(&label, box_x + 32.0, ry + 16.0, 15.0, name_color);

            // Quantity
            let qty_text = format!("x{}", qty);
            draw_text(&qty_text, box_x + 200.0, ry + 16.0, 15.0, Color::from_hex(0xaaaaaa));

            // Price
            let price_text = format!("{}g", qty * price);
            let pw = measure_text(&price_text, None, 15, 1.0).width;
            let price_color = if checked { Color::from_hex(0xf1c40f) } else { Color::from_hex(0x666666) };
            draw_text(&price_text, box_x + box_w - pw - 14.0, ry + 16.0, 15.0, price_color);
        } else {
            // "Ship Selected" button row
            if selected_row {
                draw_rectangle(box_x + 4.0, ry - 2.0, box_w - 8.0, row_h - 2.0, Color::from_hex(0x1a4a1a));
            }
            let btn = "▶ Ship Selected";
            let bw = measure_text(btn, None, 18, 1.0).width;
            let btn_color = if selected_row { Color::from_hex(0x2ecc71) } else { Color::from_hex(0x44aa44) };
            draw_text(btn, box_x + box_w / 2.0 - bw / 2.0, ry + 16.0, 18.0, btn_color);
        }
    }

    // Footer hints
    draw_text(
        "↑↓: Select  |  E: Toggle/Ship  |  Esc: Cancel",
        box_x + 14.0, box_y + box_h - 14.0, 13.0, Color::from_hex(0xaaaaaa),
    );
}

fn item_label(item: &ItemKind) -> String {
    match item {
        ItemKind::Seed(s) => format!("{} seed", s.name()),
        ItemKind::Crop(c) => c.name().to_string(),
        ItemKind::Forage(f) => f.name().to_string(),
        ItemKind::Fish(f) => f.name().to_string(),
        ItemKind::Ore(o) => o.name().to_string(),
        ItemKind::Pendant => "pendant".to_string(),
        ItemKind::Egg => "egg".to_string(),
        ItemKind::Milk => "milk".to_string(),
        ItemKind::Fiber => "fiber".to_string(),
    }
}
