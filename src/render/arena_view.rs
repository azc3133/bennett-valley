use macroquad::prelude::*;
use crate::game::state::GameState;

/// Draw the arena jump editor overlay.
pub fn draw(state: &GameState) {
    let sw = screen_width();
    let sh = screen_height();

    draw_rectangle(0.0, 0.0, sw, sh, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.6 });

    let cols = 9u8;
    let rows = 4u8;
    let cell = 52.0f32;
    let grid_w = cols as f32 * cell;
    let grid_h = rows as f32 * cell;
    let gx = sw / 2.0 - grid_w / 2.0;
    let gy = sh / 2.0 - grid_h / 2.0 + 10.0;

    // Title
    let title = "Jump Editor";
    let tw = measure_text(title, None, 24, 1.0).width;
    draw_text(title, sw / 2.0 - tw / 2.0, gy - 30.0, 24.0, Color::from_hex(0xf1c40f));

    let info = format!("Jumps: {}/6", state.arena_jumps.len());
    let iw = measure_text(&info, None, 16, 1.0).width;
    draw_text(&info, sw / 2.0 - iw / 2.0, gy - 10.0, 16.0, WHITE);

    // Arena grid
    let sand = Color::from_hex(0xd4b880);
    draw_rectangle(gx, gy, grid_w, grid_h, sand);
    draw_rectangle_lines(gx, gy, grid_w, grid_h, 3.0, Color::from_hex(0x8b5e3c));

    // Grid lines
    for c in 1..cols {
        let lx = gx + c as f32 * cell;
        draw_line(lx, gy, lx, gy + grid_h, 1.0, Color { r: 0.6, g: 0.5, b: 0.3, a: 0.3 });
    }
    for r in 1..rows {
        let ly = gy + r as f32 * cell;
        draw_line(gx, ly, gx + grid_w, ly, 1.0, Color { r: 0.6, g: 0.5, b: 0.3, a: 0.3 });
    }

    // Draw placed jumps
    let jump_colors = [0xe74c3c, 0x2980b9, 0x27ae60, 0xf39c12, 0x8e44ad, 0x1abc9c];
    for (i, &(jc, jr, orient)) in state.arena_jumps.iter().enumerate() {
        let jx = gx + jc as f32 * cell;
        let jy = gy + jr as f32 * cell;
        let color = Color::from_hex(jump_colors[i % jump_colors.len()]);
        draw_jump_icon(jx, jy, cell, orient, color);
    }

    // Cursor
    let (cc, cr) = state.arena_cursor;
    let cx = gx + cc as f32 * cell;
    let cy = gy + cr as f32 * cell;
    draw_rectangle_lines(cx, cy, cell, cell, 3.0, Color::from_hex(0xf1c40f));

    // Cursor contents hint
    let existing = state.arena_jumps.iter().find(|j| j.0 == cc && j.1 == cr);
    if let Some(&(_, _, orient)) = existing {
        let orient_name = match orient {
            0 => "Horizontal",
            1 => "Vertical",
            2 => "Diagonal /",
            _ => "Diagonal \\",
        };
        let next = if orient < 3 { "rotate" } else { "remove" };
        let hint = format!("{} — E: {}", orient_name, next);
        let hw = measure_text(&hint, None, 12, 1.0).width;
        draw_text(&hint, cx + cell / 2.0 - hw / 2.0, cy + cell + 14.0, 12.0, Color::from_hex(0xf39c12));
    } else {
        let hint = "E: Place jump";
        let hw = measure_text(hint, None, 12, 1.0).width;
        draw_text(hint, cx + cell / 2.0 - hw / 2.0, cy + cell + 14.0, 12.0, Color::from_hex(0x27ae60));
    }

    // Footer
    let footer = "WASD: Move  |  E: Place/Rotate/Remove  |  Esc: Done";
    let fw = measure_text(footer, None, 13, 1.0).width;
    draw_text(footer, sw / 2.0 - fw / 2.0, gy + grid_h + 30.0, 13.0, Color::from_hex(0xaaaaaa));

    // Fence label at edges
    draw_text("Fence", gx - 4.0, gy - 4.0, 10.0, Color::from_hex(0x8b5e3c));
    draw_text("Gate", gx + grid_w / 2.0 - 12.0, gy + grid_h + 12.0, 10.0, Color::from_hex(0x8b5e3c));
}

fn draw_jump_icon(x: f32, y: f32, cell: f32, orient: u8, color: Color) {
    let cx = x + cell / 2.0;
    let cy = y + cell / 2.0;

    match orient {
        0 => {
            // Horizontal — rails go left-right
            draw_rectangle(cx - 18.0, cy - 4.0, 4.0, 18.0, Color::from_hex(0xeeeeee));
            draw_rectangle(cx + 14.0, cy - 4.0, 4.0, 18.0, Color::from_hex(0xeeeeee));
            draw_rectangle(cx - 18.0, cy, 36.0, 4.0, color);
            draw_rectangle(cx - 18.0, cy + 8.0, 36.0, 4.0, color);
            draw_rectangle(cx - 6.0, cy, 12.0, 4.0, WHITE);
        }
        1 => {
            // Vertical — rails go up-down
            draw_rectangle(cx - 4.0, cy - 18.0, 18.0, 4.0, Color::from_hex(0xeeeeee));
            draw_rectangle(cx - 4.0, cy + 14.0, 18.0, 4.0, Color::from_hex(0xeeeeee));
            draw_rectangle(cx, cy - 18.0, 4.0, 36.0, color);
            draw_rectangle(cx + 8.0, cy - 18.0, 4.0, 36.0, color);
            draw_rectangle(cx, cy - 6.0, 4.0, 12.0, WHITE);
        }
        2 => {
            // Diagonal / (bottom-left to top-right)
            draw_circle(cx - 14.0, cy + 14.0, 4.0, Color::from_hex(0xeeeeee));
            draw_circle(cx + 14.0, cy - 14.0, 4.0, Color::from_hex(0xeeeeee));
            draw_line(cx - 14.0, cy + 14.0, cx + 14.0, cy - 14.0, 4.0, color);
            draw_line(cx - 10.0, cy + 16.0, cx + 18.0, cy - 12.0, 4.0, color);
            draw_line(cx - 2.0, cy + 2.0, cx + 4.0, cy - 4.0, 4.0, WHITE);
        }
        _ => {
            // Diagonal \ (top-left to bottom-right)
            draw_circle(cx - 14.0, cy - 14.0, 4.0, Color::from_hex(0xeeeeee));
            draw_circle(cx + 14.0, cy + 14.0, 4.0, Color::from_hex(0xeeeeee));
            draw_line(cx - 14.0, cy - 14.0, cx + 14.0, cy + 14.0, 4.0, color);
            draw_line(cx - 10.0, cy - 16.0, cx + 18.0, cy + 12.0, 4.0, color);
            draw_line(cx - 2.0, cy - 2.0, cx + 4.0, cy + 4.0, 4.0, WHITE);
        }
    }
}
