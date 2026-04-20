use macroquad::prelude::*;
use crate::game::state::GameState;

pub fn draw(state: &GameState) {
    let sw = screen_width();
    let sh = screen_height();

    // Dark arcade room background
    draw_rectangle(0.0, 0.0, sw, sh, Color::from_hex(0x0a0a1a));

    // Neon border
    draw_rectangle_lines(30.0, 30.0, sw - 60.0, sh - 60.0, 2.0, Color::from_hex(0xff00ff));

    // Title
    let title = "REACTION GAME";
    let tw = measure_text(title, None, 28, 1.0).width;
    draw_text(title, sw / 2.0 - tw / 2.0, 70.0, 28.0, Color::from_hex(0x00ffff));

    // The light
    let light_x = sw / 2.0;
    let light_y = sh / 2.0 - 30.0;
    let light_r = 60.0;

    match state.arcade_phase {
        0 => {
            // Waiting — red light
            draw_circle(light_x, light_y, light_r, Color::from_hex(0x331111));
            draw_circle(light_x, light_y, light_r - 8.0, Color::from_hex(0xcc2222));
            // Glow
            draw_circle(light_x, light_y, light_r + 10.0, Color { r: 0.8, g: 0.1, b: 0.1, a: 0.1 });

            let hint = "Wait for GREEN...";
            let hw = measure_text(hint, None, 20, 1.0).width;
            draw_text(hint, sw / 2.0 - hw / 2.0, light_y + light_r + 40.0, 20.0, Color::from_hex(0xff4444));

            let hint2 = "Press E when it turns green!";
            let hw2 = measure_text(hint2, None, 14, 1.0).width;
            draw_text(hint2, sw / 2.0 - hw2 / 2.0, light_y + light_r + 65.0, 14.0, Color::from_hex(0x888888));
        }
        1 => {
            // Green! React now!
            draw_circle(light_x, light_y, light_r, Color::from_hex(0x113311));
            draw_circle(light_x, light_y, light_r - 8.0, Color::from_hex(0x22cc22));
            // Bright glow
            draw_circle(light_x, light_y, light_r + 20.0, Color { r: 0.1, g: 0.8, b: 0.1, a: 0.15 });
            draw_circle(light_x, light_y, light_r + 40.0, Color { r: 0.1, g: 0.6, b: 0.1, a: 0.08 });

            let hint = "NOW! Press E!";
            let hw = measure_text(hint, None, 24, 1.0).width;
            draw_text(hint, sw / 2.0 - hw / 2.0, light_y + light_r + 40.0, 24.0, Color::from_hex(0x22ff22));
        }
        _ => {
            // Results
            draw_circle(light_x, light_y, light_r, Color::from_hex(0x222222));
            draw_circle(light_x, light_y, light_r - 8.0, Color::from_hex(0x555555));

            if state.arcade_prize > 0 {
                let time_text = format!("{:.3}s", state.arcade_reaction);
                let ttw = measure_text(&time_text, None, 32, 1.0).width;
                draw_text(&time_text, sw / 2.0 - ttw / 2.0, light_y + light_r + 35.0, 32.0, Color::from_hex(0x00ffff));

                let rating = if state.arcade_reaction < 0.2 {
                    ("INCREDIBLE!", 0xffff00)
                } else if state.arcade_reaction < 0.4 {
                    ("GREAT!", 0x00ff00)
                } else if state.arcade_reaction < 0.7 {
                    ("Good!", 0x44ff44)
                } else if state.arcade_reaction < 1.0 {
                    ("OK", 0xffaa00)
                } else {
                    ("Slow...", 0xff4444)
                };
                let rw = measure_text(rating.0, None, 22, 1.0).width;
                draw_text(rating.0, sw / 2.0 - rw / 2.0, light_y + light_r + 65.0, 22.0, Color::from_hex(rating.1));

                let prize_text = format!("Won {}g!", state.arcade_prize);
                let pw = measure_text(&prize_text, None, 20, 1.0).width;
                draw_text(&prize_text, sw / 2.0 - pw / 2.0, light_y + light_r + 90.0, 20.0, Color::from_hex(0xf1c40f));
            } else {
                let fail = "TOO EARLY!";
                let fw = measure_text(fail, None, 28, 1.0).width;
                draw_text(fail, sw / 2.0 - fw / 2.0, light_y + light_r + 40.0, 28.0, Color::from_hex(0xff0000));
                let no_prize = "No prize this time.";
                let npw = measure_text(no_prize, None, 16, 1.0).width;
                draw_text(no_prize, sw / 2.0 - npw / 2.0, light_y + light_r + 70.0, 16.0, Color::from_hex(0x888888));
            }

            let hint = "E: Play again  |  Esc: Leave";
            let hw = measure_text(hint, None, 14, 1.0).width;
            draw_text(hint, sw / 2.0 - hw / 2.0, sh - 50.0, 14.0, Color::from_hex(0xaaaaaa));
        }
    }

    // Decorative arcade machines on sides
    for &ax in &[60.0, sw - 90.0] {
        draw_rectangle(ax, sh / 2.0 - 40.0, 30.0, 80.0, Color::from_hex(0x222244));
        draw_rectangle(ax + 4.0, sh / 2.0 - 36.0, 22.0, 18.0, Color::from_hex(0x00aa00));
        draw_rectangle(ax + 8.0, sh / 2.0 - 32.0, 14.0, 10.0, Color::from_hex(0x003300));
        draw_circle(ax + 15.0, sh / 2.0 + 10.0, 5.0, Color::from_hex(0xff0000));
    }

    // Floor pattern
    for i in 0..20 {
        for j in 0..4 {
            let fx = 40.0 + i as f32 * ((sw - 80.0) / 20.0);
            let fy = sh - 40.0 - j as f32 * 8.0;
            if (i + j) % 2 == 0 {
                draw_rectangle(fx, fy, (sw - 80.0) / 20.0, 8.0, Color::from_hex(0x1a1a2e));
            }
        }
    }
}
