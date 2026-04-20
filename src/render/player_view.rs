use macroquad::prelude::*;
use crate::game::player::{Direction, Player};
use crate::render::camera::TILE_SIZE;

const SKIN: Color  = Color { r: 0.96, g: 0.80, b: 0.62, a: 1.0 };
const HAIR: Color  = Color { r: 0.32, g: 0.18, b: 0.08, a: 1.0 };
const SHIRT: Color = Color { r: 0.90, g: 0.92, b: 1.00, a: 1.0 }; // light periwinkle
const PANTS: Color = Color { r: 0.22, g: 0.32, b: 0.62, a: 1.0 }; // denim
const SHOES: Color = Color { r: 0.22, g: 0.14, b: 0.08, a: 1.0 };

const HAT: Color = Color { r: 0.15, g: 0.55, b: 0.25, a: 1.0 }; // green farmer hat

/// Player is always drawn at the centre of the screen.
pub fn draw(player: &Player) {
    draw_player_with_leap(player, false, 0.0);
}

pub fn draw_player(player: &Player, riding: bool) {
    draw_player_with_leap(player, riding, 0.0);
}

fn outfit_colors(player: &Player) -> (Color, Color, Color, Color) {
    use crate::game::state::OUTFITS;
    let o = &OUTFITS[player.outfit.min(OUTFITS.len() as u8 - 1) as usize];
    (
        Color::new(o.shirt.0 as f32 / 255.0, o.shirt.1 as f32 / 255.0, o.shirt.2 as f32 / 255.0, 1.0),
        Color::new(o.pants.0 as f32 / 255.0, o.pants.1 as f32 / 255.0, o.pants.2 as f32 / 255.0, 1.0),
        Color::new(o.shoes.0 as f32 / 255.0, o.shoes.1 as f32 / 255.0, o.shoes.2 as f32 / 255.0, 1.0),
        Color::new(o.hat.0 as f32 / 255.0, o.hat.1 as f32 / 255.0, o.hat.2 as f32 / 255.0, 1.0),
    )
}

pub fn draw_player_with_leap(player: &Player, riding: bool, leap_offset: f32) {
    let sw = screen_width();
    let sh = screen_height();
    let x = sw / 2.0 - TILE_SIZE / 2.0;
    let y = sh / 2.0 - TILE_SIZE / 2.0 - leap_offset;

    let (shirt, pants, shoes, hat) = outfit_colors(player);
    let hair = player_hair_color(player);

    if riding {
        let ground_y = sh / 2.0 - TILE_SIZE / 2.0 + 28.0;
        let shadow_scale = 1.0 - (leap_offset / 20.0).min(0.5);
        draw_rectangle(
            sw / 2.0 - 14.0 * shadow_scale, ground_y,
            28.0 * shadow_scale, 4.0,
            Color { r: 0.0, g: 0.0, b: 0.0, a: 0.2 * shadow_scale },
        );

        draw_horse_mount(x, y, &player.facing);
        let py = y - 14.0;
        draw_character(x, py, shirt, pants, shoes, SKIN, hair, &player.facing);
        if player.gender == 1 && player.hairstyle > 0 {
            draw_hairstyle(x, py, hair, player.hairstyle);
        }
        if player.gender == 0 { draw_hat(x, py, hat); }
        draw_player_indicator(x, py);
    } else {
        draw_character(x, y, shirt, pants, shoes, SKIN, hair, &player.facing);
        if player.gender == 1 && player.hairstyle > 0 {
            draw_hairstyle(x, y, hair, player.hairstyle);
        }
        if player.gender == 0 { draw_hat(x, y, hat); }
        draw_player_indicator(x, y);
    }
}

pub fn draw_horse_mount_at(x: f32, y: f32, facing: &Direction) {
    draw_horse_mount(x, y, facing);
}

fn draw_horse_mount(x: f32, y: f32, facing: &Direction) {
    let white = Color { r: 0.94, g: 0.93, b: 0.89, a: 1.0 };
    let silver = Color { r: 0.82, g: 0.82, b: 0.82, a: 1.0 };

    // Horse body under player — facing-aware
    let flip = matches!(facing, Direction::Left);
    let fx = if flip { x + TILE_SIZE } else { x };
    let dir = if flip { -1.0 } else { 1.0 };

    // Body
    draw_rectangle(fx + dir * 2.0, y + 10.0, dir * 28.0, 14.0, white);
    draw_circle(fx + dir * 2.0, y + 17.0, 7.0, white);
    draw_circle(fx + dir * 30.0, y + 17.0, 7.0, white);

    // Head + neck (front)
    draw_line(fx + dir * 28.0, y + 10.0, fx + dir * 34.0, y + 2.0, 5.0, white);
    draw_rectangle(fx + dir * 31.0 - 4.0, y - 2.0, 8.0, 6.0, white);

    // Eye
    draw_circle(fx + dir * 33.0, y, 1.0, Color::from_hex(0x1a1a1a));

    // Legs
    for &lx in &[4.0, 10.0, 20.0, 26.0] {
        draw_rectangle(fx + dir * lx - 1.0, y + 24.0, 2.5, 8.0, white);
        draw_rectangle(fx + dir * lx - 1.5, y + 31.0, 3.5, 2.0, Color::from_hex(0x888888));
    }

    // Mane
    for i in 0..3 {
        draw_line(fx + dir * (26.0 - i as f32 * 2.0), y + 4.0 + i as f32 * 3.0,
                  fx + dir * (24.0 - i as f32 * 2.0), y + 2.0 + i as f32 * 3.0,
                  2.0, silver);
    }

    // Tail
    draw_line(fx + dir * 0.0, y + 14.0, fx + dir * (-4.0), y + 20.0, 2.5, silver);
    draw_line(fx + dir * (-4.0), y + 20.0, fx + dir * (-6.0), y + 26.0, 2.0, silver);
}

pub fn player_hair_color(player: &Player) -> Color {
    use crate::game::state::HAIR_COLORS;
    let idx = (player.hair_color as usize).min(HAIR_COLORS.len() - 1);
    let (r, g, b, _) = HAIR_COLORS[idx];
    Color::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0)
}

/// Draw extra hair details for female hairstyles (called after draw_character).
pub fn draw_hairstyle(x: f32, y: f32, hair: Color, style: u8) {
    match style {
        1 => {
            // Long — hair flows down past shoulders
            draw_rectangle(x + 6.0, y + 2.0, 5.0, 18.0, hair);
            draw_rectangle(x + 21.0, y + 2.0, 5.0, 18.0, hair);
            draw_circle(x + 8.0, y + 20.0, 3.0, hair);
            draw_circle(x + 24.0, y + 20.0, 3.0, hair);
        }
        2 => {
            // Ponytail — bun at back
            draw_circle(x + 16.0, y - 2.0, 5.0, hair);
            draw_rectangle(x + 13.0, y - 2.0, 6.0, 4.0, hair);
            // Tail hanging down
            draw_rectangle(x + 22.0, y + 2.0, 4.0, 14.0, hair);
            draw_circle(x + 24.0, y + 16.0, 2.5, hair);
            // Hair tie
            draw_rectangle(x + 21.0, y + 3.0, 6.0, 2.0, Color::from_hex(0xe74c3c));
        }
        3 => {
            // Curly — poofy sides
            draw_circle(x + 7.0, y + 5.0, 6.0, hair);
            draw_circle(x + 25.0, y + 5.0, 6.0, hair);
            draw_circle(x + 6.0, y + 12.0, 4.0, hair);
            draw_circle(x + 26.0, y + 12.0, 4.0, hair);
            draw_circle(x + 10.0, y + 14.0, 3.0, hair);
            draw_circle(x + 22.0, y + 14.0, 3.0, hair);
        }
        4 => {
            // Braids — two braids hanging down
            draw_rectangle(x + 5.0, y + 2.0, 4.0, 22.0, hair);
            draw_rectangle(x + 23.0, y + 2.0, 4.0, 22.0, hair);
            // Braid pattern (alternating bumps)
            for i in 0..4 {
                let by = y + 6.0 + i as f32 * 5.0;
                draw_circle(x + 6.0 + (i % 2) as f32 * 2.0, by, 2.5, hair);
                draw_circle(x + 24.0 + (i % 2) as f32 * 2.0, by, 2.5, hair);
            }
            // Hair ties at bottom
            draw_circle(x + 7.0, y + 24.0, 2.0, Color::from_hex(0x3498db));
            draw_circle(x + 25.0, y + 24.0, 2.0, Color::from_hex(0x3498db));
        }
        5 => {
            // Bob — short rounded sides
            draw_circle(x + 8.0, y + 6.0, 5.0, hair);
            draw_circle(x + 24.0, y + 6.0, 5.0, hair);
            draw_rectangle(x + 6.0, y + 2.0, 4.0, 10.0, hair);
            draw_rectangle(x + 22.0, y + 2.0, 4.0, 10.0, hair);
        }
        6 => {
            // Headband — hair flowing out with a band across
            draw_rectangle(x + 6.0, y + 2.0, 4.0, 14.0, hair);
            draw_rectangle(x + 22.0, y + 2.0, 4.0, 14.0, hair);
            draw_circle(x + 8.0, y + 16.0, 3.0, hair);
            draw_circle(x + 24.0, y + 16.0, 3.0, hair);
            // Headband
            draw_rectangle(x + 7.0, y + 1.0, 18.0, 3.0, Color::from_hex(0xe74c3c));
            // Small bow on side
            draw_circle(x + 24.0, y + 1.0, 2.5, Color::from_hex(0xe74c3c));
            draw_circle(x + 27.0, y + 0.0, 2.0, Color::from_hex(0xc0392b));
        }
        _ => {
            // Short (0) — default, no extra drawing needed
        }
    }
}

fn draw_hat(x: f32, y: f32, color: Color) {
    // Brim
    draw_rectangle(x + 5.0, y - 2.0, 22.0, 4.0, color);
    // Crown
    draw_rectangle(x + 9.0, y - 9.0, 14.0, 8.0, color);
    // Band
    draw_rectangle(x + 9.0, y - 3.0, 14.0, 2.0, Color::from_hex(0xf1c40f));
}

fn draw_player_indicator(x: f32, y: f32) {
    // Small downward-pointing arrow above the hat
    let ax = x + TILE_SIZE / 2.0;
    let ay = y - 16.0;
    // Arrow body
    draw_line(ax, ay - 6.0, ax, ay, 2.0, Color::from_hex(0xf1c40f));
    // Arrow head
    draw_triangle(
        Vec2::new(ax, ay + 3.0),
        Vec2::new(ax - 4.0, ay - 2.0),
        Vec2::new(ax + 4.0, ay - 2.0),
        Color::from_hex(0xf1c40f),
    );
}

/// Draw a humanoid character sprite inside a TILE_SIZE×TILE_SIZE cell.
/// `outfit` is the shirt/body color; used for NPCs too.
pub fn draw_character(
    x: f32, y: f32,
    outfit: Color, pants: Color, shoes: Color,
    skin: Color, hair: Color,
    facing: &Direction,
) {
    // Drop shadow
    draw_rectangle(
        x + 6.0, y + 28.0, 20.0, 4.0,
        Color { r: 0.0, g: 0.0, b: 0.0, a: 0.15 },
    );

    // Shoes
    draw_rectangle(x + 6.0,  y + 24.0, 8.0, 5.0, shoes);
    draw_rectangle(x + 18.0, y + 24.0, 8.0, 5.0, shoes);

    // Pants
    draw_rectangle(x + 7.0, y + 17.0, 18.0, 8.0, pants);
    // Pants leg split
    draw_line(x + 16.0, y + 17.0, x + 16.0, y + 25.0, 1.0,
              Color { r: 0.0, g: 0.0, b: 0.0, a: 0.18 });

    // Shirt / body
    draw_rectangle(x + 5.0, y + 10.0, 22.0, 9.0, outfit);

    // Arms
    draw_rectangle(x + 1.0,  y + 11.0, 5.0, 7.0, outfit);
    draw_rectangle(x + 26.0, y + 11.0, 5.0, 7.0, outfit);
    // Hands
    draw_circle(x + 3.5,  y + 19.0, 3.0, skin);
    draw_circle(x + 28.5, y + 19.0, 3.0, skin);

    // Neck
    draw_rectangle(x + 13.0, y + 8.0, 6.0, 4.0, skin);

    // Head (skin)
    draw_circle(x + 16.0, y + 7.0, 7.5, skin);

    // Hair
    draw_circle(x + 16.0, y + 2.5, 7.0, hair);
    draw_rectangle(x + 9.0, y + 2.0, 14.0, 7.0, hair);

    // Face detail based on direction
    match facing {
        Direction::Up => {
            // Back of head — no face visible
        }
        Direction::Down | Direction::Left | Direction::Right => {
            // Eyes
            draw_rectangle(x + 11.0, y + 7.0, 3.0, 3.0, WHITE);
            draw_rectangle(x + 18.0, y + 7.0, 3.0, 3.0, WHITE);
            draw_rectangle(x + 12.0, y + 8.0, 2.0, 2.0, Color::from_hex(0x2a1800));
            draw_rectangle(x + 19.0, y + 8.0, 2.0, 2.0, Color::from_hex(0x2a1800));
            // Smile
            draw_line(x + 13.0, y + 12.0, x + 19.0, y + 12.0, 1.0, Color::from_hex(0x9b5020));
        }
    }
}
