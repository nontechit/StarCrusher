use crate::assets;
use crate::levels::Grade;
use macroquad::prelude::*;

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
        580.0,
        font_size as f32,
        WHITE,
    );

    // Score (top-center)
    let score_txt = format!("SCORE: {}", score);
    let tm = measure_text(&score_txt, None, font_size as u16, 1.0);
    draw_text(
        &score_txt,
        400.0 - tm.w / 2.0,
        580.0,
        font_size as f32,
        YELLOW,
    );

    // Wave number (top-right)
    let wave_txt = format!("WAVE: {}", wave);
    draw_text(&wave_txt, 790.0, 580.0, font_size as f32, WHITE);

    // Lives display (bottom-left corner area)
    for i in 0..lives {
        assets::draw_life_icon(10.0 + (i as f32) * 20.0, 590.0);
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
            400.0 - tm.w / 2.0,
            510.0 + (i as f32) * 18.0,
            font_size as f32,
            YELLOW,
        );
    }

    set_default_color();
}

/// Draws the title screen with game logo and instructions.
pub fn draw_title_screen() {
    // Background stars for atmosphere
    let seed = 42;
    for i in 0..80 {
        let x = ((seed + i * 7) % 800) as f32;
        let y = ((seed + i * 13) % 560) as f32;
        assets::draw_star(x, y, (i % 3) as f32 * 0.5 + 0.5);
    }

    // Title text with glow effect
    let title = "STAR CRUSHER";
    let tm_title = measure_text(title, None, 48, 1.0);
    set_color(Color::new(0.3, 0.6, 1.0, 1.0));
    draw_text(
        title,
        400.0 - tm_title.w / 2.0 + 2.0,
        150.0 + 2.0,
        48.0,
        current_color(),
    );

    set_color(WHITE);
    draw_text(
        title,
        400.0 - tm_title.w / 2.0,
        150.0,
        48.0,
        current_color(),
    );

    // Subtitle
    let subtitle = "Math Space Invaders";
    let tm_sub = measure_text(subtitle, None, 24, 1.0);
    set_color(Color::new(0.6, 0.9, 1.0, 1.0));
    draw_text(
        subtitle,
        400.0 - tm_sub.w / 2.0,
        200.0,
        24.0,
        current_color(),
    );

    // Instructions
    let instructions = [
        "Arrow Keys / A,D to Move",
        "Spacebar to Shoot",
        "",
        "Answer math questions by shooting the correct enemy!",
        "Clear each grade level to advance.",
        "",
        "Press ENTER or SPACE to Start Math Invaders",
        "Press R for Reading Snake",
    ];

    set_color(WHITE);
    for (i, line) in instructions.iter().enumerate() {
        let tm = measure_text(line, None, 18, 1.0);
        draw_text(
            line,
            400.0 - tm.w / 2.0,
            300.0 + (i as f32) * 25.0,
            18.0,
            WHITE,
        );
    }

    // Grade progression preview at bottom
    set_color(Color::new(0.4, 0.7, 0.9, 0.6));
    let grades = Grade::all();
    for (i, grade) in grades.iter().enumerate() {
        let _tm = measure_text(grade.display_name(), None, 12, 1.0);
        draw_text(
            grade.display_name(),
            50.0 + (i as f32) * 98.0,
            540.0,
            12.0,
            WHITE,
        );
    }

    set_default_color();
}

/// Draws the question gate screen between waves.
pub fn draw_question_gate(grade: &Grade, math_topics: &str) {
    // Semi-transparent overlay
    set_color(Color::new(0.05, 0.05, 0.15, 0.85));
    draw_rectangle(0.0, 0.0, 800.0, 600.0);

    // Gate title
    let gate_title = "WAVE COMPLETE!";
    set_color(GREEN);
    let tm_gt = measure_text(gate_title, None, 32, 1.0);
    draw_text(
        gate_title,
        400.0 - tm_gt.w / 2.0,
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
        400.0 - tm_gn.w / 2.0,
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
        400.0 - tm_tp.w / 2.0,
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
            400.0 - tm.w / 2.0,
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
    draw_rectangle(0.0, 0.0, 800.0, 600.0);

    // Game Over title with pulsing effect
    let pulse = (get_time() as f32 * 3.0).sin() * 0.2 + 0.8;
    set_color(Color::new(1.0, 0.2, 0.2, pulse));

    let go_title = "GAME OVER";
    let tm_go = measure_text(go_title, None, 48, 1.0);
    draw_text(
        go_title,
        400.0 - tm_go.w / 2.0,
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
        400.0 - tm_sc.w / 2.0,
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
        400.0 - tm_gr.w / 2.0,
        320.0,
        24.0,
        current_color(),
    );

    // Restart prompt
    set_color(WHITE);
    let restart = "Press ENTER to Play Again";
    let tm_rs = measure_text(restart, None, 20, 1.0);
    draw_text(restart, 400.0 - tm_rs.w / 2.0, 450.0, 20.0, current_color());

    set_default_color();
}

/// Draws the victory screen (completed all grades through 5th).
pub fn draw_victory_screen(score: u32) {
    // Celebration overlay with gradient-like effect
    set_color(Color::new(0.1, 0.1, 0.2, 0.9));
    draw_rectangle(0.0, 0.0, 800.0, 600.0);

    // Victory title with rainbow cycling color
    let hue = (get_time() as f32 * 0.5) % 1.0;
    set_color(hsl_to_rgb(hue, 0.8, 0.7));

    let vic_title = "★ VICTORY! ★";
    let tm_vt = measure_text(vic_title, None, 48, 1.0);
    draw_text(
        vic_title,
        400.0 - tm_vt.w / 2.0,
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
        400.0 - tm_sc.w / 2.0,
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
            400.0 - tm.w / 2.0,
            340.0 + (i as f32) * 25.0,
            20.0,
            WHITE,
        );
    }

    // Restart prompt
    let restart = "Press ENTER to Play Again";
    set_color(WHITE);
    let tm_rs = measure_text(restart, None, 18, 1.0);
    draw_text(restart, 400.0 - tm_rs.w / 2.0, 500.0, 18.0, current_color());

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
    draw_text(current_input, 400.0 - tm.w / 2.0, 500.0, 24.0, WHITE);

    // Blinking cursor effect
    let blink = if (get_time() as f32 * 3.0).fract() > 0.5 {
        1.0
    } else {
        0.0
    };
    set_color(Color::new(0.3, 1.0, 1.0, blink));
    let cursor_x = if current_input.is_empty() {
        400.0 - tm.w / 2.0 + 5.0
    } else {
        400.0 - tm.w / 2.0 + tm.w + 3.0
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
    draw_text(text, 400.0 - tm.w / 2.0, 560.0, 28.0, color);

    set_default_color();
}
