use macroquad::prelude::*;
use crate::game::state::GameState;

pub fn draw(state: &GameState) {
    let sw = screen_width();
    let sh = screen_height();

    // Semi-transparent overlay
    draw_rectangle(0.0, 0.0, sw, sh, Color { r: 0.0, g: 0.1, b: 0.2, a: 0.5 });

    // Fishing bar dimensions
    let bar_w = 40.0;
    let bar_h = sh * 0.6;
    let bar_x = sw / 2.0 - bar_w / 2.0;
    let bar_y = sh / 2.0 - bar_h / 2.0;

    // ── Fishing rod (left side) ──────────────────────────────────────────
    let rod_base_x = bar_x - 80.0;
    let rod_base_y = bar_y + bar_h + 20.0;
    let rod_tip_x = bar_x - 10.0;
    let rod_tip_y = bar_y - 20.0;
    // Rod handle (thick, brown)
    draw_line(rod_base_x, rod_base_y, rod_base_x + 20.0, rod_base_y - 40.0, 6.0, Color::from_hex(0x6b3a2a));
    // Rod shaft (thinner, getting thinner toward tip)
    draw_line(rod_base_x + 20.0, rod_base_y - 40.0, rod_tip_x - 10.0, rod_tip_y + 30.0, 4.0, Color::from_hex(0x8b5e3c));
    draw_line(rod_tip_x - 10.0, rod_tip_y + 30.0, rod_tip_x, rod_tip_y, 2.0, Color::from_hex(0xa0714a));
    // Rod tip ring
    draw_circle(rod_tip_x, rod_tip_y, 3.0, Color::from_hex(0x888888));
    // Reel (circle on handle)
    draw_circle(rod_base_x + 14.0, rod_base_y - 28.0, 6.0, Color::from_hex(0x888888));
    draw_circle(rod_base_x + 14.0, rod_base_y - 28.0, 3.0, Color::from_hex(0x666666));
    // Handle grip
    draw_rectangle(rod_base_x - 4.0, rod_base_y - 4.0, 12.0, 8.0, Color::from_hex(0x333333));

    // Fishing line — from rod tip down into the bar, curving toward fish
    let fish_y_screen = bar_y + bar_h - state.fish_pos * bar_h;
    let line_color = Color { r: 0.8, g: 0.8, b: 0.8, a: 0.7 };
    // Rod tip to water surface
    draw_line(rod_tip_x, rod_tip_y, bar_x + bar_w / 2.0, bar_y, 1.5, line_color);
    // Water surface to fish (slight curve via midpoint)
    let mid_x = bar_x + bar_w / 2.0 - 6.0;
    let mid_y = (bar_y + fish_y_screen) / 2.0;
    draw_line(bar_x + bar_w / 2.0, bar_y, mid_x, mid_y, 1.0, line_color);
    draw_line(mid_x, mid_y, bar_x + bar_w / 2.0, fish_y_screen, 1.0, line_color);
    // Bobber at water surface
    draw_circle(bar_x + bar_w / 2.0, bar_y, 4.0, Color::from_hex(0xe74c3c));
    draw_circle(bar_x + bar_w / 2.0, bar_y - 2.0, 2.5, WHITE);

    // Bar background (water)
    draw_rectangle(bar_x, bar_y, bar_w, bar_h, Color::from_hex(0x1a5a8a));
    draw_rectangle_lines(bar_x, bar_y, bar_w, bar_h, 2.0, Color::from_hex(0x2980b9));

    // Water ripple lines
    for i in 0..6 {
        let ry = bar_y + 20.0 + i as f32 * (bar_h / 6.0);
        draw_line(bar_x + 4.0, ry, bar_x + bar_w - 4.0, ry, 1.0,
                  Color { r: 0.3, g: 0.6, b: 0.9, a: 0.2 });
    }

    // Catch zone (green bar that player controls)
    let catch_zone_size = 0.2; // 20% of bar
    let zone_h = bar_h * catch_zone_size;
    let zone_y = bar_y + bar_h - (state.fish_bar + catch_zone_size / 2.0) * bar_h;
    draw_rectangle(bar_x + 2.0, zone_y, bar_w - 4.0, zone_h,
                   Color { r: 0.2, g: 0.8, b: 0.3, a: 0.5 });
    draw_rectangle_lines(bar_x + 2.0, zone_y, bar_w - 4.0, zone_h, 2.0,
                         Color::from_hex(0x27ae60));

    // Fish icon
    let fish_y = bar_y + bar_h - state.fish_pos * bar_h;
    let fish_x = bar_x + bar_w / 2.0;
    // Check if fish is in catch zone
    let bar_top = state.fish_bar + catch_zone_size / 2.0;
    let bar_bot = state.fish_bar - catch_zone_size / 2.0;
    let in_zone = state.fish_pos >= bar_bot && state.fish_pos <= bar_top;
    let fish_color = if in_zone { Color::from_hex(0xf1c40f) } else { Color::from_hex(0xe74c3c) };

    // Draw fish
    draw_circle(fish_x, fish_y, 6.0, fish_color);
    // Tail
    let tail_dir = if state.fish_vel > 0.0 { 1.0 } else { -1.0 };
    draw_triangle(
        Vec2::new(fish_x - 6.0, fish_y),
        Vec2::new(fish_x - 12.0, fish_y - 5.0 * tail_dir),
        Vec2::new(fish_x - 12.0, fish_y + 5.0 * tail_dir),
        fish_color,
    );
    // Eye
    draw_circle(fish_x + 3.0, fish_y - 1.0, 1.5, WHITE);
    draw_circle(fish_x + 3.5, fish_y - 1.0, 0.8, Color::from_hex(0x1a1a1a));

    // Progress bar (right side)
    let prog_x = bar_x + bar_w + 12.0;
    let prog_w = 14.0;
    draw_rectangle(prog_x, bar_y, prog_w, bar_h, Color::from_hex(0x333333));
    draw_rectangle_lines(prog_x, bar_y, prog_w, bar_h, 1.0, Color::from_hex(0x555555));
    // Fill from bottom
    let fill_h = bar_h * state.fish_progress.clamp(0.0, 1.0);
    let fill_color = if state.fish_progress > 0.7 {
        Color::from_hex(0x27ae60)
    } else if state.fish_progress > 0.3 {
        Color::from_hex(0xf39c12)
    } else {
        Color::from_hex(0xe74c3c)
    };
    draw_rectangle(prog_x + 1.0, bar_y + bar_h - fill_h, prog_w - 2.0, fill_h, fill_color);

    // Fish name
    if let Some(ref fish) = state.fish_target {
        let name = fish.name();
        let nw = measure_text(name, None, 20, 1.0).width;
        draw_text(name, sw / 2.0 - nw / 2.0, bar_y - 20.0, 20.0, Color::from_hex(0xf1c40f));
    }

    // Instructions
    let hint = "W/Up: Reel   |   Keep fish in green zone!";
    let hw = measure_text(hint, None, 14, 1.0).width;
    draw_text(hint, sw / 2.0 - hw / 2.0, bar_y + bar_h + 24.0, 14.0, Color::from_hex(0xcccccc));

    let hint2 = "Esc: Give up";
    let hw2 = measure_text(hint2, None, 12, 1.0).width;
    draw_text(hint2, sw / 2.0 - hw2 / 2.0, bar_y + bar_h + 44.0, 12.0, Color::from_hex(0x888888));
}
