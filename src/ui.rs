use crate::assets;
use crate::levels::Grade;
use crate::screen::{SCREEN_H, SCREEN_W};
use macroquad::prelude::*;

const CENTER_X: f32 = SCREEN_W / 2.0;
pub const TITLE_MENU_X: f32 = 552.0;
pub const TITLE_MENU_Y: f32 = 206.0;
pub const TITLE_MENU_W: f32 = 332.0;
pub const TITLE_MENU_ROW_TOP: f32 = 276.0;
pub const TITLE_MENU_ROW_H: f32 = 54.0;
pub const TITLE_MENU_ROW_GAP: f32 = 18.0;
pub const KEYPAD_X: f32 = 704.0;
pub const KEYPAD_Y: f32 = 438.0;
pub const KEYPAD_KEY: f32 = 54.0;
pub const KEYPAD_GAP: f32 = 8.0;

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

fn measure_text(text: &str, font: Option<&Font>, font_size: u16, font_scale: f32) -> TextMeasure {
    let measure = macroquad::prelude::measure_text(text, font, font_size, font_scale);
    TextMeasure { w: measure.width }
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> Color {
    let a = s * l.min(1.0 - l);
    let channel = |n: f32| {
        let k = (n + h * 12.0) % 12.0;
        l - a * (k - 3.0).min(9.0 - k).clamp(-1.0, 1.0)
    };
    Color::new(channel(0.0), channel(8.0), channel(4.0), 1.0)
}

/// Draws the heads-up display (HUD) at top of screen.
pub fn draw_hud(grade: &Grade, score: u32, lives: u8, wave: usize, question_text: Option<&str>) {
    let portrait = screen_height() > screen_width() * 1.15;
    let font_size = if portrait { 28 } else { 16 };

    // Grade level indicator (top-left)
    set_color(WHITE);
    draw_text(
        format!("GRADE: {}", grade.display_name()).as_str(),
        10.0,
        740.0,
        font_size as f32,
        WHITE,
    );

    // Score (top-center)
    let score_txt = format!("SCORE: {}", score);
    let tm = measure_text(&score_txt, None, font_size as u16, 1.0);
    draw_text(
        &score_txt,
        CENTER_X - tm.w / 2.0,
        740.0,
        font_size as f32,
        YELLOW,
    );

    // Wave number (top-right)
    let wave_txt = format!("WAVE: {}", wave);
    draw_text(&wave_txt, 930.0, 740.0, font_size as f32, WHITE);

    // Lives display (bottom-left corner area)
    for i in 0..lives {
        assets::draw_life_icon(10.0 + (i as f32) * 20.0, 748.0);
    }

    // Active question text at bottom of screen (above player ship zone)
    if let Some(qtext) = question_text {
        draw_question_banner(qtext);
    }

    set_default_color();
}

/// Draws a semi-transparent banner with the current math question.
fn draw_question_banner(text: &str) {
    let portrait = screen_height() > screen_width() * 1.15;
    let lines: Vec<&str> = text.lines().collect();
    let banner_w = 760.0;
    let banner_x = CENTER_X - banner_w / 2.0;
    let banner_y = 28.0;
    let base_font_size = if lines.len() > 2 { 22 } else { 28 };
    let font_size = if portrait { (base_font_size as f32 * 1.5) as u16 } else { base_font_size };
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
            CENTER_X - tm.w / 2.0,
            banner_y + 28.0 + (i as f32) * line_h,
            font_size as f32,
            YELLOW,
        );
    }

    set_default_color();
}

/// Draws the RPG-style title screen and adventure menu.
pub fn draw_title_screen(showing_mini_games: bool, selected_index: usize) {
    let portrait = screen_height() > screen_width() * 1.15;
    let ink = Color::new(0.08, 0.1, 0.11, 1.0);
    let stone_dark = Color::new(0.2, 0.24, 0.24, 1.0);
    let stone = Color::new(0.46, 0.51, 0.48, 1.0);
    let stone_light = Color::new(0.72, 0.77, 0.68, 1.0);
    let parchment = Color::new(0.82, 0.86, 0.72, 1.0);
    let torch = Color::new(0.95, 0.75, 0.28, 1.0);

    let title_size = if portrait { 72 } else { 48 };
    let subtitle_size = if portrait { 40 } else { 26 };
    let hint_size = if portrait { 28 } else { 18 };
    let shortcut_size = if portrait { 24 } else { 16 };

    clear_background(Color::new(0.12, 0.15, 0.14, 1.0));
    draw_dungeon_tiles(stone_dark, ink);
    draw_stone_frame(54.0, 42.0, 916.0, 684.0, stone, stone_light, ink);

    centered_text("STAR CRUSHER", 116.0, title_size, stone_light);
    centered_text("DUNGEON DWELLERS", 154.0, subtitle_size, parchment);
    centered_text(
        "Choose a path, then press ENTER or SPACE.",
        654.0,
        hint_size,
        stone_light,
    );
    let shortcuts = if showing_mini_games {
        "ESC Back   P Pong   R Snake   N Nightmare"
    } else {
        "Shortcuts: M Math Invaders   P Mini Games   L Custom Spelling List"
    };
    centered_text(shortcuts, 680.0, shortcut_size, parchment);

    draw_title_scene(
        112.0,
        198.0,
        360.0,
        374.0,
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

fn draw_dungeon_tiles(stone_dark: Color, ink: Color) {
    for row in 0..16 {
        for col in 0..22 {
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
    for i in 0..18 {
        let bx = x + 22.0 + i as f32 * 50.0;
        draw_rectangle_lines(bx, y + 8.0, 42.0, 24.0);
        draw_rectangle_lines(bx, y + h - 32.0, 42.0, 24.0);
    }
    for i in 0..12 {
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
    set_color(Color::new(0.13, 0.16, 0.15, 1.0));
    draw_rectangle(x, y, w, h);
    set_color(stone_light);
    draw_rectangle_lines(x, y, w, h);

    draw_door_glyph(x + 132.0, y + 44.0, ink, stone, stone_light);
    draw_torch_glyph(x + 52.0, y + 82.0, ink, torch);
    draw_torch_glyph(x + 282.0, y + 82.0, ink, torch);
    draw_monster_glyph(x + 256.0, y + 238.0, ink, stone_light);
    draw_hero_glyph(x + 88.0, y + 248.0, ink, parchment);

    set_color(stone);
    for step in 0..5 {
        draw_rectangle(
            x + 128.0 + step as f32 * 12.0,
            y + 282.0 + step as f32 * 12.0,
            114.0 - step as f32 * 24.0,
            8.0,
        );
    }
}

fn draw_adventure_menu(
    x: f32,
    y: f32,
    w: f32,
    showing_mini_games: bool,
    selected_index: usize,
    ink: Color,
    stone: Color,
    stone_light: Color,
    parchment: Color,
) {
    let main_options = [
        ("Start Adventure", "Begin the dungeon trail"),
        ("Play Mini Games", "Open arcade side quests"),
        ("Custom Spelling List", "Enter weekly words"),
    ];
    let mini_game_options = [
        ("Reading Snake", "Word path encounter"),
        ("Math Pong", "Paddle target challenge"),
        ("Nightmare Snake", "Same-color letter trial"),
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
        let label_y = row_top + 24.0;
        let detail_y = row_top + 43.0;
        let selected = selected_index % options.len() == index;
        if selected {
            set_color(parchment);
            draw_rectangle(x + 22.0, row_top, w - 44.0, TITLE_MENU_ROW_H);
            set_color(ink);
            draw_rectangle(x + 32.0, row_top + 20.0, 10.0, 10.0);
            draw_text(label, x + 54.0, label_y, 20.0, ink);
            draw_text(detail, x + 54.0, detail_y, 12.0, ink);
        } else {
            set_color(stone);
            draw_rectangle_lines(x + 22.0, row_top, w - 44.0, TITLE_MENU_ROW_H);
            draw_text(label, x + 54.0, label_y, 20.0, stone_light);
            draw_text(detail, x + 54.0, detail_y, 12.0, parchment);
        }
    }

    if showing_mini_games {
        centered_text_in("Press ESC to return", x, y + 338.0, w, 14, parchment);
    }
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
    let tm = measure_text(text, None, font_size, 1.0);
    draw_text(text, x + w / 2.0 - tm.w / 2.0, y, font_size as f32, color);
}

const ADVENTURE_INTRO_PAGES: [(&str, &str); 5] = [
    ("THE MATH DUNGEON", "A Star Crusher Adventure"),
    (
        "Deep beneath the Crystal Mountains lies an ancient dungeon.",
        "Its stone halls glow with quiet number magic.",
    ),
    (
        "Its doors open only for heroes who can solve the number riddles.",
        "Every correct answer lights the path ahead.",
    ),
    (
        "You are the newest Star Crusher, brave, clever, and ready.",
        "Take your first steps into the torchlit halls.",
    ),
    (
        "First quest: clear the drifting number monsters.",
        "Math Invaders blocks await between challenges.",
    ),
];

pub fn adventure_intro_page_count() -> usize {
    ADVENTURE_INTRO_PAGES.len()
}

/// Draws the lightweight RPG intro before Start Adventure enters Math Invaders.
pub fn draw_adventure_intro(page: usize) {
    let portrait = screen_height() > screen_width() * 1.15;
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
    draw_stone_frame(68.0, 48.0, 888.0, 642.0, stone, stone_light, ink);

    draw_door_glyph(464.0, 164.0, ink, stone, stone_light);
    draw_torch_glyph(314.0, 218.0, ink, torch);
    draw_torch_glyph(684.0, 218.0, ink, torch);
    draw_hero_glyph(302.0, 432.0, ink, parchment);
    draw_monster_glyph(658.0, 420.0, ink, stone_light);

    set_color(stone);
    for step in 0..6 {
        draw_rectangle(
            424.0 + step as f32 * 16.0,
            330.0 + step as f32 * 18.0,
            176.0 - step as f32 * 32.0,
            10.0,
        );
    }

    let title_size = if portrait { 40 } else { 26 };
    let page_size = if portrait { 24 } else { 16 };
    let line1_size = if page == 0 { if portrait { 52 } else { 34 } } else { if portrait { 36 } else { 24 } };
    let line2_size = if page == 0 { if portrait { 34 } else { 22 } } else { if portrait { 30 } else { 20 } };
    let hint_size = if portrait { 26 } else { 18 };

    centered_text("START ADVENTURE", 104.0, title_size, parchment);
    centered_text_in(
        &format!("PAGE {} / {}", page + 1, ADVENTURE_INTRO_PAGES.len()),
        722.0,
        106.0,
        170.0,
        page_size,
        stone_light,
    );

    set_color(Color::new(0.1, 0.08, 0.12, 0.96));
    draw_rectangle(112.0, 552.0, 800.0, 116.0);
    set_color(parchment);
    draw_rectangle_lines(112.0, 552.0, 800.0, 116.0);

    let _title_card = page == 0;
    centered_text_in(
        line_one,
        142.0,
        596.0,
        740.0,
        line1_size,
        parchment,
    );
    centered_text_in(
        line_two,
        142.0,
        632.0,
        740.0,
        line2_size,
        WHITE,
    );
    centered_text(
        "ENTER / SPACE: continue     ESC: return to title",
        716.0,
        hint_size,
        stone_light,
    );

    set_default_color();
}

/// Draws the weekly spelling-list entry screen for Reading Snake.
pub fn draw_spelling_list_screen(input: &str) {
    for i in 0..80 {
        let x = ((17 + i * 11) % SCREEN_W as i32) as f32;
        let y = ((31 + i * 19) % (SCREEN_H as i32 - 40)) as f32;
        assets::draw_star(x, y, (i % 3) as f32 * 0.4 + 0.5);
    }

    centered_text(
        "WEEKLY SPELLING LIST",
        100.0,
        36,
        Color::new(0.4, 1.0, 0.65, 1.0),
    );
    centered_text(
        "Type word: definition pairs separated by semicolons.",
        148.0,
        20,
        WHITE,
    );
    centered_text("Plain word lists still work too.", 176.0, 18, GRAY);
    centered_text(
        "ENTER plays Reading Snake   N starts Nightmare",
        218.0,
        20,
        WHITE,
    );

    let input_w = 760.0;
    let input_h = 118.0;
    let input_x = CENTER_X - input_w / 2.0;
    let input_y = 278.0;

    set_color(Color::new(0.05, 0.12, 0.08, 0.95));
    draw_rectangle(input_x, input_y, input_w, input_h);
    set_color(Color::new(0.25, 0.75, 0.4, 1.0));
    draw_rectangle_lines(input_x, input_y, input_w, input_h);

    let shown_input = if input.is_empty() {
        "apple: a fruit; moon: shines at night"
    } else {
        input
    };
    let color = if input.is_empty() {
        Color::new(0.58, 0.68, 0.62, 1.0)
    } else {
        WHITE
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
    set_color(Color::new(0.3, 1.0, 0.5, blink));
    if input.is_empty() {
        draw_rectangle(input_x + 26.0, input_y + 78.0, 16.0, 3.0);
    }

    centered_text(
        "Leave it blank to use the default words.",
        456.0,
        18,
        YELLOW,
    );
    centered_text("Backspace deletes   ESC returns to title", 504.0, 18, GRAY);
    set_default_color();
}

fn centered_text(text: &str, y: f32, font_size: u16, color: Color) {
    let tm = measure_text(text, None, font_size, 1.0);
    draw_text(text, CENTER_X - tm.w / 2.0, y, font_size as f32, color);
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
pub fn draw_question_gate(grade: &Grade, math_topics: &str) {
    let portrait = screen_height() > screen_width() * 1.15;
    let title_size = if portrait { 48 } else { 32 };
    let grade_size = if portrait { 40 } else { 26 };
    let topic_size = if portrait { 32 } else { 20 };
    let instr_size = if portrait { 26 } else { 16 };

    let box_w = 640.0;
    let box_h = 280.0;
    let box_x = CENTER_X - box_w / 2.0;
    let box_y = 140.0;

    // Semi-transparent overlay
    set_color(Color::new(0.05, 0.05, 0.15, 0.85));
    draw_rectangle(0.0, 0.0, SCREEN_W, SCREEN_H);

    // Gate title
    let gate_title = "WAVE COMPLETE!";
    set_color(GREEN);
    let tm_gt = measure_text(gate_title, None, title_size, 1.0);
    draw_text(
        gate_title,
        CENTER_X - tm_gt.w / 2.0,
        80.0,
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
        CENTER_X - tm_gn.w / 2.0,
        180.0,
        grade_size as f32,
        current_color(),
    );

    // Math topics for this grade
    set_color(Color::new(0.7, 0.9, 1.0, 1.0));
    let topic_txt = format!("Topics: {}", math_topics);
    let tm_tp = measure_text(&topic_txt, None, topic_size, 1.0);
    draw_text(
        &topic_txt,
        CENTER_X - tm_tp.w / 2.0,
        230.0,
        topic_size as f32,
        current_color(),
    );

    // Instructions for gate questions
    let instructions = [
        "Answer the math question correctly to advance.",
        "",
        "Type your answer and press ENTER.",
        "",
        "Press SPACE or ENTER when ready!",
    ];

    set_color(WHITE);
    for (i, line) in instructions.iter().enumerate() {
        let tm = measure_text(line, None, instr_size, 1.0);
        draw_text(
            line,
            CENTER_X - tm.w / 2.0,
            300.0 + (i as f32) * 28.0,
            instr_size as f32,
            WHITE,
        );
    }

    set_default_color();
}

/// Draws the game over screen with final score and grade reached.
pub fn draw_game_over(score: u32, grade_reached: &Grade) {
    let portrait = screen_height() > screen_width() * 1.15;
    let title_size = if portrait { 72 } else { 48 };
    let score_size = if portrait { 42 } else { 28 };
    let grade_size = if portrait { 36 } else { 24 };
    let restart_size = if portrait { 30 } else { 20 };

    // Dark overlay
    set_color(Color::new(0.1, 0.05, 0.05, 0.9));
    draw_rectangle(0.0, 0.0, SCREEN_W, SCREEN_H);

    // Game Over title with pulsing effect
    let pulse = (get_time() as f32 * 3.0).sin() * 0.2 + 0.8;
    set_color(Color::new(1.0, 0.2, 0.2, pulse));

    let go_title = "GAME OVER";
    let tm_go = measure_text(go_title, None, title_size, 1.0);
    draw_text(
        go_title,
        CENTER_X - tm_go.w / 2.0,
        150.0,
        title_size as f32,
        current_color(),
    );

    // Stats box
    let box_w = 520.0;
    let box_h = 200.0;
    let box_x = CENTER_X - box_w / 2.0;
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
        CENTER_X - tm_sc.w / 2.0,
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
        CENTER_X - tm_gr.w / 2.0,
        320.0,
        grade_size as f32,
        current_color(),
    );

    // Restart prompt
    set_color(WHITE);
    let restart = "Press ENTER to Play Again";
    let tm_rs = measure_text(restart, None, restart_size, 1.0);
    draw_text(
        restart,
        CENTER_X - tm_rs.w / 2.0,
        450.0,
        restart_size as f32,
        current_color(),
    );

    set_default_color();
}

/// Draws the victory screen (completed all grades through 5th).
pub fn draw_victory_screen(score: u32) {
    let portrait = screen_height() > screen_width() * 1.15;
    let title_size = if portrait { 72 } else { 48 };
    let score_size = if portrait { 54 } else { 36 };
    let achievement_size = if portrait { 30 } else { 20 };
    let restart_size = if portrait { 28 } else { 18 };

    // Celebration overlay with gradient-like effect
    set_color(Color::new(0.1, 0.1, 0.2, 0.9));
    draw_rectangle(0.0, 0.0, SCREEN_W, SCREEN_H);

    // Victory title with rainbow cycling color
    let hue = (get_time() as f32 * 0.5) % 1.0;
    set_color(hsl_to_rgb(hue, 0.8, 0.7));

    let vic_title = "★ VICTORY! ★";
    let tm_vt = measure_text(vic_title, None, title_size, 1.0);
    draw_text(
        vic_title,
        CENTER_X - tm_vt.w / 2.0,
        150.0,
        title_size as f32,
        current_color(),
    );

    // Stats box
    let box_w = 620.0;
    let box_h = 220.0;
    let box_x = CENTER_X - box_w / 2.0;
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
        CENTER_X - tm_sc.w / 2.0,
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
            CENTER_X - tm.w / 2.0,
            340.0 + (i as f32) * 25.0,
            achievement_size as f32,
            WHITE,
        );
    }

    // Restart prompt
    let restart = "Press ENTER to Play Again";
    set_color(WHITE);
    let tm_rs = measure_text(restart, None, restart_size, 1.0);
    draw_text(
        restart,
        CENTER_X - tm_rs.w / 2.0,
        500.0,
        restart_size as f32,
        current_color(),
    );

    set_default_color();
}

/// Draws the current answer input display during question gates.
pub fn draw_answer_input(current_input: &str) {
    let input_w = 240.0;
    let input_h = 50.0;
    let input_x = CENTER_X - input_w / 2.0;
    let input_y = 540.0;

    // Input box background
    set_color(Color::new(0.2, 0.2, 0.4, 0.9));
    draw_rectangle(input_x, input_y, input_w, input_h);

    // Border (cyan for active input)
    set_color(Color::new(0.3, 1.0, 1.0, 1.0));
    draw_rectangle_lines(input_x, input_y, input_w, input_h);

    // Current typed answer (centered)
    set_color(WHITE);
    let tm = measure_text(current_input, None, 24, 1.0);
    draw_text(
        current_input,
        CENTER_X - tm.w / 2.0,
        input_y + 32.0,
        24.0,
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
        CENTER_X - tm.w / 2.0 + 5.0
    } else {
        CENTER_X - tm.w / 2.0 + tm.w + 3.0
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

    set_color(color);
    let tm = measure_text(text, None, 28, 1.0);
    draw_text(text, CENTER_X - tm.w / 2.0, 630.0, 28.0, color);

    set_default_color();
}

fn draw_number_pad() {
    let labels = [
        "1", "2", "3", "4", "5", "6", "7", "8", "9", "DEL", "0", "OK",
    ];

    for (index, label) in labels.iter().enumerate() {
        let col = index % 3;
        let row = index / 3;
        let x = KEYPAD_X + col as f32 * (KEYPAD_KEY + KEYPAD_GAP);
        let y = KEYPAD_Y + row as f32 * (KEYPAD_KEY + KEYPAD_GAP);
        let accent = *label == "OK";

        set_color(if accent {
            Color::new(0.2, 0.78, 0.65, 0.92)
        } else {
            Color::new(0.16, 0.18, 0.28, 0.92)
        });
        draw_rectangle(x, y, KEYPAD_KEY, KEYPAD_KEY);
        set_color(if accent {
            WHITE
        } else {
            Color::new(0.5, 0.9, 1.0, 1.0)
        });
        draw_rectangle_lines(x, y, KEYPAD_KEY, KEYPAD_KEY);

        let font_size = if label.len() > 1 { 16 } else { 24 };
        let tm = measure_text(label, None, font_size, 1.0);
        draw_text(
            label,
            x + KEYPAD_KEY / 2.0 - tm.w / 2.0,
            y + KEYPAD_KEY / 2.0 + font_size as f32 / 3.0,
            font_size as f32,
            current_color(),
        );
    }
}
