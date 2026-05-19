use crate::assets;
use crate::levels::Grade;
use macroquad::prelude::*;

const SCREEN_W: f32 = 1024.0;
const SCREEN_H: f32 = 768.0;
const CENTER_X: f32 = SCREEN_W / 2.0;

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
    let font_size = 16;

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
    // Banner background
    set_color(Color::new(0.1, 0.1, 0.25, 0.8));
    let lines: Vec<&str> = text.lines().collect();
    let banner_h = (lines.len() as f32 * 16.0 + 16.0).max(40.0);
    draw_rectangle(100.0, 500.0, 600.0, banner_h);

    // Banner border
    set_color(Color::new(0.3, 0.3, 0.7, 1.0));
    draw_rectangle_lines(100.0, 500.0, 600.0, banner_h);

    // Question text (centered in banner)
    set_color(YELLOW);
    let font_size = if lines.len() > 2 { 14 } else { 18 };
    for (i, line) in lines.iter().enumerate() {
        let tm = measure_text(line, None, font_size as u16, 1.0);
        draw_text(
            line,
            CENTER_X - tm.w / 2.0,
            510.0 + (i as f32) * 18.0,
            font_size as f32,
            YELLOW,
        );
    }

    set_default_color();
}

/// Draws the RPG-style title screen and adventure menu.
pub fn draw_title_screen(selected_index: usize) {
    let ink = Color::new(0.08, 0.1, 0.11, 1.0);
    let stone_dark = Color::new(0.2, 0.24, 0.24, 1.0);
    let stone = Color::new(0.46, 0.51, 0.48, 1.0);
    let stone_light = Color::new(0.72, 0.77, 0.68, 1.0);
    let parchment = Color::new(0.82, 0.86, 0.72, 1.0);
    let torch = Color::new(0.95, 0.75, 0.28, 1.0);

    clear_background(Color::new(0.12, 0.15, 0.14, 1.0));
    draw_dungeon_tiles(stone_dark, ink);
    draw_stone_frame(54.0, 42.0, 916.0, 684.0, stone, stone_light, ink);

    centered_text("STAR CRUSHER", 116.0, 48, stone_light);
    centered_text("DUNGEON DWELLERS", 154.0, 26, parchment);
    centered_text(
        "Choose a path, then press ENTER or SPACE.",
        688.0,
        18,
        stone_light,
    );
    centered_text(
        "Shortcuts: M Math Invaders   P Pong   R Snake   N Nightmare   L List",
        714.0,
        16,
        parchment,
    );

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
        552.0,
        206.0,
        332.0,
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
    selected_index: usize,
    ink: Color,
    stone: Color,
    stone_light: Color,
    parchment: Color,
) {
    let options = [
        ("Start Adventure", "Begin the dungeon trail"),
        ("Math Invaders", "Space wave encounter"),
        ("Math Pong", "Paddle target challenge"),
        ("Reading Snake", "Word path encounter"),
        ("Nightmare Snake", "Same-color letter trial"),
        ("Spelling List", "Enter custom words"),
    ];

    set_color(Color::new(0.16, 0.19, 0.18, 1.0));
    draw_rectangle(x, y, w, 382.0);
    set_color(stone_light);
    draw_rectangle_lines(x, y, w, 382.0);
    centered_text_in("ADVENTURE MENU", x, y + 42.0, w, 24, parchment);

    for (index, (label, detail)) in options.iter().enumerate() {
        let row_y = y + 72.0 + index as f32 * 48.0;
        let selected = selected_index % options.len() == index;
        if selected {
            set_color(parchment);
            draw_rectangle(x + 22.0, row_y - 24.0, w - 44.0, 38.0);
            set_color(ink);
            draw_rectangle(x + 32.0, row_y - 10.0, 10.0, 10.0);
            draw_text(label, x + 54.0, row_y, 22.0, ink);
            draw_text(detail, x + 54.0, row_y + 18.0, 13.0, ink);
        } else {
            set_color(stone);
            draw_rectangle_lines(x + 22.0, row_y - 24.0, w - 44.0, 38.0);
            draw_text(label, x + 54.0, row_y, 22.0, stone_light);
            draw_text(detail, x + 54.0, row_y + 18.0, 13.0, parchment);
        }
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
        "Press ENTER or SPACE to begin!",
    ),
];

pub fn adventure_intro_page_count() -> usize {
    ADVENTURE_INTRO_PAGES.len()
}

/// Draws the lightweight RPG intro before Start Adventure enters Math Invaders.
pub fn draw_adventure_intro(page: usize) {
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

    centered_text("START ADVENTURE", 104.0, 26, parchment);
    centered_text_in(
        &format!("PAGE {} / {}", page + 1, ADVENTURE_INTRO_PAGES.len()),
        722.0,
        106.0,
        170.0,
        16,
        stone_light,
    );

    set_color(Color::new(0.1, 0.08, 0.12, 0.96));
    draw_rectangle(112.0, 552.0, 800.0, 116.0);
    set_color(parchment);
    draw_rectangle_lines(112.0, 552.0, 800.0, 116.0);

    let title_card = page == 0;
    centered_text_in(
        line_one,
        142.0,
        596.0,
        740.0,
        if title_card { 34 } else { 24 },
        parchment,
    );
    centered_text_in(
        line_two,
        142.0,
        632.0,
        740.0,
        if title_card { 22 } else { 20 },
        WHITE,
    );
    centered_text(
        "ENTER / SPACE: continue     ESC: return to title",
        716.0,
        18,
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
        110.0,
        36,
        Color::new(0.4, 1.0, 0.65, 1.0),
    );
    centered_text(
        "Type word: definition pairs separated by semicolons.",
        155.0,
        20,
        WHITE,
    );
    centered_text("Plain word lists still work too.", 182.0, 18, GRAY);
    centered_text("Press ENTER to play Reading Snake.", 207.0, 20, WHITE);
    centered_text("Press N for Nightmare with this list.", 232.0, 18, GRAY);

    set_color(Color::new(0.05, 0.12, 0.08, 0.95));
    draw_rectangle(100.0, 270.0, 600.0, 120.0);
    set_color(Color::new(0.25, 0.75, 0.4, 1.0));
    draw_rectangle_lines(100.0, 270.0, 600.0, 120.0);

    let shown_input = if input.is_empty() {
        "apple: a fruit; moon: shines at night"
    } else {
        input
    };
    let color = if input.is_empty() { GRAY } else { WHITE };
    draw_wrapped_text(shown_input, 125.0, 310.0, 550.0, 22, color);

    let blink = if (get_time() as f32 * 3.0).fract() > 0.5 {
        1.0
    } else {
        0.0
    };
    set_color(Color::new(0.3, 1.0, 0.5, blink));
    draw_rectangle(126.0, 355.0, 16.0, 3.0);

    centered_text(
        "Leave it blank to use the default words.",
        435.0,
        18,
        YELLOW,
    );
    centered_text("Backspace deletes   ESC returns to title", 475.0, 18, GRAY);
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
    // Semi-transparent overlay
    set_color(Color::new(0.05, 0.05, 0.15, 0.85));
    draw_rectangle(0.0, 0.0, SCREEN_W, SCREEN_H);

    // Gate title
    let gate_title = "WAVE COMPLETE!";
    set_color(GREEN);
    let tm_gt = measure_text(gate_title, None, 32, 1.0);
    draw_text(
        gate_title,
        CENTER_X - tm_gt.w / 2.0,
        80.0,
        32.0,
        current_color(),
    );

    // Grade info box
    set_color(Color::new(0.15, 0.15, 0.3, 0.9));
    draw_rectangle(150.0, 140.0, 500.0, 280.0);

    // Border around info box
    set_color(Color::new(0.3, 0.6, 1.0, 1.0));
    draw_rectangle_lines(150.0, 140.0, 500.0, 280.0);

    // Grade name
    set_color(WHITE);
    let grade_txt = format!("Next: {}", grade.display_name());
    let tm_gn = measure_text(&grade_txt, None, 26, 1.0);
    draw_text(
        &grade_txt,
        CENTER_X - tm_gn.w / 2.0,
        180.0,
        26.0,
        current_color(),
    );

    // Math topics for this grade
    set_color(Color::new(0.7, 0.9, 1.0, 1.0));
    let topic_txt = format!("Topics: {}", math_topics);
    let tm_tp = measure_text(&topic_txt, None, 20, 1.0);
    draw_text(
        &topic_txt,
        CENTER_X - tm_tp.w / 2.0,
        230.0,
        20.0,
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
        let tm = measure_text(line, None, 16, 1.0);
        draw_text(
            line,
            CENTER_X - tm.w / 2.0,
            300.0 + (i as f32) * 28.0,
            16.0,
            WHITE,
        );
    }

    set_default_color();
}

/// Draws the game over screen with final score and grade reached.
pub fn draw_game_over(score: u32, grade_reached: &Grade) {
    // Dark overlay
    set_color(Color::new(0.1, 0.05, 0.05, 0.9));
    draw_rectangle(0.0, 0.0, SCREEN_W, SCREEN_H);

    // Game Over title with pulsing effect
    let pulse = (get_time() as f32 * 3.0).sin() * 0.2 + 0.8;
    set_color(Color::new(1.0, 0.2, 0.2, pulse));

    let go_title = "GAME OVER";
    let tm_go = measure_text(go_title, None, 48, 1.0);
    draw_text(
        go_title,
        CENTER_X - tm_go.w / 2.0,
        150.0,
        48.0,
        current_color(),
    );

    // Stats box
    set_color(Color::new(0.15, 0.15, 0.3, 0.9));
    draw_rectangle(200.0, 200.0, 400.0, 200.0);

    // Border
    set_color(Color::new(0.8, 0.3, 0.3, 1.0));
    draw_rectangle_lines(200.0, 200.0, 400.0, 200.0);

    // Final score
    set_color(YELLOW);
    let score_txt = format!("Final Score: {}", score);
    let tm_sc = measure_text(&score_txt, None, 28, 1.0);
    draw_text(
        &score_txt,
        CENTER_X - tm_sc.w / 2.0,
        260.0,
        28.0,
        current_color(),
    );

    // Grade reached
    set_color(WHITE);
    let grade_txt = format!("Grade Reached: {}", grade_reached.display_name());
    let tm_gr = measure_text(&grade_txt, None, 24, 1.0);
    draw_text(
        &grade_txt,
        CENTER_X - tm_gr.w / 2.0,
        320.0,
        24.0,
        current_color(),
    );

    // Restart prompt
    set_color(WHITE);
    let restart = "Press ENTER to Play Again";
    let tm_rs = measure_text(restart, None, 20, 1.0);
    draw_text(
        restart,
        CENTER_X - tm_rs.w / 2.0,
        450.0,
        20.0,
        current_color(),
    );

    set_default_color();
}

/// Draws the victory screen (completed all grades through 5th).
pub fn draw_victory_screen(score: u32) {
    // Celebration overlay with gradient-like effect
    set_color(Color::new(0.1, 0.1, 0.2, 0.9));
    draw_rectangle(0.0, 0.0, SCREEN_W, SCREEN_H);

    // Victory title with rainbow cycling color
    let hue = (get_time() as f32 * 0.5) % 1.0;
    set_color(hsl_to_rgb(hue, 0.8, 0.7));

    let vic_title = "★ VICTORY! ★";
    let tm_vt = measure_text(vic_title, None, 48, 1.0);
    draw_text(
        vic_title,
        CENTER_X - tm_vt.w / 2.0,
        150.0,
        48.0,
        current_color(),
    );

    // Stats box
    set_color(Color::new(0.15, 0.2, 0.3, 0.9));
    draw_rectangle(150.0, 200.0, 500.0, 220.0);

    // Border with gold color
    set_color(YELLOW);
    draw_rectangle_lines(150.0, 200.0, 500.0, 220.0);

    // Final score (large)
    let score_txt = format!("Final Score: {}", score);
    let tm_sc = measure_text(&score_txt, None, 36, 1.0);
    set_color(YELLOW);
    draw_text(
        &score_txt,
        CENTER_X - tm_sc.w / 2.0,
        280.0,
        36.0,
        current_color(),
    );

    // Achievement text
    let achievement = "You've mastered math from Preschool through 5th Grade!";
    set_color(WHITE);
    for (i, line) in achievement.lines().enumerate() {
        let tm = measure_text(line, None, 20, 1.0);
        draw_text(
            line,
            CENTER_X - tm.w / 2.0,
            340.0 + (i as f32) * 25.0,
            20.0,
            WHITE,
        );
    }

    // Restart prompt
    let restart = "Press ENTER to Play Again";
    set_color(WHITE);
    let tm_rs = measure_text(restart, None, 18, 1.0);
    draw_text(
        restart,
        CENTER_X - tm_rs.w / 2.0,
        500.0,
        18.0,
        current_color(),
    );

    set_default_color();
}

/// Draws the current answer input display during question gates.
pub fn draw_answer_input(current_input: &str) {
    // Input box background
    set_color(Color::new(0.2, 0.2, 0.4, 0.9));
    draw_rectangle(300.0, 480.0, 200.0, 50.0);

    // Border (cyan for active input)
    set_color(Color::new(0.3, 1.0, 1.0, 1.0));
    draw_rectangle_lines(300.0, 480.0, 200.0, 50.0);

    // Current typed answer (centered)
    set_color(WHITE);
    let tm = measure_text(current_input, None, 24, 1.0);
    draw_text(current_input, CENTER_X - tm.w / 2.0, 500.0, 24.0, WHITE);

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
    draw_rectangle(cursor_x, 490.0, 2.0, 28.0);

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
    draw_text(text, CENTER_X - tm.w / 2.0, 560.0, 28.0, color);

    set_default_color();
}
