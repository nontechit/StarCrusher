use crate::assets;
use crate::levels::Grade;
use crate::screen;
use macroquad::prelude::*;

fn center_x() -> f32 {
    screen::screen_w() / 2.0
}
pub const TITLE_MENU_X: f32 = 720.0;
pub const TITLE_MENU_Y: f32 = 190.0;
pub const TITLE_MENU_W: f32 = 420.0;
pub const TITLE_MENU_ROW_TOP: f32 = 270.0;
pub const TITLE_MENU_ROW_H: f32 = 54.0;
pub const TITLE_MENU_ROW_GAP: f32 = 18.0;
pub const MOBILE_TITLE_MENU_X: f32 = 24.0;
pub const MOBILE_TITLE_MENU_ROW_TOP: f32 = 640.0;
pub const MOBILE_TITLE_MENU_W: f32 = 672.0;
pub const MOBILE_TITLE_MENU_ROW_H: f32 = 132.0;
pub const MOBILE_TITLE_MENU_ROW_GAP: f32 = 24.0;
pub const KEYPAD_X: f32 = 920.0;
pub const KEYPAD_Y: f32 = 414.0;
pub const KEYPAD_KEY: f32 = 54.0;
pub const KEYPAD_GAP: f32 = 8.0;
pub const GATE_QUESTION_X: f32 = 210.0;
pub const GATE_QUESTION_Y: f32 = 438.0;
pub const GATE_QUESTION_W: f32 = 620.0;
pub const GATE_QUESTION_LINE_GAP: f32 = 34.0;
pub const MOBILE_GUTTER: f32 = 24.0;
pub const MOBILE_CHROME_Y: f32 = 8.0;
pub const MOBILE_CHROME_ROW_H: f32 = 80.0;
pub const MOBILE_BACK_X: f32 = 24.0;
pub const MOBILE_BACK_Y: f32 = 24.0;
pub const MOBILE_BACK_W: f32 = 126.0;
pub const MOBILE_BACK_H: f32 = 52.0;
pub const MOBILE_ACTION_X: f32 = 24.0;
pub const MOBILE_ACTION_Y: f32 = 1040.0;
pub const MOBILE_ACTION_W: f32 = 672.0;
pub const MOBILE_ACTION_H: f32 = 136.0;
pub const SPELLING_PLAY_X: f32 = 390.0;
pub const SPELLING_NIGHTMARE_X: f32 = 670.0;
pub const SPELLING_ACTION_Y: f32 = 536.0;
pub const SPELLING_ACTION_W: f32 = 220.0;
pub const SPELLING_ACTION_H: f32 = 74.0;
pub const MOBILE_SPELLING_PLAY_Y: f32 = 864.0;
pub const MOBILE_SPELLING_NIGHTMARE_Y: f32 = 1024.0;

fn mobile_safe_x() -> f32 {
    MOBILE_GUTTER
}

fn mobile_safe_w() -> f32 {
    (screen::screen_w() - MOBILE_GUTTER * 2.0).max(1.0)
}

thread_local! {
    static CURRENT_COLOR: std::cell::Cell<Color> = const { std::cell::Cell::new(WHITE) };
}

struct TextMeasure {
    w: f32,
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

fn draw_rectangle(x: f32, y: f32, w: f32, h: f32) {
    macroquad::prelude::draw_rectangle(x, y, w, h, current_color());
}

fn draw_rectangle_lines(x: f32, y: f32, w: f32, h: f32) {
    macroquad::prelude::draw_rectangle_lines(x, y, w, h, 2.0, current_color());
}

fn draw_circle(x: f32, y: f32, r: f32) {
    macroquad::prelude::draw_circle(x, y, r, current_color());
}

fn draw_rounded_rect(x: f32, y: f32, w: f32, h: f32, radius: f32, color: Color) {
    let r = radius.min(w / 2.0).min(h / 2.0);
    set_color(color);
    draw_rectangle(x + r, y, w - r * 2.0, h);
    draw_rectangle(x, y + r, w, h - r * 2.0);
    draw_circle(x + r, y + r, r);
    draw_circle(x + w - r, y + r, r);
    draw_circle(x + r, y + h - r, r);
    draw_circle(x + w - r, y + h - r, r);
}

fn draw_rounded_panel(x: f32, y: f32, w: f32, h: f32, radius: f32, fill: Color, edge: Color) {
    draw_rounded_rect(x, y, w, h, radius, edge);
    draw_rounded_rect(x + 3.0, y + 3.0, w - 6.0, h - 6.0, radius - 3.0, fill);
}

fn measure_text(text: &str, font: Option<&Font>, font_size: u16, font_scale: f32) -> TextMeasure {
    let measure = macroquad::prelude::measure_text(text, font, font_size, font_scale);
    TextMeasure { w: measure.width }
}

fn fit_font_size(text: &str, desired: u16, max_width: f32, min_size: u16) -> u16 {
    if text.is_empty() {
        return desired;
    }

    let mut size = desired.max(min_size);
    while size > min_size && measure_text(text, None, size, 1.0).w > max_width {
        size -= 1;
    }
    size
}

fn fit_mobile_font_size(text: &str, desired: u16, max_width: f32) -> u16 {
    if screen::portrait_layout() {
        fit_font_size(text, desired, max_width, 14)
    } else {
        desired
    }
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> Color {
    let a = s * l.min(1.0 - l);
    let channel = |n: f32| {
        let k = (n + h * 12.0) % 12.0;
        l - a * (k - 3.0).min(9.0 - k).clamp(-1.0, 1.0)
    };
    Color::new(channel(0.0), channel(8.0), channel(4.0), 1.0)
}

pub fn mobile_back_button_contains(point: Vec2) -> bool {
    screen::portrait_layout()
        && point.x >= MOBILE_BACK_X
        && point.x <= MOBILE_BACK_X + MOBILE_BACK_W
        && point.y >= MOBILE_BACK_Y
        && point.y <= MOBILE_BACK_Y + MOBILE_BACK_H
}

pub fn mobile_action_button_contains(point: Vec2) -> bool {
    screen::portrait_layout() && mobile_action_button_rect().contains(point)
}

pub fn mobile_action_button_rect() -> Rect {
    Rect::new(
        MOBILE_ACTION_X,
        MOBILE_ACTION_Y,
        MOBILE_ACTION_W,
        MOBILE_ACTION_H,
    )
}

pub fn mobile_action_button_rect_at(y: f32) -> Rect {
    Rect::new(MOBILE_ACTION_X, y, MOBILE_ACTION_W, MOBILE_ACTION_H)
}

pub fn mobile_button_rect_contains(point: Vec2, rect: Rect) -> bool {
    screen::portrait_layout()
        && point.x >= rect.x
        && point.x <= rect.x + rect.w
        && point.y >= rect.y
        && point.y <= rect.y + rect.h
}

pub fn spelling_play_button_contains(point: Vec2) -> bool {
    spelling_play_button_rect().contains(point)
}

pub fn spelling_nightmare_button_contains(point: Vec2) -> bool {
    spelling_nightmare_button_rect().contains(point)
}

pub fn spelling_play_button_rect() -> Rect {
    if screen::portrait_layout() {
        Rect::new(
            MOBILE_ACTION_X,
            MOBILE_SPELLING_PLAY_Y,
            MOBILE_ACTION_W,
            MOBILE_ACTION_H,
        )
    } else {
        Rect::new(
            SPELLING_PLAY_X,
            SPELLING_ACTION_Y,
            SPELLING_ACTION_W,
            SPELLING_ACTION_H,
        )
    }
}

pub fn spelling_nightmare_button_rect() -> Rect {
    if screen::portrait_layout() {
        Rect::new(
            MOBILE_ACTION_X,
            MOBILE_SPELLING_NIGHTMARE_Y,
            MOBILE_ACTION_W,
            MOBILE_ACTION_H,
        )
    } else {
        Rect::new(
            SPELLING_NIGHTMARE_X,
            SPELLING_ACTION_Y,
            SPELLING_ACTION_W,
            SPELLING_ACTION_H,
        )
    }
}

pub fn gate_question_x() -> f32 {
    if screen::portrait_layout() {
        50.0
    } else {
        GATE_QUESTION_X
    }
}

pub fn gate_question_w() -> f32 {
    if screen::portrait_layout() {
        screen::screen_w() - 100.0
    } else {
        GATE_QUESTION_W
    }
}

pub fn gate_question_y() -> f32 {
    if screen::portrait_layout() {
        438.0
    } else {
        GATE_QUESTION_Y
    }
}

pub fn gate_question_line_gap() -> f32 {
    if screen::portrait_layout() {
        42.0
    } else {
        GATE_QUESTION_LINE_GAP
    }
}

pub fn draw_mobile_back_button(label: &str) {
    if !screen::portrait_layout() {
        return;
    }
    if mobile_html_overlay_controls() {
        let _ = label;
        return;
    }

    draw_mobile_yellow_button_rect(
        Rect::new(MOBILE_BACK_X, MOBILE_BACK_Y, MOBILE_BACK_W, MOBILE_BACK_H),
        label,
        18,
    );
}

pub fn draw_mobile_action_button(label: &str) {
    if !screen::portrait_layout() {
        return;
    }
    if mobile_html_overlay_controls() {
        let _ = label;
        return;
    }

    draw_mobile_action_button_in_rect(label, mobile_action_button_rect());
}

pub fn draw_mobile_action_button_in_rect(label: &str, rect: Rect) {
    if !screen::portrait_layout() {
        return;
    }
    if mobile_html_overlay_controls() {
        let _ = (label, rect);
        return;
    }

    draw_mobile_yellow_button_rect(rect, label, 40);
}

fn draw_mobile_yellow_button_rect(rect: Rect, label: &str, desired_size: u16) {
    draw_rounded_rect(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        18.0,
        Color::new(0.85, 0.52, 0.13, 0.98),
    );
    draw_rounded_rect(
        rect.x + 3.0,
        rect.y + 3.0,
        rect.w - 6.0,
        rect.h - 6.0,
        15.0,
        Color::new(1.0, 0.84, 0.44, 0.99),
    );
    set_color(Color::new(0.78, 0.44, 0.08, 0.28));
    draw_rectangle(rect.x + 12.0, rect.y + rect.h - 14.0, rect.w - 24.0, 8.0);

    let font_size = fit_mobile_font_size(
        label,
        screen::mobile_text_size(desired_size)
            .min((rect.h * 0.52) as u16)
            .max(16),
        rect.w - 48.0,
    );
    centered_text_in(
        label,
        rect.x,
        rect.y + rect.h * 0.68,
        rect.w,
        font_size,
        Color::new(0.035, 0.06, 0.1, 1.0),
    );
    set_default_color();
}

/// Draws the heads-up display (HUD) at top of screen.
pub fn draw_hud(grade: &Grade, score: u32, lives: u8, wave: usize, question_text: Option<&str>) {
    if screen::portrait_layout() {
        draw_mobile_hud(grade, score, lives, wave, question_text);
        return;
    }

    let font_size = screen::mobile_text_size(16);

    // Grade level indicator (top-left)
    set_color(WHITE);
    draw_text(
        format!("GRADE: {}", grade.display_name()).as_str(),
        10.0,
        18.0,
        font_size as f32,
        WHITE,
    );

    // Score (top-center)
    let score_txt = format!("SCORE: {}", score);
    let tm = measure_text(&score_txt, None, font_size as u16, 1.0);
    draw_text(
        &score_txt,
        center_x() - tm.w / 2.0,
        18.0,
        font_size as f32,
        YELLOW,
    );

    // Wave number (top-right)
    let wave_txt = format!("WAVE: {}", wave);
    let tm_w = measure_text(&wave_txt, None, font_size as u16, 1.0);
    draw_text(
        &wave_txt,
        screen::screen_w() - tm_w.w - 10.0,
        18.0,
        font_size as f32,
        WHITE,
    );

    // Lives display (below grade, top-left)
    for i in 0..lives {
        assets::draw_life_icon(10.0 + (i as f32) * 20.0, 28.0);
    }

    // Active question text at bottom of screen (above player ship zone)
    if let Some(qtext) = question_text {
        draw_question_banner(qtext);
    }

    set_default_color();
}

/// Height of the portrait question card for the given text.
pub fn mobile_question_card_height(text: &str) -> f32 {
    let lines: Vec<&str> = text.lines().collect();
    let font_size = screen::mobile_text_size(if lines.len() > 2 { 30 } else { 46 });
    let line_h = font_size as f32 + 18.0;
    (lines.len() as f32 * line_h + 56.0).max(260.0)
}

/// Yellow-bordered question card for portrait mobile HUDs. Returns the card bottom Y.
pub fn draw_mobile_question_card(text: &str, banner_y: f32) -> f32 {
    let lines: Vec<&str> = text.lines().collect();
    let banner_w = mobile_safe_w();
    let banner_x = mobile_safe_x();
    let font_size = screen::mobile_text_size(if lines.len() > 2 { 30 } else { 46 });
    let line_h = font_size as f32 + 18.0;
    let banner_h = mobile_question_card_height(text);
    let text_block_h = lines.len() as f32 * line_h;
    let first_baseline = banner_y + (banner_h - text_block_h) / 2.0 + font_size as f32 * 0.88;

    draw_rounded_panel(
        banner_x,
        banner_y,
        banner_w,
        banner_h,
        22.0,
        Color::new(0.055, 0.07, 0.17, 0.94),
        Color::new(1.0, 0.78, 0.28, 0.82),
    );

    for (i, line) in lines.iter().enumerate() {
        let line_font = fit_mobile_font_size(line, font_size, banner_w - 48.0);
        let tm = measure_text(line, None, line_font, 1.0);
        draw_text(
            line,
            center_x() - tm.w / 2.0,
            first_baseline + (i as f32) * line_h,
            line_font as f32,
            Color::new(1.0, 0.97, 0.34, 1.0),
        );
    }

    banner_y + banner_h
}

fn draw_mobile_corner_stat(left: &str, right: &str, y: f32) {
    let font_size = screen::mobile_text_size(20);
    draw_text(
        left,
        82.0,
        y,
        font_size as f32,
        Color::new(0.62, 0.88, 1.0, 1.0),
    );
    let tm = measure_text(right, None, font_size, 1.0);
    draw_text(
        right,
        screen::screen_w() - tm.w - 82.0,
        y,
        font_size as f32,
        Color::new(0.74, 1.0, 0.72, 1.0),
    );
}

fn draw_mobile_hud(grade: &Grade, score: u32, lives: u8, wave: usize, question_text: Option<&str>) {
    let chrome_clearance = 28.0;
    if let Some(qtext) = question_text {
        draw_mobile_question_card(qtext, chrome_clearance);
        let stat_font = screen::mobile_text_size(24);
        let pill_h = stat_font as f32 + 22.0;
        let pill_y = screen::screen_h() - pill_h - 24.0;
        let gap = 12.0;
        let pill_w = (mobile_safe_w() - gap * 2.0) / 3.0;
        let pill_text = Color::new(0.74, 0.9, 1.0, 1.0);

        draw_mobile_hud_pill(
            mobile_safe_x(),
            pill_y,
            pill_w,
            &format!("Wave {}", wave),
            pill_h,
            pill_text,
        );
        draw_mobile_hud_pill(
            mobile_safe_x() + pill_w + gap,
            pill_y,
            pill_w,
            &format!("Score {}", score),
            pill_h,
            pill_text,
        );
        draw_mobile_hud_pill(
            mobile_safe_x() + (pill_w + gap) * 2.0,
            pill_y,
            pill_w,
            &format!("Lives {}", lives),
            pill_h,
            pill_text,
        );
    } else {
        let score_font = screen::mobile_text_size(18);
        let score_txt = format!("SCORE {}", score);
        let tm = measure_text(&score_txt, None, score_font, 1.0);
        draw_text(
            &score_txt,
            center_x() - tm.w / 2.0,
            chrome_clearance + 8.0,
            score_font as f32,
            Color::new(1.0, 0.82, 0.32, 1.0),
        );
        draw_mobile_corner_stat(
            &format!("{}  Wave {}", grade.display_name(), wave),
            &format!("Lives {}", lives),
            chrome_clearance + score_font as f32 + 16.0,
        );
    }

    set_default_color();
}

fn draw_mobile_hud_pill(x: f32, y: f32, w: f32, text: &str, h: f32, text_color: Color) {
    let radius = h / 2.0;
    draw_rounded_panel(
        x,
        y,
        w,
        h,
        radius,
        Color::new(0.055, 0.075, 0.15, 0.9),
        Color::new(0.35, 0.55, 0.88, 0.58),
    );
    let font_size = fit_mobile_font_size(text, screen::mobile_text_size(24), w - 24.0);
    centered_text_in(text, x, y + h * 0.68, w, font_size, text_color);
}

/// Draws a semi-transparent banner with the current math question.
fn draw_question_banner(text: &str) {
    let lines: Vec<&str> = text.lines().collect();
    let banner_w = 920.0;
    let banner_x = center_x() - banner_w / 2.0;
    let banner_y = 44.0;
    let base_font_size = if lines.len() > 2 { 22 } else { 28 };
    let font_size = screen::mobile_text_size(base_font_size);
    let line_h = font_size as f32 + 8.0;
    let banner_h = (lines.len() as f32 * line_h + 22.0).max(62.0);

    // Banner background
    set_color(Color::new(0.1, 0.1, 0.25, 0.86));
    draw_rectangle(banner_x, banner_y, banner_w, banner_h);

    // Banner border
    set_color(Color::new(0.3, 0.3, 0.7, 1.0));
    draw_rectangle_lines(banner_x, banner_y, banner_w, banner_h);

    // Question text (centered in banner)
    set_color(YELLOW);
    for (i, line) in lines.iter().enumerate() {
        let tm = measure_text(line, None, font_size as u16, 1.0);
        draw_text(
            line,
            center_x() - tm.w / 2.0,
            banner_y + 28.0 + (i as f32) * line_h,
            font_size as f32,
            YELLOW,
        );
    }

    set_default_color();
}

/// Draws the space-travel title screen and adventure menu.
pub fn draw_title_screen(showing_mini_games: bool, selected_index: usize) {
    if screen::portrait_layout() {
        draw_mobile_title_screen(showing_mini_games, selected_index);
        return;
    }

    let ink = Color::new(0.08, 0.1, 0.11, 1.0);
    let stone_dark = Color::new(0.2, 0.24, 0.24, 1.0);
    let stone = Color::new(0.46, 0.51, 0.48, 1.0);
    let stone_light = Color::new(0.72, 0.77, 0.68, 1.0);
    let parchment = Color::new(0.82, 0.86, 0.72, 1.0);
    let torch = Color::new(0.95, 0.75, 0.28, 1.0);

    let title_size = screen::mobile_text_size(48);
    let subtitle_size = screen::mobile_text_size(26);
    let hint_size = screen::mobile_text_size(18);
    let shortcut_size = screen::mobile_text_size(16);

    clear_background(Color::new(0.12, 0.15, 0.14, 1.0));
    draw_dungeon_tiles(stone_dark, ink);
    draw_stone_frame(58.0, 38.0, 1164.0, 644.0, stone, stone_light, ink);

    centered_text("STAR CRUSHER", 104.0, title_size, stone_light);
    centered_text("PLANET DUNGEON CREW", 140.0, subtitle_size, parchment);
    centered_text(
        "Choose a path, then press ENTER or SPACE.",
        600.0,
        hint_size,
        stone_light,
    );
    let shortcuts = if showing_mini_games {
        "ESC Back   P Pong   R Snake   N Nightmare"
    } else {
        "Shortcuts: M Math Invaders   P Missions   L Word Cargo"
    };
    centered_text(shortcuts, 628.0, shortcut_size, parchment);

    draw_title_scene(
        120.0,
        184.0,
        470.0,
        350.0,
        ink,
        stone,
        stone_light,
        parchment,
        torch,
    );
    draw_adventure_menu(
        TITLE_MENU_X,
        TITLE_MENU_Y,
        TITLE_MENU_W,
        showing_mini_games,
        selected_index,
        ink,
        stone,
        stone_light,
        parchment,
    );

    set_default_color();
}

fn draw_mobile_title_screen(showing_mini_games: bool, selected_index: usize) {
    let void = Color::new(0.01, 0.015, 0.04, 1.0);
    let panel = Color::new(0.035, 0.07, 0.13, 0.96);
    let panel_edge = Color::new(0.42, 0.78, 1.0, 0.9);
    let moon = Color::new(0.9, 0.96, 0.98, 1.0);
    let amber = Color::new(1.0, 0.76, 0.25, 1.0);
    let cyan = Color::new(0.32, 0.9, 1.0, 1.0);
    let rose = Color::new(1.0, 0.4, 0.68, 1.0);

    clear_background(void);
    draw_star_map_background();

    // Top-left small brand label, matching the HTML topbar. Hidden on the
    // mini-games sub-page because the BACK button sits in the same corner.
    if !showing_mini_games {
        draw_text(
            "STAR CRUSHER ARCADE LEARNING MISSIONS",
            24.0,
            46.0,
            screen::mobile_text_size(20) as f32,
            cyan,
        );
    }

    // Ship/deck visualization sits above the title.
    draw_mobile_space_scene(
        24.0, 80.0, 672.0, 220.0, panel, panel_edge, moon, amber, cyan, rose,
    );

    // Big title below the ship art.
    centered_text("STAR CRUSHER", 380.0, screen::mobile_text_size(64), moon);
    centered_text(
        if showing_mini_games {
            "SELECT MISSION"
        } else {
            "Choose a mission path."
        },
        430.0,
        screen::mobile_text_size(24),
        cyan,
    );

    draw_mobile_adventure_menu(
        showing_mini_games,
        selected_index,
        panel,
        panel_edge,
        moon,
        amber,
    );

    centered_text(
        if showing_mini_games {
            "Tap a mission"
        } else {
            "Ready when you are."
        },
        1208.0,
        screen::mobile_text_size(22),
        moon,
    );

    set_default_color();
}

fn draw_star_map_background() {
    set_color(Color::new(0.04, 0.08, 0.13, 1.0));
    for row in 0..11 {
        for col in 0..20 {
            let x = col as f32 * 68.0 - if row % 2 == 0 { 0.0 } else { 34.0 };
            let y = row as f32 * 68.0 + 8.0;
            draw_rectangle_lines(x, y, 48.0, 48.0);
        }
    }

    for i in 0..72 {
        let x = ((i * 83 + 29) % screen::screen_w() as i32) as f32;
        let y = ((i * 47 + 61) % screen::screen_h() as i32) as f32;
        let size = if i % 5 == 0 { 4.0 } else { 2.0 };
        set_color(if i % 7 == 0 {
            Color::new(0.55, 0.9, 1.0, 0.86)
        } else {
            Color::new(0.86, 0.9, 0.78, 0.76)
        });
        draw_rectangle(x, y, size, size);
    }
}

fn draw_mobile_space_scene(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    panel: Color,
    panel_edge: Color,
    moon: Color,
    amber: Color,
    cyan: Color,
    rose: Color,
) {
    draw_rounded_panel(x, y, w, h, 18.0, panel, panel_edge);

    draw_dungeon_planet(x + w * 0.14, y + h * 0.55, 42.0, moon, amber);
    draw_dungeon_planet(x + w * 0.78, y + h * 0.48, 36.0, rose, cyan);
    draw_spaceship(x + w * 0.46, y + h * 0.40, cyan, moon, amber);
    draw_space_traveler(x + w * 0.32, y + h * 0.46, moon, cyan);
    draw_space_traveler(x + w * 0.62, y + h * 0.48, moon, rose);

    set_color(Color::new(0.35, 0.72, 0.95, 0.42));
    draw_rectangle(x + w * 0.18, y + h * 0.68, w * 0.24, 4.0);
    draw_rectangle(x + w * 0.52, y + h * 0.72, w * 0.22, 4.0);
}

fn draw_dungeon_planet(x: f32, y: f32, radius: f32, body: Color, accent: Color) {
    set_color(body);
    draw_circle(x, y, radius);
    set_color(Color::new(body.r * 0.55, body.g * 0.55, body.b * 0.55, 1.0));
    draw_circle(x - radius * 0.28, y + radius * 0.18, radius * 0.16);
    draw_circle(x + radius * 0.26, y - radius * 0.22, radius * 0.12);

    set_color(Color::new(0.05, 0.08, 0.1, 1.0));
    draw_rectangle(
        x - radius * 0.34,
        y + radius * 0.05,
        radius * 0.68,
        radius * 0.48,
    );
    set_color(accent);
    draw_rectangle(
        x - radius * 0.16,
        y - radius * 0.22,
        radius * 0.32,
        radius * 0.24,
    );
    draw_rectangle_lines(
        x - radius * 0.34,
        y + radius * 0.05,
        radius * 0.68,
        radius * 0.48,
    );
}

fn draw_spaceship(x: f32, y: f32, cyan: Color, moon: Color, amber: Color) {
    set_color(moon);
    draw_rectangle(x - 62.0, y - 18.0, 124.0, 36.0);
    draw_rectangle(x - 32.0, y - 42.0, 64.0, 24.0);
    set_color(cyan);
    draw_rectangle(x - 20.0, y - 34.0, 40.0, 12.0);
    draw_rectangle(x - 86.0, y - 10.0, 24.0, 20.0);
    draw_rectangle(x + 62.0, y - 10.0, 24.0, 20.0);
    set_color(amber);
    draw_rectangle(x - 18.0, y + 18.0, 12.0, 18.0);
    draw_rectangle(x + 6.0, y + 18.0, 12.0, 18.0);
}

fn draw_space_traveler(x: f32, y: f32, suit: Color, accent: Color) {
    set_color(suit);
    draw_rectangle(x - 16.0, y - 34.0, 32.0, 28.0);
    draw_rectangle(x - 22.0, y - 4.0, 44.0, 54.0);
    draw_rectangle(x - 34.0, y + 2.0, 12.0, 42.0);
    draw_rectangle(x + 22.0, y + 2.0, 12.0, 42.0);
    draw_rectangle(x - 16.0, y + 50.0, 12.0, 38.0);
    draw_rectangle(x + 4.0, y + 50.0, 12.0, 38.0);
    set_color(accent);
    draw_rectangle(x - 10.0, y - 26.0, 20.0, 10.0);
    draw_rectangle(x - 14.0, y + 14.0, 28.0, 8.0);
}

fn draw_mobile_adventure_menu(
    showing_mini_games: bool,
    selected_index: usize,
    _panel: Color,
    _panel_edge: Color,
    _moon: Color,
    _amber: Color,
) {
    let main_options = ["Launch Game", "Select Mission", "Word Cargo"];
    let mini_game_options = ["Reading Planet", "Math Orbit", "Night Planet"];
    let options: &[&str] = if showing_mini_games {
        &mini_game_options
    } else {
        &main_options
    };

    for (index, label) in options.iter().enumerate() {
        let y = MOBILE_TITLE_MENU_ROW_TOP
            + index as f32 * (MOBILE_TITLE_MENU_ROW_H + MOBILE_TITLE_MENU_ROW_GAP);
        let rect = Rect::new(
            MOBILE_TITLE_MENU_X,
            y,
            MOBILE_TITLE_MENU_W,
            MOBILE_TITLE_MENU_ROW_H,
        );
        let selected = selected_index % options.len() == index;
        if !mobile_html_overlay_controls() {
            draw_mobile_yellow_button_rect_selectable(rect, label, 44, selected);
        }
    }
}

fn draw_mobile_yellow_button_rect_selectable(
    rect: Rect,
    label: &str,
    desired_size: u16,
    selected: bool,
) {
    draw_mobile_yellow_button_rect(rect, label, desired_size);
    if selected {
        set_color(Color::new(1.0, 1.0, 1.0, 0.72));
        draw_rectangle_lines(rect.x - 4.0, rect.y - 4.0, rect.w + 8.0, rect.h + 8.0);
        set_default_color();
    }
}

fn draw_dungeon_tiles(stone_dark: Color, ink: Color) {
    for row in 0..15 {
        for col in 0..28 {
            let x = col as f32 * 48.0 + if row % 2 == 0 { 0.0 } else { -24.0 };
            let y = row as f32 * 48.0;
            let shade = if (row + col) % 2 == 0 {
                stone_dark
            } else {
                ink
            };
            set_color(Color::new(shade.r, shade.g, shade.b, 0.28));
            draw_rectangle(x, y, 46.0, 46.0);
        }
    }
}

fn draw_stone_frame(x: f32, y: f32, w: f32, h: f32, stone: Color, stone_light: Color, ink: Color) {
    set_color(ink);
    draw_rectangle(x - 10.0, y - 10.0, w + 20.0, h + 20.0);
    set_color(stone);
    draw_rectangle(x, y, w, h);
    set_color(stone_light);
    draw_rectangle_lines(x + 10.0, y + 10.0, w - 20.0, h - 20.0);
    set_color(ink);
    draw_rectangle(x + 28.0, y + 28.0, w - 56.0, h - 56.0);

    set_color(Color::new(
        stone_light.r,
        stone_light.g,
        stone_light.b,
        0.35,
    ));
    let horizontal_blocks = ((w - 44.0) / 50.0).max(1.0) as usize;
    for i in 0..horizontal_blocks {
        let bx = x + 22.0 + i as f32 * 50.0;
        draw_rectangle_lines(bx, y + 8.0, 42.0, 24.0);
        draw_rectangle_lines(bx, y + h - 32.0, 42.0, 24.0);
    }
    let vertical_blocks = ((h - 84.0) / 50.0).max(1.0) as usize;
    for i in 0..vertical_blocks {
        let by = y + 42.0 + i as f32 * 50.0;
        draw_rectangle_lines(x + 8.0, by, 24.0, 42.0);
        draw_rectangle_lines(x + w - 32.0, by, 24.0, 42.0);
    }
}

fn draw_title_scene(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    ink: Color,
    stone: Color,
    stone_light: Color,
    parchment: Color,
    torch: Color,
) {
    set_color(Color::new(0.04, 0.08, 0.13, 1.0));
    draw_rectangle(x, y, w, h);
    set_color(stone_light);
    draw_rectangle_lines(x, y, w, h);

    for i in 0..32 {
        let sx = x + ((i * 61 + 17) % w as i32) as f32;
        let sy = y + ((i * 37 + 23) % h as i32) as f32;
        set_color(if i % 5 == 0 { torch } else { parchment });
        draw_rectangle(sx, sy, 3.0, 3.0);
    }

    draw_dungeon_planet(x + 84.0, y + 92.0, 42.0, stone_light, torch);
    draw_dungeon_planet(x + 292.0, y + 88.0, 34.0, stone, parchment);
    draw_spaceship(x + 190.0, y + 104.0, stone_light, parchment, torch);
    draw_space_traveler(x + 126.0, y + 256.0, parchment, stone_light);
    draw_space_traveler(x + 238.0, y + 264.0, parchment, torch);

    set_color(Color::new(
        stone_light.r,
        stone_light.g,
        stone_light.b,
        0.45,
    ));
    draw_rectangle(x + 98.0, y + 156.0, 184.0, 4.0);
    set_color(ink);
    draw_rectangle(x + 172.0, y + 278.0, 44.0, 28.0);
    set_color(stone_light);
    draw_rectangle_lines(x + 172.0, y + 278.0, 44.0, 28.0);
}

fn draw_adventure_menu(
    x: f32,
    y: f32,
    w: f32,
    showing_mini_games: bool,
    selected_index: usize,
    _ink: Color,
    _stone: Color,
    stone_light: Color,
    parchment: Color,
) {
    let gold_fill = Color::new(1.0, 0.84, 0.44, 0.99);
    let gold_edge = Color::new(0.85, 0.62, 0.18, 1.0);
    let gold_shadow = Color::new(0.83, 0.56, 0.18, 0.28);
    let ink_dark = Color::new(0.035, 0.06, 0.1, 1.0);

    let main_options = [
        ("Launch Voyage", "Start the planet dungeon route"),
        ("Mission Select", "Practice arcade encounters"),
        ("Word Cargo", "Load a spelling list"),
    ];
    let mini_game_options = [
        ("Reading Planet", "Steer through word caves"),
        ("Math Orbit", "Bounce into number targets"),
        ("Night Planet", "Same-color letter trial"),
    ];
    let options = if showing_mini_games {
        &mini_game_options
    } else {
        &main_options
    };
    let title = if showing_mini_games {
        "PLAY MINI GAMES"
    } else {
        "ADVENTURE MENU"
    };

    set_color(Color::new(0.16, 0.19, 0.18, 1.0));
    draw_rectangle(x, y, w, 382.0);
    set_color(stone_light);
    draw_rectangle_lines(x, y, w, 382.0);
    centered_text_in(title, x, y + 42.0, w, 24, parchment);

    for (index, (label, detail)) in options.iter().enumerate() {
        let row_top = TITLE_MENU_ROW_TOP + index as f32 * (TITLE_MENU_ROW_H + TITLE_MENU_ROW_GAP);
        let selected = selected_index % options.len() == index;
        draw_adventure_menu_row(
            x + 22.0,
            row_top,
            w - 44.0,
            TITLE_MENU_ROW_H,
            label,
            detail,
            selected,
            ink_dark,
            gold_fill,
            gold_edge,
            gold_shadow,
        );
    }

    if showing_mini_games {
        centered_text_in("Press ESC to return", x, y + 338.0, w, 14, parchment);
    }
}

fn draw_adventure_menu_row(
    rx: f32,
    ry: f32,
    rw: f32,
    rh: f32,
    label: &str,
    detail: &str,
    selected: bool,
    ink_dark: Color,
    gold_fill: Color,
    gold_edge: Color,
    gold_shadow: Color,
) {
    let radius = 18.0;

    // Shadow bar at bottom
    set_color(gold_shadow);
    draw_rounded_rect(rx + 4.0, ry + rh - 12.0, rw - 8.0, 8.0, 4.0, gold_shadow);

    // Outer border (gold edge)
    draw_rounded_rect(rx, ry, rw, rh, radius, gold_edge);
    // Inner fill
    draw_rounded_rect(
        rx + 3.0,
        ry + 3.0,
        rw - 6.0,
        rh - 6.0,
        radius - 3.0,
        gold_fill,
    );

    if selected {
        set_color(Color::new(1.0, 1.0, 1.0, 0.72));
        draw_rectangle_lines(rx - 4.0, ry - 4.0, rw + 8.0, rh + 8.0);
        set_default_color();
    }

    let label_size = screen::mobile_text_size(20);
    let detail_size = screen::mobile_text_size(13);
    draw_text(label, rx + 54.0, ry + 24.0, label_size as f32, ink_dark);
    draw_text(detail, rx + 54.0, ry + 46.0, detail_size as f32, ink_dark);
}

fn draw_door_glyph(x: f32, y: f32, ink: Color, stone: Color, stone_light: Color) {
    set_color(stone);
    draw_rectangle(x, y + 40.0, 96.0, 104.0);
    set_color(ink);
    draw_rectangle(x + 18.0, y + 58.0, 60.0, 86.0);
    set_color(stone_light);
    draw_rectangle_lines(x + 18.0, y + 58.0, 60.0, 86.0);
    draw_rectangle(x + 44.0, y + 20.0, 8.0, 34.0);
    draw_rectangle(x + 30.0, y + 28.0, 36.0, 8.0);
    draw_rectangle(x + 62.0, y + 100.0, 6.0, 6.0);
}

fn draw_torch_glyph(x: f32, y: f32, ink: Color, torch: Color) {
    set_color(torch);
    draw_rectangle(x + 10.0, y, 12.0, 22.0);
    draw_rectangle(x + 4.0, y + 8.0, 24.0, 10.0);
    set_color(ink);
    draw_rectangle(x + 12.0, y + 28.0, 8.0, 56.0);
    draw_rectangle(x + 4.0, y + 44.0, 24.0, 8.0);
}

fn draw_hero_glyph(x: f32, y: f32, ink: Color, parchment: Color) {
    set_color(parchment);
    draw_rectangle(x + 22.0, y, 24.0, 22.0);
    draw_rectangle(x + 14.0, y + 24.0, 40.0, 46.0);
    draw_rectangle(x + 4.0, y + 34.0, 12.0, 28.0);
    draw_rectangle(x + 52.0, y + 34.0, 12.0, 28.0);
    draw_rectangle(x + 16.0, y + 70.0, 14.0, 30.0);
    draw_rectangle(x + 38.0, y + 70.0, 14.0, 30.0);
    set_color(ink);
    draw_rectangle(x + 28.0, y + 8.0, 4.0, 4.0);
    draw_rectangle(x + 38.0, y + 8.0, 4.0, 4.0);
}

fn draw_monster_glyph(x: f32, y: f32, ink: Color, stone_light: Color) {
    set_color(stone_light);
    draw_rectangle(x + 8.0, y + 18.0, 58.0, 48.0);
    draw_rectangle(x, y + 32.0, 10.0, 20.0);
    draw_rectangle(x + 66.0, y + 32.0, 10.0, 20.0);
    draw_rectangle(x + 14.0, y + 66.0, 12.0, 18.0);
    draw_rectangle(x + 50.0, y + 66.0, 12.0, 18.0);
    set_color(ink);
    draw_rectangle(x + 24.0, y + 32.0, 8.0, 8.0);
    draw_rectangle(x + 44.0, y + 32.0, 8.0, 8.0);
    draw_rectangle(x + 28.0, y + 52.0, 22.0, 6.0);
}

fn centered_text_in(text: &str, x: f32, y: f32, w: f32, font_size: u16, color: Color) {
    let size = fit_mobile_font_size(text, font_size, (w - 24.0).max(1.0));
    let tm = measure_text(text, None, size, 1.0);
    draw_text(text, x + w / 2.0 - tm.w / 2.0, y, size as f32, color);
}

const ADVENTURE_INTRO_PAGES: [(&str, &str); 5] = [
    ("PLANET DUNGEON ROUTE", "A Star Crusher Voyage"),
    (
        "Two travelers chart dungeon planets from their little ship.",
        "Every world hides doors powered by number magic.",
    ),
    (
        "Ancient gates open for crews who solve the number riddles.",
        "Every correct answer lights another landing path.",
    ),
    (
        "Your crew is brave, clever, and ready for launch.",
        "Step from the ship into the first alien dungeon.",
    ),
    (
        "First mission: clear the drifting number sentries.",
        "Math Invaders guards the route between planets.",
    ),
];

pub fn adventure_intro_page_count() -> usize {
    ADVENTURE_INTRO_PAGES.len()
}

/// Draws the lightweight voyage intro before Start Adventure enters Math Invaders.
pub fn draw_adventure_intro(page: usize) {
    if screen::portrait_layout() {
        draw_mobile_adventure_intro(page);
        return;
    }

    let ink = Color::new(0.07, 0.08, 0.1, 1.0);
    let stone_dark = Color::new(0.16, 0.18, 0.2, 1.0);
    let stone = Color::new(0.38, 0.42, 0.43, 1.0);
    let stone_light = Color::new(0.72, 0.76, 0.7, 1.0);
    let parchment = Color::new(0.88, 0.83, 0.62, 1.0);
    let torch = Color::new(1.0, 0.68, 0.22, 1.0);
    let page = page.min(ADVENTURE_INTRO_PAGES.len() - 1);
    let (line_one, line_two) = ADVENTURE_INTRO_PAGES[page];

    clear_background(Color::new(0.08, 0.09, 0.11, 1.0));
    draw_dungeon_tiles(stone_dark, ink);
    draw_stone_frame(92.0, 44.0, 1096.0, 616.0, stone, stone_light, ink);

    draw_door_glyph(592.0, 150.0, ink, stone, stone_light);
    draw_torch_glyph(430.0, 204.0, ink, torch);
    draw_torch_glyph(820.0, 204.0, ink, torch);
    draw_hero_glyph(438.0, 384.0, ink, parchment);
    draw_monster_glyph(784.0, 376.0, ink, stone_light);

    set_color(stone);
    for step in 0..6 {
        draw_rectangle(
            552.0 + step as f32 * 16.0,
            314.0 + step as f32 * 16.0,
            176.0 - step as f32 * 32.0,
            10.0,
        );
    }

    let title_size = screen::mobile_text_size(26);
    let page_size = screen::mobile_text_size(16);
    let line1_size = screen::mobile_text_size(if page == 0 { 34 } else { 24 });
    let line2_size = screen::mobile_text_size(if page == 0 { 22 } else { 20 });
    let hint_size = screen::mobile_text_size(18);
    let mobile = screen::portrait_layout();

    centered_text("START ADVENTURE", 96.0, title_size, parchment);
    centered_text_in(
        &format!("PAGE {} / {}", page + 1, ADVENTURE_INTRO_PAGES.len()),
        980.0,
        98.0,
        170.0,
        page_size,
        stone_light,
    );

    set_color(Color::new(0.1, 0.08, 0.12, 0.96));
    draw_rectangle(180.0, 500.0, 920.0, 110.0);
    set_color(parchment);
    draw_rectangle_lines(180.0, 500.0, 920.0, 110.0);

    let _title_card = page == 0;
    centered_text_in(line_one, 210.0, 542.0, 860.0, line1_size, parchment);
    centered_text_in(line_two, 210.0, 578.0, 860.0, line2_size, WHITE);
    if mobile {
        draw_mobile_action_button(if page + 1 >= ADVENTURE_INTRO_PAGES.len() {
            "START"
        } else {
            "CONTINUE"
        });
    } else {
        centered_text(
            "ENTER / SPACE: continue     ESC: return to title",
            686.0,
            hint_size,
            stone_light,
        );
    }

    set_default_color();
}

fn draw_mobile_adventure_intro(page: usize) {
    let page = page.min(ADVENTURE_INTRO_PAGES.len() - 1);
    let (line_one, line_two) = ADVENTURE_INTRO_PAGES[page];
    let panel = Color::new(0.035, 0.07, 0.13, 0.96);
    let edge = Color::new(0.42, 0.78, 1.0, 0.82);
    let moon = Color::new(0.9, 0.96, 0.98, 1.0);
    let amber = Color::new(1.0, 0.76, 0.25, 1.0);
    let cyan = Color::new(0.32, 0.9, 1.0, 1.0);
    let rose = Color::new(1.0, 0.4, 0.68, 1.0);

    clear_background(Color::new(0.01, 0.015, 0.04, 1.0));
    draw_star_map_background();

    centered_text_in(
        "STAR CRUSHER VOYAGE",
        96.0,
        92.0,
        528.0,
        screen::mobile_text_size(32),
        moon,
    );
    centered_text(
        &format!("PAGE {} / {}", page + 1, ADVENTURE_INTRO_PAGES.len()),
        122.0,
        screen::mobile_text_size(18),
        Color::new(0.7, 0.9, 1.0, 1.0),
    );

    draw_rounded_panel(24.0, 140.0, 672.0, 340.0, 22.0, panel, edge);
    draw_dungeon_planet(180.0, 310.0, 52.0, Color::new(0.82, 0.9, 0.7, 1.0), amber);
    draw_dungeon_planet(540.0, 288.0, 44.0, rose, cyan);
    draw_spaceship(360.0, 286.0, cyan, moon, amber);
    draw_space_traveler(280.0, 348.0, moon, cyan);
    draw_space_traveler(440.0, 354.0, moon, rose);
    set_color(Color::new(0.36, 0.74, 1.0, 0.38));
    draw_rectangle(240.0, 306.0, 120.0, 5.0);
    draw_rectangle(400.0, 306.0, 120.0, 5.0);

    draw_rounded_panel(
        24.0,
        548.0,
        672.0,
        252.0,
        22.0,
        Color::new(0.07, 0.055, 0.12, 0.96),
        Color::new(1.0, 0.82, 0.36, 0.9),
    );
    let line1_size = screen::mobile_text_size(if page == 0 { 32 } else { 24 });
    let line2_size = screen::mobile_text_size(if page == 0 { 24 } else { 20 });
    draw_wrapped_centered_text(line_one, 644.0, 600.0, line1_size, amber);
    draw_wrapped_centered_text(line_two, 704.0, 600.0, line2_size, moon);

    draw_mobile_action_button(if page + 1 >= ADVENTURE_INTRO_PAGES.len() {
        "START"
    } else {
        "CONTINUE"
    });

    set_default_color();
}

/// Draws the weekly spelling-list entry screen for Reading Snake.
pub fn draw_spelling_list_screen(input: &str) {
    clear_background(Color::new(0.01, 0.015, 0.04, 1.0));
    draw_star_map_background();

    let panel = Color::new(0.035, 0.07, 0.13, 0.96);
    let panel_edge = Color::new(0.42, 0.78, 1.0, 0.82);
    let moon = Color::new(0.9, 0.96, 0.98, 1.0);
    let amber = Color::new(1.0, 0.76, 0.25, 1.0);
    let cyan = Color::new(0.32, 0.9, 1.0, 1.0);

    centered_text("WORD CARGO", 100.0, screen::mobile_text_size(36), moon);
    centered_text(
        "Type word: definition pairs separated by semicolons.",
        148.0,
        screen::mobile_text_size(22),
        cyan,
    );
    centered_text(
        "Plain word lists still work too.",
        176.0,
        screen::mobile_text_size(20),
        Color::new(0.62, 0.86, 1.0, 1.0),
    );
    centered_text(
        "ENTER plays Reading Snake   N starts Nightmare",
        218.0,
        screen::mobile_text_size(22),
        moon,
    );

    let input_w = if screen::portrait_layout() {
        mobile_safe_w()
    } else {
        760.0
    };
    let input_h = 118.0;
    let input_x = center_x() - input_w / 2.0;
    let input_y = 278.0;

    draw_rounded_panel(input_x, input_y, input_w, input_h, 18.0, panel, panel_edge);

    let shown_input = if input.is_empty() {
        "apple: a fruit; moon: shines at night"
    } else {
        input
    };
    let color = if input.is_empty() {
        Color::new(0.62, 0.72, 0.78, 1.0)
    } else {
        moon
    };
    draw_wrapped_text(
        shown_input,
        input_x + 26.0,
        input_y + 48.0,
        input_w - 52.0,
        22,
        color,
    );

    let blink = if (get_time() as f32 * 3.0).fract() > 0.5 {
        1.0
    } else {
        0.0
    };
    set_color(Color::new(1.0, 0.82, 0.34, blink));
    if input.is_empty() {
        draw_rectangle(input_x + 26.0, input_y + 78.0, 16.0, 3.0);
    }

    centered_text("Leave it blank to use the default words.", 456.0, 18, amber);
    centered_text(
        "Backspace deletes   ESC returns to title",
        504.0,
        18,
        Color::new(0.62, 0.86, 1.0, 1.0),
    );
    draw_spelling_action_button(spelling_play_button_rect(), "PLAY", amber);
    draw_spelling_action_button(
        spelling_nightmare_button_rect(),
        "NIGHT",
        Color::new(1.0, 0.4, 0.68, 1.0),
    );
    set_default_color();
}

fn draw_spelling_action_button(rect: Rect, label: &str, color: Color) {
    if screen::portrait_layout() {
        if mobile_html_overlay_controls() {
            let _ = (rect, label);
            return;
        }
        draw_mobile_yellow_button_rect(rect, label, 40);
        return;
    }

    draw_rounded_panel(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        18.0,
        Color::new(color.r, color.g, color.b, 0.26),
        color,
    );
    centered_text_in(
        label,
        rect.x,
        rect.y + 48.0,
        rect.w,
        screen::mobile_text_size(16),
        if label == "PLAY" {
            Color::new(0.035, 0.06, 0.1, 1.0)
        } else {
            WHITE
        },
    );
}

fn mobile_html_overlay_controls() -> bool {
    cfg!(target_arch = "wasm32") && screen::portrait_layout()
}

fn centered_text(text: &str, y: f32, font_size: u16, color: Color) {
    let size = fit_mobile_font_size(text, font_size, screen::screen_w() - 48.0);
    let tm = measure_text(text, None, size, 1.0);
    draw_text(text, center_x() - tm.w / 2.0, y, size as f32, color);
}

fn draw_wrapped_centered_text(text: &str, y: f32, max_width: f32, font_size: u16, color: Color) {
    let mut line = String::new();
    let mut line_y = y;

    for word in text.split_whitespace() {
        let next = if line.is_empty() {
            word.to_string()
        } else {
            format!("{} {}", line, word)
        };

        if measure_text(&next, None, font_size, 1.0).w > max_width && !line.is_empty() {
            centered_text(&line, line_y, font_size, color);
            line = word.to_string();
            line_y += font_size as f32 + 8.0;
        } else {
            line = next;
        }
    }

    if !line.is_empty() {
        centered_text(&line, line_y, font_size, color);
    }
}

pub fn gate_question_text_size() -> u16 {
    screen::mobile_text_size(24)
}

pub fn keypad_button_rect(index: usize) -> Rect {
    let col = index % 3;
    let row = index / 3;
    let (key, gap, x, y) = if screen::portrait_layout() {
        let key = 68.0;
        let gap = 10.0;
        let total_w = key * 3.0 + gap * 2.0;
        (key, gap, center_x() - total_w / 2.0, 676.0)
    } else {
        (KEYPAD_KEY, KEYPAD_GAP, KEYPAD_X, KEYPAD_Y)
    };

    Rect::new(
        x + col as f32 * (key + gap),
        y + row as f32 * (key + gap),
        key,
        key,
    )
}

fn draw_wrapped_text(text: &str, x: f32, y: f32, max_width: f32, font_size: u16, color: Color) {
    let mut line = String::new();
    let mut line_y = y;

    for word in text.split_whitespace() {
        let next = if line.is_empty() {
            word.to_string()
        } else {
            format!("{} {}", line, word)
        };

        if measure_text(&next, None, font_size, 1.0).w > max_width && !line.is_empty() {
            draw_text(&line, x, line_y, font_size as f32, color);
            line = word.to_string();
            line_y += font_size as f32 + 8.0;
        } else {
            line = next;
        }
    }

    if !line.is_empty() {
        draw_text(&line, x, line_y, font_size as f32, color);
    }
}

/// Draws the question gate screen between waves.
pub fn draw_question_gate(grade: &Grade, math_topics: &str, show_start_button: bool) {
    if screen::portrait_layout() {
        draw_mobile_question_gate(grade, math_topics, show_start_button);
        return;
    }

    let title_size = screen::mobile_text_size(32);
    let grade_size = screen::mobile_text_size(26);
    let topic_size = screen::mobile_text_size(20);
    let instr_size = screen::mobile_text_size(16);

    let box_w = 720.0;
    let box_h = 270.0;
    let box_x = center_x() - box_w / 2.0;
    let box_y = 128.0;

    // Semi-transparent overlay
    set_color(Color::new(0.05, 0.05, 0.15, 0.85));
    draw_rectangle(0.0, 0.0, screen::screen_w(), screen::screen_h());

    // Gate title
    let gate_title = "WAVE COMPLETE!";
    set_color(GREEN);
    let tm_gt = measure_text(gate_title, None, title_size, 1.0);
    draw_text(
        gate_title,
        center_x() - tm_gt.w / 2.0,
        76.0,
        title_size as f32,
        current_color(),
    );

    // Grade info box
    set_color(Color::new(0.15, 0.15, 0.3, 0.9));
    draw_rectangle(box_x, box_y, box_w, box_h);

    // Border around info box
    set_color(Color::new(0.3, 0.6, 1.0, 1.0));
    draw_rectangle_lines(box_x, box_y, box_w, box_h);

    // Grade name
    set_color(WHITE);
    let grade_txt = format!("Next: {}", grade.display_name());
    let tm_gn = measure_text(&grade_txt, None, grade_size, 1.0);
    draw_text(
        &grade_txt,
        center_x() - tm_gn.w / 2.0,
        168.0,
        grade_size as f32,
        current_color(),
    );

    // Math topics for this grade
    set_color(Color::new(0.7, 0.9, 1.0, 1.0));
    let topic_txt = format!("Topics: {}", math_topics);
    let tm_tp = measure_text(&topic_txt, None, topic_size, 1.0);
    draw_text(
        &topic_txt,
        center_x() - tm_tp.w / 2.0,
        218.0,
        topic_size as f32,
        current_color(),
    );

    // Instructions for gate questions
    let instructions = if screen::portrait_layout() {
        if show_start_button {
            [
                "Answer the math question correctly to advance.",
                "",
                "Use the number pad on the next screen.",
                "",
                "Tap START when ready.",
            ]
        } else {
            [
                "Answer the math question correctly to advance.",
                "",
                "Use the number pad.",
                "",
                "Tap OK to submit.",
            ]
        }
    } else {
        [
            "Answer the math question correctly to advance.",
            "",
            "Type your answer and press ENTER.",
            "",
            if show_start_button {
                "Press SPACE or ENTER when ready!"
            } else {
                "Submit the answer to continue."
            },
        ]
    };

    set_color(WHITE);
    for (i, line) in instructions.iter().enumerate() {
        let tm = measure_text(line, None, instr_size, 1.0);
        draw_text(
            line,
            center_x() - tm.w / 2.0,
            286.0 + (i as f32) * 27.0,
            instr_size as f32,
            WHITE,
        );
    }

    if show_start_button {
        draw_mobile_action_button("START");
    }

    set_default_color();
}

fn draw_mobile_question_gate(grade: &Grade, math_topics: &str, show_start_button: bool) {
    set_color(Color::new(0.01, 0.015, 0.04, 0.86));
    draw_rectangle(0.0, 0.0, screen::screen_w(), screen::screen_h());

    draw_rounded_panel(
        mobile_safe_x(),
        98.0,
        mobile_safe_w(),
        276.0,
        24.0,
        Color::new(0.045, 0.065, 0.15, 0.97),
        Color::new(0.42, 0.86, 1.0, 0.84),
    );

    centered_text(
        "Planet Gate",
        158.0,
        screen::mobile_text_size(36),
        Color::new(0.42, 0.92, 1.0, 1.0),
    );
    centered_text(
        &format!("Next: {}", grade.display_name()),
        220.0,
        screen::mobile_text_size(30),
        Color::new(1.0, 0.82, 0.34, 1.0),
    );
    draw_wrapped_centered_text(
        &format!("Topics: {}", math_topics),
        270.0,
        mobile_safe_w() - 48.0,
        screen::mobile_text_size(22),
        Color::new(0.92, 0.98, 1.0, 1.0),
    );

    let (line_one, line_two) = if show_start_button {
        (
            "Answer the gate question to fly onward.",
            "Tap START when your crew is ready.",
        )
    } else {
        ("Use the number pad.", "Tap OK to submit your answer.")
    };
    centered_text(
        line_one,
        330.0,
        screen::mobile_text_size(22),
        Color::new(0.74, 1.0, 0.72, 1.0),
    );
    centered_text(
        line_two,
        354.0,
        screen::mobile_text_size(20),
        Color::new(0.7, 0.82, 0.94, 1.0),
    );

    if show_start_button {
        draw_mobile_action_button("START");
    }

    set_default_color();
}

/// Draws the game over screen with final score and grade reached.
pub fn draw_game_over(score: u32, grade_reached: &Grade) {
    if screen::portrait_layout() {
        draw_mobile_end_overlay(
            "GAME OVER",
            &format!("Final Score: {}", score),
            &format!("Reached {}", grade_reached.display_name()),
            Color::new(1.0, 0.36, 0.38, 1.0),
        );
        return;
    }

    let title_size = screen::mobile_text_size(48);
    let score_size = screen::mobile_text_size(28);
    let grade_size = screen::mobile_text_size(24);
    let restart_size = screen::mobile_text_size(20);

    // Dark overlay
    set_color(Color::new(0.1, 0.05, 0.05, 0.9));
    draw_rectangle(0.0, 0.0, screen::screen_w(), screen::screen_h());

    // Game Over title with pulsing effect
    let pulse = (get_time() as f32 * 3.0).sin() * 0.2 + 0.8;
    set_color(Color::new(1.0, 0.2, 0.2, pulse));

    let go_title = "GAME OVER";
    let tm_go = measure_text(go_title, None, title_size, 1.0);
    draw_text(
        go_title,
        center_x() - tm_go.w / 2.0,
        150.0,
        title_size as f32,
        current_color(),
    );

    // Stats box
    let box_w = 520.0;
    let box_h = 200.0;
    let box_x = center_x() - box_w / 2.0;
    let box_y = 200.0;
    set_color(Color::new(0.15, 0.15, 0.3, 0.9));
    draw_rectangle(box_x, box_y, box_w, box_h);

    // Border
    set_color(Color::new(0.8, 0.3, 0.3, 1.0));
    draw_rectangle_lines(box_x, box_y, box_w, box_h);

    // Final score
    set_color(YELLOW);
    let score_txt = format!("Final Score: {}", score);
    let tm_sc = measure_text(&score_txt, None, score_size, 1.0);
    draw_text(
        &score_txt,
        center_x() - tm_sc.w / 2.0,
        260.0,
        score_size as f32,
        current_color(),
    );

    // Grade reached
    set_color(WHITE);
    let grade_txt = format!("Grade Reached: {}", grade_reached.display_name());
    let tm_gr = measure_text(&grade_txt, None, grade_size, 1.0);
    draw_text(
        &grade_txt,
        center_x() - tm_gr.w / 2.0,
        320.0,
        grade_size as f32,
        current_color(),
    );

    // Restart prompt
    set_color(WHITE);
    if screen::portrait_layout() {
        draw_mobile_action_button("START");
    } else {
        let restart = "Press ENTER to Play Again";
        let tm_rs = measure_text(restart, None, restart_size, 1.0);
        draw_text(
            restart,
            center_x() - tm_rs.w / 2.0,
            450.0,
            restart_size as f32,
            current_color(),
        );
    }

    set_default_color();
}

/// Draws the victory screen (completed all grades through 5th).
pub fn draw_victory_screen(score: u32) {
    if screen::portrait_layout() {
        draw_mobile_end_overlay(
            "VICTORY!",
            &format!("Final Score: {}", score),
            "You mastered every planet gate.",
            Color::new(1.0, 0.82, 0.34, 1.0),
        );
        return;
    }

    let title_size = screen::mobile_text_size(48);
    let score_size = screen::mobile_text_size(36);
    let achievement_size = screen::mobile_text_size(20);
    let restart_size = screen::mobile_text_size(18);

    // Celebration overlay with gradient-like effect
    set_color(Color::new(0.1, 0.1, 0.2, 0.9));
    draw_rectangle(0.0, 0.0, screen::screen_w(), screen::screen_h());

    // Victory title with rainbow cycling color
    let hue = (get_time() as f32 * 0.5) % 1.0;
    set_color(hsl_to_rgb(hue, 0.8, 0.7));

    let vic_title = "★ VICTORY! ★";
    let tm_vt = measure_text(vic_title, None, title_size, 1.0);
    draw_text(
        vic_title,
        center_x() - tm_vt.w / 2.0,
        150.0,
        title_size as f32,
        current_color(),
    );

    // Stats box
    let box_w = 620.0;
    let box_h = 220.0;
    let box_x = center_x() - box_w / 2.0;
    let box_y = 200.0;
    set_color(Color::new(0.15, 0.2, 0.3, 0.9));
    draw_rectangle(box_x, box_y, box_w, box_h);

    // Border with gold color
    set_color(YELLOW);
    draw_rectangle_lines(box_x, box_y, box_w, box_h);

    // Final score (large)
    let score_txt = format!("Final Score: {}", score);
    let tm_sc = measure_text(&score_txt, None, score_size, 1.0);
    set_color(YELLOW);
    draw_text(
        &score_txt,
        center_x() - tm_sc.w / 2.0,
        280.0,
        score_size as f32,
        current_color(),
    );

    // Achievement text
    let achievement = "You've mastered math from Preschool through 5th Grade!";
    set_color(WHITE);
    for (i, line) in achievement.lines().enumerate() {
        let tm = measure_text(line, None, achievement_size, 1.0);
        draw_text(
            line,
            center_x() - tm.w / 2.0,
            340.0 + (i as f32) * 25.0,
            achievement_size as f32,
            WHITE,
        );
    }

    // Restart prompt
    if screen::portrait_layout() {
        draw_mobile_action_button("START");
    } else {
        let restart = "Press ENTER to Play Again";
        set_color(WHITE);
        let tm_rs = measure_text(restart, None, restart_size, 1.0);
        draw_text(
            restart,
            center_x() - tm_rs.w / 2.0,
            500.0,
            restart_size as f32,
            current_color(),
        );
    }

    set_default_color();
}

fn draw_mobile_end_overlay(title: &str, score: &str, detail: &str, accent: Color) {
    set_color(Color::new(0.01, 0.012, 0.035, 0.9));
    draw_rectangle(0.0, 0.0, screen::screen_w(), screen::screen_h());
    draw_rounded_panel(
        32.0,
        176.0,
        screen::screen_w() - 64.0,
        292.0,
        26.0,
        Color::new(0.045, 0.06, 0.14, 0.98),
        accent,
    );
    centered_text(title, 252.0, screen::mobile_text_size(44), accent);
    centered_text(
        score,
        332.0,
        screen::mobile_text_size(30),
        Color::new(1.0, 0.82, 0.34, 1.0),
    );
    centered_text(
        detail,
        382.0,
        screen::mobile_text_size(22),
        Color::new(0.93, 0.98, 1.0, 1.0),
    );
    draw_mobile_action_button("START");
    set_default_color();
}

/// Draws the current answer input display during question gates.
pub fn draw_answer_input(current_input: &str) {
    let input_font = screen::mobile_text_size(24);
    let input_w = 240.0;
    let input_h = 50.0;
    let input_x = center_x() - input_w / 2.0;
    let input_y = 540.0;

    // Input box background
    set_color(Color::new(0.2, 0.2, 0.4, 0.9));
    draw_rectangle(input_x, input_y, input_w, input_h);

    // Border (cyan for active input)
    set_color(Color::new(0.3, 1.0, 1.0, 1.0));
    draw_rectangle_lines(input_x, input_y, input_w, input_h);

    set_color(WHITE);
    let tm = measure_text(current_input, None, input_font, 1.0);
    draw_text(
        current_input,
        center_x() - tm.w / 2.0,
        input_y + 32.0,
        input_font as f32,
        WHITE,
    );

    // Blinking cursor effect
    let blink = if (get_time() as f32 * 3.0).fract() > 0.5 {
        1.0
    } else {
        0.0
    };
    set_color(Color::new(0.3, 1.0, 1.0, blink));
    let cursor_x = if current_input.is_empty() {
        center_x() - tm.w / 2.0 + 5.0
    } else {
        center_x() - tm.w / 2.0 + tm.w + 3.0
    };
    draw_rectangle(cursor_x, input_y + 10.0, 2.0, 28.0);

    draw_number_pad();
    set_default_color();
}

/// Draws feedback for correct/incorrect gate answers.
pub fn draw_answer_feedback(is_correct: bool) {
    let (color, text) = if is_correct {
        (GREEN, "CORRECT!")
    } else {
        (RED, "INCORRECT - Try Again")
    };

    let feedback_font = screen::mobile_text_size(28);
    set_color(color);
    let tm = measure_text(text, None, feedback_font, 1.0);
    draw_text(
        text,
        center_x() - tm.w / 2.0,
        630.0,
        feedback_font as f32,
        color,
    );

    set_default_color();
}

fn draw_number_pad() {
    if mobile_html_overlay_controls() {
        return;
    }

    let labels = [
        "1", "2", "3", "4", "5", "6", "7", "8", "9", "DEL", "0", "OK",
    ];

    for (index, label) in labels.iter().enumerate() {
        let rect = keypad_button_rect(index);
        if screen::portrait_layout() {
            draw_mobile_yellow_button_rect(rect, label, if label.len() > 1 { 16 } else { 24 });
        } else {
            let accent = *label == "OK";

            set_color(if accent {
                Color::new(0.2, 0.78, 0.65, 0.92)
            } else {
                Color::new(0.16, 0.18, 0.28, 0.92)
            });
            draw_rectangle(rect.x, rect.y, rect.w, rect.h);
            set_color(if accent {
                WHITE
            } else {
                Color::new(0.5, 0.9, 1.0, 1.0)
            });
            draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h);

            let font_size = screen::mobile_text_size(if label.len() > 1 { 16 } else { 24 });
            let tm = measure_text(label, None, font_size, 1.0);
            draw_text(
                label,
                rect.x + rect.w / 2.0 - tm.w / 2.0,
                rect.y + rect.h / 2.0 + font_size as f32 / 3.0,
                font_size as f32,
                current_color(),
            );
        }
    }
}
