use macroquad::prelude::*;
use crate::game::dialogue::DialogueState;
use crate::game::state::{GameState, REPLY_TOPICS};

/// Shown while waiting for the letter LLM coroutine.
pub fn draw_letter_waiting() {
    let sw = screen_width();
    let sh = screen_height();
    let box_h = 120.0;
    let box_y = sh / 2.0 - box_h / 2.0;
    let box_w = 360.0;
    let box_x = sw / 2.0 - box_w / 2.0;

    draw_rectangle(box_x, box_y, box_w, box_h, Color::from_hex(0xf5e6c8));
    draw_rectangle_lines(box_x, box_y, box_w, box_h, 2.0, Color::from_hex(0x8b6914));
    let msg = "A letter arrives...";
    let mw = measure_text(msg, None, 18, 1.0).width;
    draw_text(msg, sw / 2.0 - mw / 2.0, box_y + 50.0, 18.0, Color::from_hex(0x4a3000));
    draw_text("...", sw / 2.0 - 12.0, box_y + 80.0, 20.0, Color::from_hex(0x8b6914));
}

/// Renders a received letter as a parchment overlay.
pub fn draw_letter(friend_name: &str, text: &str) {
    let sw = screen_width();
    let sh = screen_height();

    // Dark overlay behind the letter
    draw_rectangle(0.0, 0.0, sw, sh, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.55 });

    let bw = (sw - 120.0).min(520.0);
    let bx = sw / 2.0 - bw / 2.0;

    // Word-wrap the letter body first so we can size the box
    let body_font = 15u16;
    let line_h = 22.0f32;
    let text_margin = 30.0;
    let lines = wrap_text_px(text, body_font, bw - text_margin * 2.0);
    // header(30) + divider(10) + body + footer(30)
    let bh = (70.0 + lines.len() as f32 * line_h + 30.0).max(160.0).min(sh - 60.0);
    let by = sh / 2.0 - bh / 2.0;

    // Parchment background + border
    draw_rectangle(bx, by, bw, bh, Color::from_hex(0xf5e6c8));
    draw_rectangle_lines(bx, by, bw, bh, 3.0, Color::from_hex(0x8b6914));

    // Header — "Letter from Jules"
    let header = format!("Letter from {}", friend_name);
    let hw = measure_text(&header, None, 20, 1.0).width;
    draw_text(&header, sw / 2.0 - hw / 2.0, by + 30.0, 20.0, Color::from_hex(0x4a3000));

    // Divider line
    draw_line(bx + 20.0, by + 40.0, bx + bw - 20.0, by + 40.0, 1.0, Color::from_hex(0x8b6914));

    // Letter body — pixel-accurate word-wrap
    let max_lines = ((bh - 100.0) / line_h).max(1.0) as usize;
    for (i, line) in lines.iter().take(max_lines).enumerate() {
        draw_text(line, bx + text_margin, by + 68.0 + i as f32 * line_h, body_font as f32, Color::from_hex(0x2a1800));
    }

    // Dismiss / reply hints
    draw_text("R: Reply", bx + 20.0, by + bh - 12.0, 13.0, Color::from_hex(0x8b6914));
    let hint = "E: Close";
    let hiw = measure_text(hint, None, 13, 1.0).width;
    draw_text(hint, bx + bw - hiw - 20.0, by + bh - 12.0, 13.0, Color::from_hex(0x8b6914));
}

/// Rendered during LetterReply phase — shows 3 reply topic choices over the letter.
pub fn draw_letter_reply(state: &GameState) {
    let sw = screen_width();
    let sh = screen_height();

    // Dark overlay
    draw_rectangle(0.0, 0.0, sw, sh, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.55 });

    let bw = (sw - 120.0).min(480.0);
    let bh = 200.0;
    let bx = sw / 2.0 - bw / 2.0;
    let by = sh / 2.0 - bh / 2.0;

    draw_rectangle(bx, by, bw, bh, Color::from_hex(0xf5e6c8));
    draw_rectangle_lines(bx, by, bw, bh, 3.0, Color::from_hex(0x8b6914));

    let header = "Write back...";
    let hw = measure_text(header, None, 20, 1.0).width;
    draw_text(header, sw / 2.0 - hw / 2.0, by + 30.0, 20.0, Color::from_hex(0x4a3000));
    draw_line(bx + 20.0, by + 40.0, bx + bw - 20.0, by + 40.0, 1.0, Color::from_hex(0x8b6914));

    for (i, topic) in REPLY_TOPICS.iter().enumerate() {
        let ry = by + 68.0 + i as f32 * 34.0;
        let selected = i == state.reply_cursor;
        let bg_color = if selected {
            Color { r: 0.8, g: 0.65, b: 0.3, a: 0.4 }
        } else {
            Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }
        };
        draw_rectangle(bx + 12.0, ry - 18.0, bw - 24.0, 26.0, bg_color);
        let arrow = if selected { "▶ " } else { "  " };
        let color = if selected { Color::from_hex(0x2a1800) } else { Color::from_hex(0x6a4800) };
        draw_text(&format!("{}{}", arrow, topic), bx + 24.0, ry, 16.0, color);
    }

    draw_text("↑↓ · E to send · Esc to cancel", bx + 20.0, by + bh - 12.0, 13.0, Color::from_hex(0x8b6914));
}

/// Word-wrap `text` so each line fits within `max_width` pixels at `font_size`.
fn wrap_text_px(text: &str, font_size: u16, max_width: f32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.is_empty() {
            current.push_str(word);
        } else {
            let candidate = format!("{} {}", current, word);
            let w = measure_text(&candidate, None, font_size, 1.0).width;
            if w <= max_width {
                current = candidate;
            } else {
                lines.push(current);
                current = word.to_string();
            }
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

/// Shown while the LLM is generating a response.
pub fn draw_thinking(npc_name: &str) {
    let sw = screen_width();
    let sh = screen_height();
    let box_h = 100.0;
    let box_y = sh - box_h - 30.0;

    draw_rectangle(20.0, box_y, sw - 40.0, box_h, Color::from_hex(0x1a1a2e));
    draw_rectangle_lines(20.0, box_y, sw - 40.0, box_h, 2.0, Color::from_hex(0x555577));
    draw_text(npc_name, 36.0, box_y + 22.0, 18.0, Color::from_hex(0xf1c40f));
    draw_text("...", 36.0, box_y + 50.0, 22.0, Color::from_hex(0x8888aa));
}

/// Shown during DialogueChoice phase — NPC name + 3 selectable player responses.
pub fn draw_choices(state: &GameState) {
    let sw = screen_width();
    let sh = screen_height();

    // Pre-compute total height needed for wrapped options
    let option_max_w = sw - 100.0;
    let mut total_option_h = 0.0f32;
    for option in &state.response_options {
        let lines = wrap_text_px(&format!("  {}", option), 15, option_max_w);
        total_option_h += lines.len() as f32 * 20.0 + 10.0;
    }
    let box_h = (46.0 + total_option_h + 24.0).max(140.0);
    let box_y = sh - box_h - 30.0;

    // Box background
    draw_rectangle(20.0, box_y, sw - 40.0, box_h, Color::from_hex(0x1a1a2e));
    draw_rectangle_lines(20.0, box_y, sw - 40.0, box_h, 2.0, Color::from_hex(0xf1c40f));

    // Prompt line
    let npc_name = state.waiting_npc_name.as_deref().unwrap_or("NPC");
    draw_text(npc_name, 36.0, box_y + 22.0, 18.0, Color::from_hex(0xf1c40f));
    draw_text("How do you respond?", 130.0, box_y + 22.0, 14.0, Color::from_hex(0xaaaaaa));

    // Options — wrap long text
    let option_max_w = sw - 100.0;
    let mut ry = box_y + 46.0;
    for (i, option) in state.response_options.iter().enumerate() {
        let selected = i == state.choice_cursor;
        let arrow = if selected { "▶ " } else { "  " };
        let full = format!("{}{}", arrow, option);
        let lines = wrap_text_px(&full, 15, option_max_w);
        let block_h = lines.len() as f32 * 20.0 + 6.0;
        let bg_color = if selected { Color::from_hex(0x2a2a4e) } else { Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 } };
        draw_rectangle(30.0, ry - 16.0, sw - 60.0, block_h, bg_color);
        let text_color = if selected { WHITE } else { Color::from_hex(0x9999bb) };
        for (j, line) in lines.iter().enumerate() {
            draw_text(line, 40.0, ry + j as f32 * 20.0, 15.0, text_color);
        }
        ry += block_h + 4.0;
    }

    // Hint
    draw_text("↑↓ to choose · E to confirm", sw - 220.0, box_y + box_h - 8.0, 13.0, Color::from_hex(0x888888));
}

pub fn draw(dialogue: &DialogueState) {
    let sw = screen_width();
    let sh = screen_height();

    // NPC name
    let name_y = 22.0;
    let text_start_y = 46.0;
    let line_h = 20.0;
    let text_margin = 36.0;
    let box_x = 20.0;
    let box_inner_w = sw - 40.0 - text_margin * 2.0 + 20.0;

    // Word-wrap dialogue text to determine box height
    let lines = if let Some(text) = dialogue.current_text() {
        wrap_text_px(text, 16, box_inner_w)
    } else {
        vec![]
    };
    let box_h = (text_start_y + lines.len().max(1) as f32 * line_h + 24.0).max(80.0);
    let box_y = sh - box_h - 30.0;

    // Box background
    draw_rectangle(box_x, box_y, sw - 40.0, box_h, Color::from_hex(0x1a1a2e));
    draw_rectangle_lines(box_x, box_y, sw - 40.0, box_h, 2.0, WHITE);

    // NPC name
    draw_text(&dialogue.npc_name, text_margin, box_y + name_y, 18.0, Color::from_hex(0xf1c40f));

    // Dialogue text — wrapped
    for (i, line) in lines.iter().enumerate() {
        draw_text(line, text_margin, box_y + text_start_y + i as f32 * line_h, 16.0, WHITE);
    }

    // Advance hint
    draw_text("Press E to continue", sw - 180.0, box_y + box_h - 10.0, 14.0, Color::from_hex(0xaaaaaa));
}
