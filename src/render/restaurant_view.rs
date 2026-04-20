use macroquad::prelude::*;
use crate::game::state::GameState;

pub fn draw(state: &GameState) {
    let sw = screen_width();
    let sh = screen_height();

    let menu = GameState::MENU;
    let row_h = 38.0;
    let header_h = 70.0;
    let footer_h = 40.0;
    let box_h = (header_h + menu.len() as f32 * row_h + footer_h).min(sh - 40.0);
    let box_w = 420.0f32.min(sw - 40.0);
    let box_x = sw / 2.0 - box_w / 2.0;
    let box_y = sh / 2.0 - box_h / 2.0;

    // Warm background
    draw_rectangle(box_x, box_y, box_w, box_h, Color::from_hex(0x3a1a0a));
    draw_rectangle_lines(box_x, box_y, box_w, box_h, 2.0, Color::from_hex(0xc04020));

    // Italian flag stripe at top
    let stripe_w = box_w / 3.0;
    draw_rectangle(box_x, box_y, stripe_w, 4.0, Color::from_hex(0x009246));
    draw_rectangle(box_x + stripe_w, box_y, stripe_w, 4.0, WHITE);
    draw_rectangle(box_x + stripe_w * 2.0, box_y, stripe_w, 4.0, Color::from_hex(0xce2b37));

    // Title
    let title = "Ristorante Bella Valle";
    let tw = measure_text(title, None, 22, 1.0).width;
    draw_text(title, box_x + box_w / 2.0 - tw / 2.0, box_y + 30.0, 22.0, Color::from_hex(0xf1c40f));

    // Subtitle
    let sub = "Buon appetito!";
    let sw2 = measure_text(sub, None, 13, 1.0).width;
    draw_text(sub, box_x + box_w / 2.0 - sw2 / 2.0, box_y + 46.0, 13.0, Color::from_hex(0xddaa88));

    // Gold
    let gold_text = format!("Gold: {}g", state.player.gold);
    let gw = measure_text(&gold_text, None, 14, 1.0).width;
    draw_text(&gold_text, box_x + box_w - gw - 14.0, box_y + 60.0, 14.0, Color::from_hex(0xf1c40f));

    // Energy
    let energy_text = format!("Energy: {}/{}", state.player.energy, state.player.max_energy);
    draw_text(&energy_text, box_x + 14.0, box_y + 60.0, 14.0, Color::from_hex(0x2ecc71));

    let list_y = box_y + header_h;
    let rainbow = state.rainbow_day;

    for (i, &(name, price, energy)) in menu.iter().enumerate() {
        let ry = list_y + i as f32 * row_h;
        let selected = i == state.restaurant_cursor;

        if selected {
            draw_rectangle(box_x + 4.0, ry, box_w - 8.0, row_h - 2.0, Color::from_hex(0x5a2a1a));
        }

        // Food icon
        let ix = box_x + 14.0;
        let iy = ry + 8.0;
        draw_food_icon(i, ix, iy);

        // Name
        let name_color = if selected { WHITE } else { Color::from_hex(0xddccaa) };
        draw_text(name, box_x + 46.0, ry + 18.0, 15.0, name_color);

        // Energy restore
        let energy_text = format!("+{} energy", energy);
        draw_text(&energy_text, box_x + 46.0, ry + 32.0, 11.0, Color::from_hex(0x2ecc71));

        // Price
        let eff_price = if rainbow { price / 2 } else { price };
        let price_text = format!("{}g", eff_price);
        let pw = measure_text(&price_text, None, 15, 1.0).width;
        let affordable = state.player.gold >= eff_price;
        let price_color = if affordable { Color::from_hex(0xf1c40f) } else { Color::from_hex(0xe74c3c) };
        draw_text(&price_text, box_x + box_w - pw - 14.0, ry + 22.0, 15.0, price_color);
        if rainbow {
            let orig = format!("{}g", price);
            let ow = measure_text(&orig, None, 11, 1.0).width;
            draw_text(&orig, box_x + box_w - ow - 14.0, ry + 34.0, 11.0, Color::from_hex(0x888888));
        }
    }

    draw_text(
        "↑↓: Browse  |  E: Order  |  Esc: Leave",
        box_x + 14.0, box_y + box_h - 12.0, 13.0, Color::from_hex(0xaaaaaa),
    );
}

fn draw_food_icon(idx: usize, x: f32, y: f32) {
    match idx {
        0 => { // Pizza
            draw_circle(x + 12.0, y + 12.0, 10.0, Color::from_hex(0xf1c40f));
            draw_circle(x + 12.0, y + 12.0, 8.0, Color::from_hex(0xe74c3c));
            draw_circle(x + 10.0, y + 10.0, 2.0, Color::from_hex(0xf1c40f));
            draw_circle(x + 15.0, y + 13.0, 2.0, Color::from_hex(0xf1c40f));
        }
        1 => { // Spaghetti
            draw_rectangle(x + 4.0, y + 6.0, 16.0, 14.0, Color::from_hex(0xf5deb3));
            draw_circle(x + 12.0, y + 8.0, 4.0, Color::from_hex(0xc04020));
        }
        2 => { // Lasagna
            draw_rectangle(x + 4.0, y + 6.0, 18.0, 14.0, Color::from_hex(0xd4a050));
            draw_rectangle(x + 6.0, y + 10.0, 14.0, 4.0, Color::from_hex(0xc04020));
            draw_rectangle(x + 6.0, y + 14.0, 14.0, 3.0, Color::from_hex(0xf5deb3));
        }
        3 => { // Risotto
            draw_circle(x + 12.0, y + 12.0, 10.0, Color::from_hex(0xddddcc));
            draw_circle(x + 12.0, y + 12.0, 7.0, Color::from_hex(0xf5f0d0));
            draw_circle(x + 10.0, y + 11.0, 2.0, Color::from_hex(0x27ae60));
        }
        4 => { // Tiramisu
            draw_rectangle(x + 4.0, y + 8.0, 16.0, 12.0, Color::from_hex(0x8b5e3c));
            draw_rectangle(x + 6.0, y + 8.0, 12.0, 4.0, Color::from_hex(0xf5f0e0));
            draw_circle(x + 8.0, y + 10.0, 1.5, Color::from_hex(0x4a3020));
        }
        5 => { // Bruschetta
            draw_rectangle(x + 4.0, y + 10.0, 18.0, 8.0, Color::from_hex(0xd4a050));
            draw_circle(x + 10.0, y + 10.0, 3.0, Color::from_hex(0xe74c3c));
            draw_circle(x + 16.0, y + 10.0, 2.0, Color::from_hex(0x27ae60));
        }
        6 => { // Ravioli
            draw_circle(x + 8.0, y + 10.0, 5.0, Color::from_hex(0xf5deb3));
            draw_circle(x + 16.0, y + 14.0, 5.0, Color::from_hex(0xf5deb3));
            draw_circle(x + 12.0, y + 8.0, 3.0, Color::from_hex(0xc04020));
        }
        _ => { // Gelato
            draw_circle(x + 12.0, y + 8.0, 6.0, Color::from_hex(0xffb6c1));
            draw_circle(x + 12.0, y + 8.0, 4.0, Color::from_hex(0xf5f0d0));
            draw_triangle(
                Vec2::new(x + 6.0, y + 12.0),
                Vec2::new(x + 18.0, y + 12.0),
                Vec2::new(x + 12.0, y + 22.0),
                Color::from_hex(0xd4a050),
            );
        }
    }
}
