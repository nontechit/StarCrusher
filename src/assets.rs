use macroquad::prelude::*;

thread_local! {
    static CURRENT_COLOR: std::cell::Cell<Color> = const { std::cell::Cell::new(WHITE) };
}

struct TextMeasure {
    w: f32,
    h: f32,
}

fn set_color(color: Color) {
    CURRENT_COLOR.with(|current| current.set(color));
}

fn set_default_color() {
    set_color(WHITE);
}

fn current_color() -> Color {
    CURRENT_COLOR.with(|current| current.get())
}

fn draw_rect(x: f32, y: f32, w: f32, h: f32) {
    macroquad::prelude::draw_rectangle(x, y, w, h, current_color());
}

fn draw_triangle(x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32) {
    macroquad::prelude::draw_triangle(vec2(x1, y1), vec2(x2, y2), vec2(x3, y3), current_color());
}

fn draw_circle(x: f32, y: f32, r: f32) {
    macroquad::prelude::draw_circle(x, y, r, current_color());
}

fn draw_rectangle(x: f32, y: f32, w: f32, h: f32) {
    macroquad::prelude::draw_rectangle(x, y, w, h, current_color());
}

fn draw_text(text: &str, x: f32, y: f32, font_size: f32) {
    macroquad::prelude::draw_text(text, x, y, font_size, current_color());
}

fn measure_text(text: &str, font: Option<&Font>, font_size: u16, font_scale: f32) -> TextMeasure {
    let measure = macroquad::prelude::measure_text(text, font, font_size, font_scale);
    TextMeasure {
        w: measure.width,
        h: measure.height,
    }
}

/// Draws a starfield background with randomly placed stars.
#[allow(dead_code)]
pub fn draw_background() {
    clear_background(BLACK);
}

/// Draws the player ship at (x, y) using procedural shapes.
pub fn draw_player_ship(x: f32, y: f32, scale: f32) {
    let s = 16.0 * scale;

    // Ship body - triangle pointing up
    set_color(Color::new(0.2, 0.8, 1.0, 1.0));
    draw_triangle(x + s / 2.0, y - s, x, y + s / 2.0, x + s, y + s / 2.0);

    // Cockpit
    set_color(Color::new(0.5, 1.0, 1.0, 1.0));
    draw_circle(x + s / 2.0, y - s * 0.2, s * 0.2);

    // Engine glow
    let flicker = get_time() as f32 % 1.0;
    set_color(Color::new(
        1.0,
        0.4,
        0.1,
        (flicker.sin() * 0.5 + 0.7).min(1.0),
    ));
    draw_circle(x + s / 2.0, y + s * 0.6, s * 0.15);

    set_default_color();
}

/// Draws a standard enemy invader at (x, y) with the given color and scale.
pub fn draw_enemy_invader(x: f32, y: f32, color: Color, scale: f32) {
    let s = 14.0 * scale;

    // Main body
    set_color(color);
    draw_rect(x + s * 0.25, y, s * 0.5, s * 0.6);

    // Side arms (animated slightly based on time)
    let arm_offset = if (get_time() as f32 * 4.0).fract() > 0.5 {
        1.0
    } else {
        -1.0
    };
    draw_rect(x, y + s * 0.2 + arm_offset, s * 0.25, s * 0.3);
    draw_rect(x + s * 0.75, y + s * 0.2 + arm_offset, s * 0.25, s * 0.3);

    // Eyes (dark)
    set_color(BLACK);
    draw_circle(x + s * 0.35, y + s * 0.25, s * 0.08);
    draw_circle(x + s * 0.65, y + s * 0.25, s * 0.08);

    // Legs (animated)
    set_color(color);
    let leg_offset = if arm_offset > 0.0 { -1.0 } else { 1.0 };
    draw_rect(x + s * 0.2, y + s * 0.6 + leg_offset, s * 0.15, s * 0.3);
    draw_rect(x + s * 0.65, y + s * 0.6 + leg_offset, s * 0.15, s * 0.3);

    set_default_color();
}

/// Draws a puzzle target as a large answer number.
pub fn draw_puzzle_enemy(x: f32, y: f32, color: Color, scale: f32, answer_number: i64) {
    let target_w = 44.0 * scale;
    let target_h = 34.0 * scale;
    let txt = format!("{}", answer_number);
    let font_size = if txt.len() >= 3 { 24.0 } else { 30.0 } * scale;
    let tm = measure_text(&txt, None, font_size as u16, 1.0);

    let text_x = x + target_w / 2.0 - tm.w / 2.0;
    let text_y = y + target_h / 2.0 + tm.h * 0.35;

    set_color(Color::new(color.r, color.g, color.b, 0.25));
    draw_rectangle(x - 3.0, y - 2.0, target_w + 6.0, target_h + 4.0);

    set_color(BLACK);
    draw_text(&txt, text_x + 2.0, text_y + 2.0, font_size);

    set_color(color);
    draw_text(&txt, text_x - 1.0, text_y, font_size);
    draw_text(&txt, text_x + 1.0, text_y, font_size);

    set_color(WHITE);
    draw_text(&txt, text_x, text_y, font_size);

    set_default_color();
}

/// Draws a bullet (player projectile).
pub fn draw_bullet(x: f32, y: f32) {
    let flicker = get_time() as f32 % 1.0;
    // Bright core
    set_color(Color::new(1.0, 1.0, 0.5, 1.0));
    draw_circle(x, y, 2.0);

    // Glow trail
    set_color(Color::new(1.0, 0.8, 0.3, flicker.sin() * 0.4 + 0.6));
    draw_rectangle(x - 1.0, y, 2.0, 8.0);

    set_default_color();
}

/// Draws an enemy bullet descending toward the player.
pub fn draw_enemy_bullet(x: f32, y: f32) {
    let flicker = get_time() as f32 % 1.0;
    set_color(Color::new(1.0, 0.3, 0.3, flicker.sin() * 0.4 + 0.6));
    draw_circle(x, y, 2.5);
    draw_rectangle(x - 1.0, y - 8.0, 2.0, 8.0);

    set_default_color();
}

/// Draws an explosion effect at (x, y) with given radius and alpha fade.
pub fn draw_explosion(x: f32, y: f32, progress: f32) {
    let max_radius = 15.0;
    let r = max_radius * progress.min(1.0);
    let alpha = (1.0 - progress).max(0.0);

    set_color(Color::new(1.0, 0.6, 0.2, alpha));
    draw_circle(x, y, r);

    if progress < 0.5 {
        set_color(Color::new(1.0, 1.0, 0.8, alpha * 0.7));
        draw_circle(x, y, r * 0.6);
    }

    set_default_color();
}

/// Draws a decorative border around the play area.
pub fn draw_border(screen_w: f32, screen_h: f32) {
    let thickness = 4.0;
    let color = Color::new(0.15, 0.15, 0.3, 1.0);

    set_color(color);
    // Top and bottom borders
    draw_rectangle(0.0, 0.0, screen_w, thickness);
    draw_rectangle(0.0, screen_h - thickness, screen_w, thickness);
    // Left and right borders
    draw_rectangle(0.0, 0.0, thickness, screen_h);
    draw_rectangle(screen_w - thickness, 0.0, thickness, screen_h);

    set_default_color();
}

/// Draws a star at the given position with optional twinkle effect.
pub fn draw_star(x: f32, y: f32, size: f32) {
    let brightness = (get_time() as f32 * 2.0 + x).sin() * 0.3 + 0.7;
    set_color(Color::new(brightness, brightness, brightness.min(1.0), 0.8));
    draw_circle(x, y, size);
    set_default_color();
}

/// Draws a shield/power-up icon at (x, y).
#[allow(dead_code)]
pub fn draw_shield_icon(x: f32, y: f32) {
    let s = 12.0;
    // Shield shape - hexagon-ish
    set_color(Color::new(0.3, 0.8, 0.9, 0.7));
    draw_circle(x + s / 2.0, y + s / 2.0, s * 0.6);

    // S symbol inside
    let txt = "S";
    let tm = measure_text(txt, None, 14, 1.0);
    set_color(WHITE);
    draw_text(txt, x + s - tm.w / 2.0, y + s * 0.75, 14.0);

    set_default_color();
}

/// Draws a life icon (mini ship) at the given position.
pub fn draw_life_icon(x: f32, y: f32) {
    let s = 8.0;
    set_color(Color::new(0.2, 0.8, 1.0, 1.0));
    draw_triangle(x + s / 2.0, y - s * 0.5, x, y + s * 0.5, x + s, y + s * 0.5);
    set_default_color();
}
