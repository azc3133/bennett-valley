use macroquad::prelude::*;
use macroquad::camera::{Camera2D, set_camera, set_default_camera};
use crate::game::state::{Bird, GamePhase, GameState, Squirrel};
use crate::render::animal_view;
use crate::render::{
    camera::Camera,
    dialogue_view, farm_view, farmhouse_view, hud, npc_view, player_view, relationships_view,
    arcade_view, arena_view, festival_view, fishing_view, furniture_view, outfit_view, restaurant_view, ship_view, shop_view,
};

pub fn draw(state: &GameState, camera: &Camera) {
    draw_with_coop(state, camera, None);
}

pub fn draw_coop(state: &GameState, camera1: &Camera, camera2: &Camera) {
    if state.phase == GamePhase::FarmhouseInterior {
        hud::draw(state);
        farmhouse_view::draw(state);
        return;
    }

    let sw = screen_width();
    let sh = screen_height();
    let half_w = (sw / 2.0) as i32;
    let sh_i = sh as i32;

    // ── Left half: P1's view (normal scale, cropped to center) ──
    let cam_left = Camera2D {
        zoom: vec2(4.0 / sw, 2.0 / sh),
        target: vec2(sw / 2.0, sh / 2.0),
        viewport: Some((0, 0, half_w, sh_i)),
        ..Default::default()
    };
    set_camera(&cam_left);
    draw_world(state, camera1, false);

    // ── Right half: P2's view (normal scale, cropped to center) ──
    let cam_right = Camera2D {
        zoom: vec2(4.0 / sw, 2.0 / sh),
        target: vec2(sw / 2.0, sh / 2.0),
        viewport: Some((half_w, 0, half_w, sh_i)),
        ..Default::default()
    };
    set_camera(&cam_right);
    draw_world(state, camera2, true);

    // ── Back to full screen for HUD + divider ──
    set_default_camera();

    // Divider line
    draw_rectangle(sw / 2.0 - 1.5, 0.0, 3.0, sh, Color::from_hex(0x1a1a2e));

    // Player labels
    draw_text("P1", 10.0, sh - 30.0, 16.0, Color::from_hex(0xf1c40f));
    draw_text("P2", sw / 2.0 + 10.0, sh - 30.0, 16.0, Color::from_hex(0x3498db));

    hud::draw(state);

    // Relationships overlays (full screen, on top of everything)
    if state.show_relationships {
        relationships_view::draw_labeled(&state.npcs, None, "P1 Relationships (I)");
    }
    if state.show_relationships_p2 {
        relationships_view::draw_labeled(&state.npcs, Some(&state.p2_friendships), "P2 Relationships (/)");
    }
}

fn draw_with_coop(state: &GameState, camera: &Camera, _camera2: Option<&Camera>) {
    if state.phase == GamePhase::FarmhouseInterior {
        hud::draw(state);
        farmhouse_view::draw(state);
        return;
    }

    draw_world(state, camera, false);
    hud::draw(state);

    match &state.phase {
        GamePhase::DialogueOpen => {
            if let Some(dlg) = &state.dialogue {
                dialogue_view::draw(dlg);
            }
        }
        GamePhase::DialogueChoice => {
            dialogue_view::draw_choices(state);
        }
        GamePhase::LlmWaiting => {
            let name = state.waiting_npc_name.as_deref().unwrap_or("NPC");
            dialogue_view::draw_thinking(name);
        }
        GamePhase::LetterWaiting => {
            dialogue_view::draw_letter_waiting();
        }
        GamePhase::LetterOpen => {
            if let Some((friend_name, text)) = &state.current_letter {
                dialogue_view::draw_letter(friend_name, text);
            }
        }
        GamePhase::LetterReply => {
            dialogue_view::draw_letter_reply(state);
        }
        GamePhase::ShopOpen => {
            shop_view::draw(state);
        }
        GamePhase::DaySummary => {
            if let Some(summary) = &state.day_summary {
                draw_day_summary(summary);
            }
        }
        GamePhase::ShipSelect => {
            ship_view::draw(state);
        }
        GamePhase::OutfitShopOpen => {
            outfit_view::draw(state);
        }
        GamePhase::RestaurantOpen => {
            restaurant_view::draw(state);
        }
        GamePhase::ArcadePlaying => {
            arcade_view::draw(state);
        }
        GamePhase::ArenaEditor => {
            arena_view::draw(state);
        }
        GamePhase::FishingMinigame => {
            fishing_view::draw(state);
        }
        GamePhase::FurnitureShopOpen => {
            furniture_view::draw(state);
        }
        GamePhase::AnimalShopOpen => {
            animal_view::draw_shop(state);
        }
        GamePhase::FestivalAnnounce | GamePhase::FestivalPlaying | GamePhase::FestivalResults => {
            festival_view::draw(state);
        }
        GamePhase::IceCreamShopOpen => {
            draw_icecream_menu(state);
        }
        GamePhase::HorseDestination => {
            draw_horse_destinations(state);
        }
        GamePhase::Playing | GamePhase::FarmhouseInterior => {}
        GamePhase::Won => {
            draw_win_screen();
        }
        GamePhase::ResetConfirm => {
            draw_reset_confirm();
        }
    }

    // Relationships overlay — renders on top of everything.
    if state.show_relationships {
        relationships_view::draw_labeled(&state.npcs, None, "P1 Relationships");
    }
    if state.show_relationships_p2 {
        relationships_view::draw_labeled(&state.npcs, Some(&state.p2_friendships), "P2 Relationships");
    }
}

fn draw_win_screen() {
    let sw = screen_width();
    let sh = screen_height();

    // Dark overlay
    draw_rectangle(0.0, 0.0, sw, sh, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.75 });

    let bw = 480.0;
    let bh = 280.0;
    let bx = sw / 2.0 - bw / 2.0;
    let by = sh / 2.0 - bh / 2.0;

    draw_rectangle(bx, by, bw, bh, Color::from_hex(0x1a1a2e));
    draw_rectangle_lines(bx, by, bw, bh, 3.0, Color::from_hex(0xf1c40f));

    // Title
    let title = "You Did It!";
    let tw = measure_text(title, None, 32, 1.0).width;
    draw_text(title, bx + bw / 2.0 - tw / 2.0, by + 48.0, 32.0, Color::from_hex(0xf1c40f));

    // Subtitle
    let sub = "Victor calls you a true neighbour.";
    let sw2 = measure_text(sub, None, 18, 1.0).width;
    draw_text(sub, bx + bw / 2.0 - sw2 / 2.0, by + 88.0, 18.0, Color::from_hex(0xe0e0e0));

    // Flavour lines
    draw_text("Bennett Valley is at peace.", bx + 80.0, by + 136.0, 16.0, WHITE);
    draw_text("Crops grow. Friendships last.", bx + 80.0, by + 160.0, 16.0, Color::from_hex(0xaaaaaa));
    draw_text("The valley thanks you, farmer.", bx + 80.0, by + 184.0, 16.0, Color::from_hex(0xaaaaaa));

    // Prompt
    let hint = "Press E to keep farming";
    let hw = measure_text(hint, None, 14, 1.0).width;
    draw_text(hint, bx + bw / 2.0 - hw / 2.0, by + 248.0, 14.0, Color::from_hex(0x888888));
}

fn draw_reset_confirm() {
    let sw = screen_width();
    let sh = screen_height();

    draw_rectangle(0.0, 0.0, sw, sh, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.65 });

    let bw = 380.0;
    let bh = 160.0;
    let bx = sw / 2.0 - bw / 2.0;
    let by = sh / 2.0 - bh / 2.0;

    draw_rectangle(bx, by, bw, bh, Color::from_hex(0x1a1a2e));
    draw_rectangle_lines(bx, by, bw, bh, 3.0, Color::from_hex(0xe74c3c));

    let title = "Reset Game?";
    let tw = measure_text(title, None, 24, 1.0).width;
    draw_text(title, bx + bw / 2.0 - tw / 2.0, by + 38.0, 24.0, Color::from_hex(0xe74c3c));

    let warn = "All progress will be permanently lost.";
    let ww = measure_text(warn, None, 15, 1.0).width;
    draw_text(warn, bx + bw / 2.0 - ww / 2.0, by + 68.0, 15.0, Color::from_hex(0xcccccc));

    let hint = "E: Confirm reset     Esc: Cancel";
    let hw = measure_text(hint, None, 14, 1.0).width;
    draw_text(hint, bx + bw / 2.0 - hw / 2.0, by + bh - 18.0, 14.0, Color::from_hex(0x888888));
}

fn draw_icecream_menu(state: &GameState) {
    let sw = screen_width();
    let sh = screen_height();
    let menu = GameState::ICE_CREAM_MENU;
    let row_h = 26.0;
    let pad = 14.0;
    let bw = 280.0;
    let bh = pad * 2.0 + 32.0 + menu.len() as f32 * row_h + 24.0;
    let bx = sw / 2.0 - bw / 2.0;
    let by = sh / 2.0 - bh / 2.0;

    draw_rectangle(0.0, 0.0, sw, sh, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.5 });
    draw_rectangle(bx, by, bw, bh, Color::from_hex(0xffb6c1));
    draw_rectangle_lines(bx, by, bw, bh, 3.0, Color::from_hex(0xff69b4));

    let title = "Ice Cream Shop";
    let tw = measure_text(title, None, 22, 1.0).width;
    draw_text(title, bx + bw / 2.0 - tw / 2.0, by + pad + 18.0, 22.0, Color::from_hex(0x8b0040));

    for (i, &(name, price, energy)) in menu.iter().enumerate() {
        let ry = by + pad + 38.0 + i as f32 * row_h;
        let selected = i == state.icecream_cursor;
        if selected {
            draw_rectangle(bx + 4.0, ry - 14.0, bw - 8.0, row_h - 2.0, Color::from_hex(0xff89a8));
        }
        let color = if selected { Color::from_hex(0x4a0020) } else { Color::from_hex(0x660033) };
        let eff_price = state.effective_price(price);
        let arrow = if selected { "> " } else { "  " };
        draw_text(&format!("{}{}", arrow, name), bx + pad, ry, 17.0, color);
        let price_text = format!("{}g +{}", eff_price, energy);
        let pw = measure_text(&price_text, None, 14, 1.0).width;
        draw_text(&price_text, bx + bw - pw - pad, ry, 14.0, color);
    }

    let hint = "W/S: Select   E: Order   Esc: Leave";
    let hw = measure_text(hint, None, 12, 1.0).width;
    draw_text(hint, bx + bw / 2.0 - hw / 2.0, by + bh - 8.0, 12.0, Color::from_hex(0x993355));
}

fn draw_horse_destinations(state: &GameState) {
    let sw = screen_width();
    let sh = screen_height();
    let dests = GameState::horse_destinations();
    let total = dests.len();
    let rows_per_col = (total + 1) / 2; // ceil division
    let row_h = 22.0;
    let pad = 12.0;
    let col_w = 160.0;
    let bw = col_w * 2.0 + pad * 3.0;
    let bh = pad * 2.0 + 30.0 + rows_per_col as f32 * row_h + 24.0;
    let bx = sw / 2.0 - bw / 2.0;
    let by = sh / 2.0 - bh / 2.0;

    draw_rectangle(0.0, 0.0, sw, sh, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.5 });
    draw_rectangle(bx, by, bw, bh, Color::from_hex(0x1a1a2e));
    draw_rectangle_lines(bx, by, bw, bh, 2.0, Color::from_hex(0xf1c40f));

    let title = "Where to, partner?";
    let tw = measure_text(title, None, 20, 1.0).width;
    draw_text(title, bx + bw / 2.0 - tw / 2.0, by + pad + 18.0, 20.0, Color::from_hex(0xf1c40f));

    let cursor = state.horse_dest_cursor;

    // Two-column layout: left column = indices 0..rows_per_col, right = rows_per_col..total
    for i in 0..total {
        let (name, _, _) = dests[i];
        let col_idx = i / rows_per_col;
        let row_idx = i % rows_per_col;
        let cx = bx + pad + col_idx as f32 * (col_w + pad);
        let ry = by + pad + 38.0 + row_idx as f32 * row_h;
        let selected = i == cursor;
        if selected {
            draw_rectangle(cx - 2.0, ry - 14.0, col_w + 4.0, row_h - 2.0, Color::from_hex(0x334466));
        }
        let color = if selected { Color::from_hex(0xf1c40f) } else { WHITE };
        let arrow = if selected { "> " } else { "  " };
        draw_text(&format!("{}{}", arrow, name), cx, ry, 16.0, color);
    }

    // Column divider line
    let div_x = bx + pad + col_w + pad * 0.5;
    draw_line(div_x, by + pad + 26.0, div_x, by + bh - 22.0, 1.0, Color::from_hex(0x444466));

    let hint = "WASD: Navigate   E: Go   Esc: Cancel";
    let hw = measure_text(hint, None, 12, 1.0).width;
    draw_text(hint, bx + bw / 2.0 - hw / 2.0, by + bh - 8.0, 12.0, Color::from_hex(0x888888));
}

fn draw_day_summary(summary: &crate::game::state::DaySummary) {
    let sw = screen_width();
    let sh = screen_height();
    let is_season_end = summary.season_ended.is_some();
    let bw = 320.0;
    let bh = if is_season_end { 200.0 } else { 160.0 };
    let bx = sw / 2.0 - bw / 2.0;
    let by = sh / 2.0 - bh / 2.0;

    draw_rectangle(bx, by, bw, bh, Color::from_hex(0x1a1a2e));
    draw_rectangle_lines(bx, by, bw, bh, 2.0, WHITE);

    if let Some(season) = &summary.season_ended {
        let title = format!("{} Complete!", season);
        draw_text(&title, bx + 60.0, by + 34.0, 22.0, Color::from_hex(0xf1c40f));
        draw_text("All unharvested crops are gone.", bx + 20.0, by + 62.0, 14.0, Color::from_hex(0xcc8844));
        draw_text(&format!("Gold earned: {}g", summary.gold_earned), bx + 20.0, by + 92.0, 18.0, WHITE);
        draw_text(&format!("Crops shipped: {}", summary.crops_shipped), bx + 20.0, by + 118.0, 18.0, WHITE);
        draw_text("Press E to continue", bx + 70.0, by + 168.0, 14.0, Color::from_hex(0xaaaaaa));
    } else {
        draw_text(&format!("Day {} Complete", summary.day), bx + 60.0, by + 34.0, 22.0, Color::from_hex(0xf1c40f));
        draw_text(&format!("Gold earned: {}g", summary.gold_earned), bx + 20.0, by + 70.0, 18.0, WHITE);
        draw_text(&format!("Crops shipped: {}", summary.crops_shipped), bx + 20.0, by + 96.0, 18.0, WHITE);
        draw_text("Press E to continue", bx + 60.0, by + 136.0, 14.0, Color::from_hex(0xaaaaaa));
    }
}

fn draw_world(state: &GameState, camera: &Camera, _is_right_half: bool) {
    if state.raining {
        farm_view::draw_rain(&state.map, camera, state.house_upgraded, state.clock.hour);
    } else if state.rainbow_day {
        farm_view::draw_rainbow(&state.map, camera, state.house_upgraded, state.clock.hour);
    } else {
        farm_view::draw(&state.map, camera, state.house_upgraded, state.clock.hour);
    }
    animal_view::draw_farm_animals(state, camera);
    npc_view::draw(&state.npcs, camera);
    draw_birds(&state.birds, camera);
    draw_squirrels(&state.squirrels, camera);

    if state.coop_active {
        // Co-op: draw both players at their world positions (camera follows midpoint)
        let (p1x, p1y) = camera.world_to_screen(state.player.tile.0, state.player.tile.1);
        draw_p1_at_world(state, p1x, p1y);
        let (p2x, p2y) = camera.world_to_screen(state.player2.tile.0, state.player2.tile.1);
        draw_p2_character(state, p2x, p2y);
    } else {
        // Solo: draw P1 at screen center (classic behavior)
        player_view::draw_player_with_leap(&state.player, state.riding_horse, state.horse_leap_height);
    }
}

fn draw_p1_at_world(state: &GameState, x: f32, y: f32) {
    use crate::game::state::OUTFITS;
    let p = &state.player;
    let o = &OUTFITS[p.outfit.min(OUTFITS.len() as u8 - 1) as usize];
    let shirt = Color::new(o.shirt.0 as f32/255.0, o.shirt.1 as f32/255.0, o.shirt.2 as f32/255.0, 1.0);
    let pants = Color::new(o.pants.0 as f32/255.0, o.pants.1 as f32/255.0, o.pants.2 as f32/255.0, 1.0);
    let shoes = Color::new(o.shoes.0 as f32/255.0, o.shoes.1 as f32/255.0, o.shoes.2 as f32/255.0, 1.0);
    let hat_c = Color::new(o.hat.0 as f32/255.0, o.hat.1 as f32/255.0, o.hat.2 as f32/255.0, 1.0);
    let skin = Color { r: 0.96, g: 0.80, b: 0.62, a: 1.0 };
    let hair = player_view::player_hair_color(p);
    let y_off = y - state.horse_leap_height;

    if state.riding_horse {
        player_view::draw_horse_mount_at(x, y_off, &p.facing);
        let py = y_off - 14.0;
        player_view::draw_character(x, py, shirt, pants, shoes, skin, hair, &p.facing);
        if p.gender == 1 && p.hairstyle > 0 { player_view::draw_hairstyle(x, py, hair, p.hairstyle); }
        if p.gender == 0 { draw_rectangle(x + 5.0, py - 2.0, 22.0, 4.0, hat_c); draw_rectangle(x + 9.0, py - 9.0, 14.0, 8.0, hat_c); }
    } else {
        player_view::draw_character(x, y_off, shirt, pants, shoes, skin, hair, &p.facing);
        if p.gender == 1 && p.hairstyle > 0 { player_view::draw_hairstyle(x, y_off, hair, p.hairstyle); }
        if p.gender == 0 { draw_rectangle(x + 5.0, y_off - 2.0, 22.0, 4.0, hat_c); draw_rectangle(x + 9.0, y_off - 9.0, 14.0, 8.0, hat_c); }
    }
    // P1 yellow arrow
    let ax = x + 16.0;
    let ay = y_off - 16.0;
    draw_line(ax, ay - 6.0, ax, ay, 2.0, Color::from_hex(0xf1c40f));
    draw_triangle(Vec2::new(ax, ay + 3.0), Vec2::new(ax - 4.0, ay - 2.0), Vec2::new(ax + 4.0, ay - 2.0), Color::from_hex(0xf1c40f));
}

fn draw_p2_character(state: &GameState, x: f32, y: f32) {
    use crate::game::state::OUTFITS;
    let p2 = &state.player2;
    let o = &OUTFITS[p2.outfit.min(OUTFITS.len() as u8 - 1) as usize];
    let shirt = Color::new(o.shirt.0 as f32/255.0, o.shirt.1 as f32/255.0, o.shirt.2 as f32/255.0, 1.0);
    let pants = Color::new(o.pants.0 as f32/255.0, o.pants.1 as f32/255.0, o.pants.2 as f32/255.0, 1.0);
    let shoes = Color::new(o.shoes.0 as f32/255.0, o.shoes.1 as f32/255.0, o.shoes.2 as f32/255.0, 1.0);
    let hat_c = Color::new(o.hat.0 as f32/255.0, o.hat.1 as f32/255.0, o.hat.2 as f32/255.0, 1.0);
    let skin = Color { r: 0.96, g: 0.80, b: 0.62, a: 1.0 };
    let hair = player_view::player_hair_color(p2);
    player_view::draw_character(x, y, shirt, pants, shoes, skin, hair, &p2.facing);
    if p2.gender == 1 && p2.hairstyle > 0 {
        player_view::draw_hairstyle(x, y, hair, p2.hairstyle);
    }
    if p2.gender == 0 {
        // Hat for male P2
        draw_rectangle(x + 5.0, y - 2.0, 22.0, 4.0, hat_c);
        draw_rectangle(x + 9.0, y - 9.0, 14.0, 8.0, hat_c);
    }
    // P2 indicator arrow (blue instead of yellow)
    let ax = x + 16.0;
    let ay = y - 16.0;
    draw_line(ax, ay - 6.0, ax, ay, 2.0, Color::from_hex(0x3498db));
    draw_triangle(
        Vec2::new(ax, ay + 3.0),
        Vec2::new(ax - 4.0, ay - 2.0),
        Vec2::new(ax + 4.0, ay - 2.0),
        Color::from_hex(0x3498db),
    );
}

fn draw_squirrels(squirrels: &[Squirrel], camera: &Camera) {
    for sq in squirrels {
        let sx = sq.x - camera.offset_x();
        let sy = sq.y - camera.offset_y();

        if sx < -32.0 || sx > screen_width() + 32.0 || sy < -32.0 || sy > screen_height() + 32.0 {
            continue;
        }

        let brown = Color::from_hex(0x8b5e3c);
        let belly = Color::from_hex(0xd4a870);
        let dark = Color::from_hex(0x5a3a1a);

        // Determine facing direction based on movement
        let going_right = if sq.phase == 0 { sq.target_x > sq.home_x } else { sq.home_x > sq.target_x };
        let dir = if going_right { 1.0f32 } else { -1.0 };

        // Body
        draw_circle(sx, sy, 5.0, brown);
        draw_circle(sx + dir * 3.0, sy + 1.0, 4.0, brown);
        // Belly
        draw_circle(sx, sy + 1.0, 3.0, belly);
        // Head
        draw_circle(sx + dir * 7.0, sy - 2.0, 3.5, brown);
        // Ear
        draw_circle(sx + dir * 6.0, sy - 5.0, 1.5, brown);
        // Eye
        draw_circle(sx + dir * 8.0, sy - 2.5, 1.0, Color::from_hex(0x1a1a1a));
        draw_circle(sx + dir * 8.2, sy - 2.8, 0.4, WHITE);
        // Nose
        draw_circle(sx + dir * 10.0, sy - 1.5, 0.8, Color::from_hex(0x333333));
        // Tail — big fluffy curve
        let tail_x = sx - dir * 5.0;
        draw_line(tail_x, sy, tail_x - dir * 2.0, sy - 6.0, 3.0, brown);
        draw_line(tail_x - dir * 2.0, sy - 6.0, tail_x + dir * 2.0, sy - 10.0, 2.5, brown);
        draw_circle(tail_x + dir * 2.0, sy - 10.0, 2.0, brown);
        // Legs (little feet)
        if sq.phase != 1 {
            // Running animation
            let leg_offset = (sq.timer * 15.0).sin() * 3.0;
            draw_rectangle(sx - 2.0, sy + 4.0 + leg_offset, 2.0, 3.0, dark);
            draw_rectangle(sx + 2.0, sy + 4.0 - leg_offset, 2.0, 3.0, dark);
        } else {
            // Sitting/pausing — upright pose
            draw_rectangle(sx - 2.0, sy + 3.0, 2.0, 4.0, dark);
            draw_rectangle(sx + 2.0, sy + 3.0, 2.0, 4.0, dark);
        }
    }
}

fn draw_birds(birds: &[Bird], camera: &Camera) {
    use crate::render::camera::TILE_SIZE;

    let bird_colors: [Color; 3] = [
        Color::from_hex(0x4a3a2a), // brown sparrow
        Color::from_hex(0x2a4a6a), // blue jay
        Color::from_hex(0x8a2a2a), // robin
    ];

    for bird in birds {
        // Skip birds that are respawning (invisible)
        if !bird.flying && bird.respawn_timer > 0.0 {
            continue;
        }

        let sx = bird.x - camera.offset_x();
        let sy = bird.y - camera.offset_y();

        // Cull off-screen birds
        if sx < -32.0 || sx > screen_width() + 32.0 || sy < -32.0 || sy > screen_height() + 32.0 {
            continue;
        }

        let color = bird_colors[bird.variant as usize % 3];

        if bird.flying {
            // Flying bird — wings spread, smaller as it rises
            let scale = (1.0 - bird.fly_timer / 1.5).max(0.2);
            let wing_flap = (bird.fly_timer * 12.0).sin() * 4.0 * scale;
            // Body
            draw_circle(sx, sy, 3.0 * scale, color);
            // Wings (flapping)
            draw_line(sx - 6.0 * scale, sy + wing_flap, sx, sy, 2.0 * scale, color);
            draw_line(sx + 6.0 * scale, sy - wing_flap, sx, sy, 2.0 * scale, color);
        } else {
            // Grounded bird — pecking animation
            let bob = ((bird.x * 3.0 + bird.y * 7.0 + bird.fly_timer * 2.0).sin() * 1.5).abs();
            // Body
            draw_circle(sx, sy - bob, 3.5, color);
            // Head
            draw_circle(sx + 3.0, sy - 3.0 - bob, 2.5, color);
            // Beak
            draw_line(sx + 5.0, sy - 3.0 - bob, sx + 8.0, sy - 2.0 - bob, 1.5, Color::from_hex(0xf39c12));
            // Tail
            draw_line(sx - 3.0, sy - bob, sx - 6.0, sy - 3.0 - bob, 1.5, color);
            // Legs
            draw_line(sx - 1.0, sy + 1.0, sx - 2.0, sy + 4.0, 1.0, Color::from_hex(0x8b5e3c));
            draw_line(sx + 1.0, sy + 1.0, sx + 2.0, sy + 4.0, 1.0, Color::from_hex(0x8b5e3c));
        }
    }
}
