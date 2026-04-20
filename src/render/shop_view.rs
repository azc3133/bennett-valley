use macroquad::prelude::*;
use crate::game::state::GameState;

pub fn draw(state: &GameState) {
    let sw = screen_width();
    let sh = screen_height();
    let box_w = 340.0;
    let box_x = sw / 2.0 - box_w / 2.0;
    let box_y = sh / 2.0 - 180.0;
    let box_h = 360.0;

    draw_rectangle(box_x, box_y, box_w, box_h, Color::from_hex(0x1a1a2e));
    draw_rectangle_lines(box_x, box_y, box_w, box_h, 2.0, WHITE);
    draw_text("Shop", box_x + 130.0, box_y + 28.0, 24.0, Color::from_hex(0xf1c40f));
    draw_text(
        &format!("Gold: {}g", state.player.gold),
        box_x + 10.0, box_y + 54.0, 18.0, WHITE,
    );

    let names = state.shop_sorted_names();
    let mut y = box_y + 86.0;
    for (i, name) in names.iter().enumerate() {
        let selected = i == state.shop_cursor;
        if selected {
            draw_rectangle(box_x + 6.0, y - 14.0, box_w - 12.0, 22.0, Color::from_hex(0x2a3a5e));
        }
        let item = state.shop.items.get(name).unwrap();
        let qty = {
            use crate::game::inventory::ItemKind;
            // Derive seed variant from shop name via state helper
            if let Some(seed) = crate::game::state::shop_name_to_seed_pub(name) {
                state.player.inventory.count(&ItemKind::Seed(seed))
            } else {
                state.player.inventory.count(&ItemKind::Pendant)
            }
        };
        let label = if crate::game::state::shop_name_to_seed_pub(name).is_some() {
            format!("{}  {}g/seed  sell:{}g  [{}]", name, item.buy_price, item.sell_price, qty)
        } else {
            format!("{}  {}g  [{}]", name, item.buy_price, qty)
        };
        let color = if selected { Color::from_hex(0xf1c40f) } else { WHITE };
        draw_text(&label, box_x + 12.0, y, 16.0, color);
        y += 28.0;
    }

    draw_text(
        "Up/Down: select  |  E: Buy 1  |  Esc: Close",
        box_x + 14.0, box_y + box_h - 16.0, 13.0, Color::from_hex(0xaaaaaa),
    );
}
