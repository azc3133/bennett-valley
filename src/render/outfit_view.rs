use macroquad::prelude::*;
use crate::game::state::{GameState, OUTFITS};
use crate::render::player_view::{draw_character, draw_hairstyle, player_hair_color};

pub fn draw(state: &GameState) {
    let sw = screen_width();
    let sh = screen_height();

    let row_h = 40.0;
    let header_h = 60.0;
    let footer_h = 40.0;
    let total = OUTFITS.len();
    let box_h = (header_h + total as f32 * row_h + footer_h).min(sh - 40.0);
    let box_w = 440.0f32.min(sw - 40.0);
    let box_x = sw / 2.0 - box_w / 2.0;
    let box_y = sh / 2.0 - box_h / 2.0;

    draw_rectangle(box_x, box_y, box_w, box_h, Color::from_hex(0x1a1a3e));
    draw_rectangle_lines(box_x, box_y, box_w, box_h, 2.0, Color::from_hex(0x8844aa));

    let title = "Wardrobe";
    let tw = measure_text(title, None, 22, 1.0).width;
    draw_text(title, box_x + box_w / 2.0 - tw / 2.0, box_y + 28.0, 22.0, Color::from_hex(0xf1c40f));

    let gold_text = format!("Gold: {}g", state.player.gold);
    let gw = measure_text(&gold_text, None, 16, 1.0).width;
    draw_text(&gold_text, box_x + box_w - gw - 14.0, box_y + 50.0, 16.0, Color::from_hex(0xf1c40f));

    // Gender display
    let gender_label = if state.player.gender == 0 { "♂ Male" } else { "♀ Female" };
    draw_text(gender_label, box_x + 14.0, box_y + 50.0, 14.0, Color::from_hex(0xcccccc));

    // Hair info (female only)
    if state.player.gender == 1 {
        use crate::game::state::{HAIRSTYLES, HAIR_COLORS};
        let style_name = HAIRSTYLES[state.player.hairstyle.min(HAIRSTYLES.len() as u8 - 1) as usize];
        let color_name = HAIR_COLORS[state.player.hair_color.min(HAIR_COLORS.len() as u8 - 1) as usize].3;
        let hair_text = format!("Hair: {} {}", color_name, style_name);
        let htw = measure_text(&hair_text, None, 12, 1.0).width;
        draw_text(&hair_text, box_x + box_w / 2.0 - htw / 2.0, box_y + 50.0, 12.0, Color::from_hex(0xddaadd));
    }

    let list_y = box_y + header_h;
    for (i, outfit) in OUTFITS.iter().enumerate() {
        let ry = list_y + i as f32 * row_h;
        let selected = i == state.outfit_cursor;
        let owned = state.owned_outfits.contains(&(i as u8));
        let equipped = state.player.outfit == i as u8;

        if selected {
            draw_rectangle(box_x + 4.0, ry, box_w - 8.0, row_h - 2.0, Color::from_hex(0x2a2a5e));
        }

        // Preview character
        let px = box_x + 10.0;
        let py = ry + 2.0;
        let shirt = Color::new(outfit.shirt.0 as f32 / 255.0, outfit.shirt.1 as f32 / 255.0, outfit.shirt.2 as f32 / 255.0, 1.0);
        let pants = Color::new(outfit.pants.0 as f32 / 255.0, outfit.pants.1 as f32 / 255.0, outfit.pants.2 as f32 / 255.0, 1.0);
        let shoes = Color::new(outfit.shoes.0 as f32 / 255.0, outfit.shoes.1 as f32 / 255.0, outfit.shoes.2 as f32 / 255.0, 1.0);
        let hat_c = Color::new(outfit.hat.0 as f32 / 255.0, outfit.hat.1 as f32 / 255.0, outfit.hat.2 as f32 / 255.0, 1.0);
        let skin = Color { r: 0.96, g: 0.80, b: 0.62, a: 1.0 };
        let hair = player_hair_color(&state.player);

        draw_character(px, py, shirt, pants, shoes, skin, hair, &crate::game::player::Direction::Down);
        if state.player.gender == 1 && state.player.hairstyle > 0 {
            draw_hairstyle(px, py, hair, state.player.hairstyle);
        }
        // Hat
        draw_rectangle(px + 5.0, py - 2.0, 22.0, 4.0, hat_c);
        draw_rectangle(px + 9.0, py - 9.0, 14.0, 8.0, hat_c);

        // Name
        let name_color = if equipped { Color::from_hex(0xf1c40f) } else if selected { WHITE } else { Color::from_hex(0xcccccc) };
        draw_text(outfit.name, box_x + 54.0, ry + 22.0, 16.0, name_color);

        // Status
        if equipped {
            let ew = measure_text("Wearing", None, 14, 1.0).width;
            draw_text("Wearing", box_x + box_w - ew - 14.0, ry + 16.0, 14.0, Color::from_hex(0xf1c40f));
            draw_text("★", box_x + box_w - ew - 28.0, ry + 17.0, 14.0, Color::from_hex(0xf1c40f));
        } else if owned {
            let label = "E: Wear";
            let lw = measure_text(label, None, 13, 1.0).width;
            draw_text(label, box_x + box_w - lw - 14.0, ry + 22.0, 13.0, Color::from_hex(0x27ae60));
        } else {
            let price_text = format!("{}g", outfit.price);
            let pw = measure_text(&price_text, None, 16, 1.0).width;
            let affordable = state.player.gold >= outfit.price;
            let price_color = if affordable { Color::from_hex(0xf1c40f) } else { Color::from_hex(0xe74c3c) };
            draw_text(&price_text, box_x + box_w - pw - 14.0, ry + 22.0, 16.0, price_color);
        }
    }

    let hint = if state.player.gender == 1 {
        "↑↓: Outfit  |  E: Buy/Wear  |  G: Gender  |  H: Hair  |  C: Color  |  Esc: Close"
    } else {
        "↑↓: Outfit  |  E: Buy/Wear  |  G: Gender  |  Esc: Close"
    };
    draw_text(hint, box_x + 14.0, box_y + box_h - 12.0, 11.0, Color::from_hex(0xaaaaaa));
}
