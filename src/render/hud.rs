use macroquad::prelude::*;
use crate::game::state::GameState;
use crate::game::npc::VICTOR_ID;
use crate::game::inventory::ItemKind;

pub fn draw(state: &GameState) {
    let sw = screen_width();
    let sh = screen_height();

    // ── Top bar ───────────────────────────────────────────────────────────────
    draw_rectangle(0.0, 0.0, sw, 36.0, Color::from_hex(0x1a1a2e));

    // Date & time
    let date = state.clock.display_date();
    let time = state.clock.display_time();
    let weather = if state.raining { " | Rain" } else if state.rainbow_day { " | Rainbow! 50% OFF" } else { "" };
    draw_text(&format!("{} | {}{}", date, time, weather), 10.0, 24.0, 20.0, WHITE);

    // Energy bar
    let energy_ratio = state.player.energy as f32 / state.player.max_energy as f32;
    let bar_x = sw / 2.0 - 60.0;
    draw_rectangle(bar_x, 8.0, 120.0, 16.0, Color::from_hex(0x333333));
    draw_rectangle(bar_x, 8.0, 120.0 * energy_ratio.max(0.0), 16.0, Color::from_hex(0x2ecc71));
    draw_text("Energy", bar_x + 35.0, 22.0, 14.0, WHITE);

    // Music indicator
    let music_label = match state.music_track {
        1 => "♪ Guqin",
        2 => "♪ Pop",
        _ => "",
    };
    let mw = if !music_label.is_empty() {
        let w = measure_text(music_label, None, 16, 1.0).width;
        draw_text(music_label, sw - w - 10.0, 22.0, 16.0, Color::from_hex(0x88ddff));
        w + 14.0
    } else { 0.0 };

    // Gold
    let gold_text = format!("{}g", state.player.gold);
    let gw = measure_text(&gold_text, None, 20, 1.0).width;
    draw_text(&gold_text, sw - gw - 10.0 - mw, 24.0, 20.0, Color::from_hex(0xf1c40f));

    // Charisma level
    let charm_text = format!("Charm: {}/10", state.player.charisma_level());
    let cw = measure_text(&charm_text, None, 18, 1.0).width;
    draw_text(&charm_text, sw - gw - cw - 30.0 - mw, 24.0, 18.0, Color::from_hex(0xe74cdb));

    // ── Victor hearts strip ───────────────────────────────────────────────────
    let victor_strip_y = 58.0;
    if let Some(victor) = state.npcs.iter().find(|n| n.id == VICTOR_ID) {
        let hearts = victor.hearts() as usize;
        let max = crate::game::npc::VICTOR_FINAL_HEARTS as usize;
        let label = "Victor:";
        let lw = measure_text(label, None, 14, 1.0).width;
        let strip_x = sw / 2.0 - (lw + max as f32 * 16.0 + 4.0) / 2.0;
        draw_text(label, strip_x, victor_strip_y, 14.0, Color::from_hex(0xcc4444));
        for i in 0..max {
            let hx = strip_x + lw + 4.0 + i as f32 * 16.0;
            let filled = i < hearts;
            let color = if filled { Color::from_hex(0xe74c3c) } else { Color::from_hex(0x555555) };
            draw_heart(hx, victor_strip_y - 10.0, color);
        }
    }

    // ── Spouse hearts strip ───────────────────────────────────────────────────
    if let Some(spouse_id) = state.married_npc_id {
        if let Some(spouse) = state.npcs.iter().find(|n| n.id == spouse_id) {
            let hearts = spouse.hearts() as usize;
            let max = 10usize;
            let label_str = format!("{}:", &spouse.name[..spouse.name.len().min(6)]);
            let label = label_str.as_str();
            let lw = measure_text(label, None, 14, 1.0).width;
            let strip_x = sw / 2.0 - (lw + max as f32 * 16.0 + 4.0) / 2.0;
            let sy = victor_strip_y + 20.0;
            draw_text(label, strip_x, sy, 14.0, Color::from_hex(0xff88cc));
            for i in 0..max {
                let hx = strip_x + lw + 4.0 + i as f32 * 16.0;
                let filled = i < hearts;
                let color = if filled { Color::from_hex(0xff44aa) } else { Color::from_hex(0x555555) };
                draw_heart(hx, sy - 10.0, color);
            }
        }
    }

    // ── Pending gold ──────────────────────────────────────────────────────────
    if state.pending_gold > 0 {
        let pending = format!("Shipping: {}g", state.pending_gold);
        draw_text(&pending, sw - 180.0, 54.0, 16.0, Color::from_hex(0xf39c12));
    }

    // ── Notification ──────────────────────────────────────────────────────────
    if let Some((msg, ttl)) = &state.notification {
        let alpha = (ttl / 2.5).min(1.0);
        let color = Color { r: 1.0, g: 0.9, b: 0.2, a: alpha };
        let w = measure_text(msg, None, 20, 1.0).width;
        draw_text(msg, sw / 2.0 - w / 2.0, sh - 50.0, 20.0, color);
    }

    // ── Inventory panel (right side) ──────────────────────────────────────────
    draw_inventory(state, sw, sh);

    // ── P2 stats (co-op only) ────────────────────────────────────────────────
    if state.coop_active {
        let p2_energy_ratio = state.player2.energy as f32 / state.player2.max_energy as f32;
        let p2_bar_x = sw / 2.0 + 40.0;
        draw_rectangle(p2_bar_x, 8.0, 100.0, 14.0, Color::from_hex(0x333333));
        draw_rectangle(p2_bar_x, 8.0, 100.0 * p2_energy_ratio.max(0.0), 14.0, Color::from_hex(0x3498db));
        draw_text("P2", p2_bar_x - 20.0, 20.0, 12.0, Color::from_hex(0x3498db));

        // P2 inventory panel (left side)
        draw_inventory_p2(state, sw, sh);
    }

    // ── Controls hint at bottom ───────────────────────────────────────────────
    draw_rectangle(0.0, sh - 24.0, sw, 24.0, Color::from_hex(0x1a1a2e));
    let hint = if state.coop_active {
        "P1: WASD+keys | P2: IJKL+N (or Arrows+Enter) | /: P2 Relations | Z: Sleep"
    } else {
        "WASD: Move | H: Hoe | F: Water | P: Plant | R: Harvest | G: Forage | C: Fish | M: Mine | T: Gift | E: Interact | O: Outfits | J: Horse | F1: Music | Z: Sleep"
    };
    draw_text(hint, 10.0, sh - 6.0, 14.0, Color::from_hex(0xaaaaaa));
}

fn draw_heart(x: f32, y: f32, color: Color) {
    // Simple heart using two circles + a triangle
    draw_circle(x + 4.0, y + 4.0, 4.0, color);
    draw_circle(x + 9.0, y + 4.0, 4.0, color);
    draw_triangle(
        Vec2::new(x,      y + 6.0),
        Vec2::new(x + 13.0, y + 6.0),
        Vec2::new(x + 6.5, y + 14.0),
        color,
    );
}

fn draw_inventory_p2(state: &GameState, sw: f32, sh: f32) {
    let inv = state.player2.inventory.items();
    if inv.is_empty() { return; }
    let mut entries: Vec<(String, u32)> = inv.iter().map(|(k, &qty)| (item_label(k), qty)).collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    let row_h = 18.0;
    let pad = 8.0;
    let panel_w = 110.0;
    let panel_h = pad * 2.0 + entries.len() as f32 * row_h;
    let panel_x = 6.0;
    let panel_y = sh - panel_h - 30.0;
    draw_rectangle(panel_x, panel_y, panel_w, panel_h, Color { r: 0.06, g: 0.06, b: 0.15, a: 0.82 });
    draw_rectangle_lines(panel_x, panel_y, panel_w, panel_h, 1.0, Color::from_hex(0x224466));
    draw_text("P2 Bag", panel_x + pad, panel_y + pad + 10.0, 13.0, Color::from_hex(0x3498db));
    for (i, (label, qty)) in entries.iter().enumerate() {
        let ry = panel_y + pad + (i + 1) as f32 * row_h + 2.0;
        let qty_str = format!("x{}", qty);
        let qw = measure_text(&qty_str, None, 13, 1.0).width;
        draw_text(label, panel_x + pad, ry, 13.0, WHITE);
        draw_text(&qty_str, panel_x + panel_w - qw - pad, ry, 13.0, Color::from_hex(0x3498db));
    }
}

fn draw_inventory(state: &GameState, sw: f32, sh: f32) {
    // Collect non-empty items, sorted by category then name
    let inv = state.player.inventory.items();
    if inv.is_empty() {
        return;
    }

    let mut entries: Vec<(String, u32)> = inv.iter().map(|(k, &qty)| {
        (item_label(k), qty)
    }).collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let row_h  = 18.0;
    let pad    = 8.0;
    let panel_w = 110.0;
    let panel_h = pad * 2.0 + entries.len() as f32 * row_h;
    let panel_x = sw - panel_w - 6.0;
    let panel_y = sh - panel_h - 30.0; // just above controls bar

    // Panel background
    draw_rectangle(panel_x, panel_y, panel_w, panel_h,
                   Color { r: 0.06, g: 0.06, b: 0.15, a: 0.82 });
    draw_rectangle_lines(panel_x, panel_y, panel_w, panel_h, 1.0,
                         Color::from_hex(0x334466));

    // Header
    draw_text("Bag", panel_x + pad, panel_y + pad + 10.0, 13.0, Color::from_hex(0xaaaacc));

    for (i, (label, qty)) in entries.iter().enumerate() {
        let ry = panel_y + pad + (i + 1) as f32 * row_h + 2.0;
        let qty_str = format!("x{}", qty);
        let qw = measure_text(&qty_str, None, 13, 1.0).width;
        draw_text(label,    panel_x + pad,               ry, 13.0, WHITE);
        draw_text(&qty_str, panel_x + panel_w - qw - pad, ry, 13.0, Color::from_hex(0xf1c40f));
    }
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
