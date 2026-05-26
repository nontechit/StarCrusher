// Star Academy hub screen — Phase 1b
//
// This module owns all drawing and input logic for the hub screen and
// the grade-picker overlay.  It is called exclusively from main.rs and
// depends only on macroquad, screen, levels, and progress.
//
// Virtual coordinate space: 720 × 1280 (portrait) — see screen.rs.
// All layout constants are in those virtual units.

use crate::levels::Grade;
use crate::progress::{AcademyGame, PlayerProgress, STARS_TO_ADVANCE};
use crate::screen;
use macroquad::prelude::*;

// ── Layout constants ──────────────────────────────────────────────────────────

const SW: f32 = 720.0;  // virtual screen width
const SH: f32 = 1280.0; // virtual screen height
const CX: f32 = SW / 2.0;
const GUTTER: f32 = 30.0;
const CARD_X: f32 = GUTTER;
const CARD_W: f32 = SW - GUTTER * 2.0; // 660

// Header
const TITLE_Y: f32 = 76.0;
const BADGE_Y: f32 = 104.0;
const BADGE_H: f32 = 58.0;
const BADGE_W: f32 = 290.0;
const CHANGE_Y: f32 = 196.0; // centre of the tappable "Change Grade" row
const CHANGE_H: f32 = 50.0;

// Star meter
const METER_CY: f32 = 272.0; // centre-y of the pip row
const PIP_R: f32 = 15.0;
const PIP_GAP: f32 = 8.0;
const METER_LABEL_Y: f32 = 316.0;

// Game cards
const CARD_H: f32 = 144.0;
const CARD_GAP: f32 = 18.0;
const CARD_SECTION_LABEL_Y: f32 = 356.0;
const CARD_1_Y: f32 = 378.0;
const CARD_2_Y: f32 = CARD_1_Y + CARD_H + CARD_GAP; // 540
const CARD_3_Y: f32 = CARD_2_Y + CARD_H + CARD_GAP; // 702

// Colors (sRGB)
const C_BG: Color      = Color { r: 0.04, g: 0.04, b: 0.12, a: 1.0 };
const C_CARD: Color    = Color { r: 0.09, g: 0.09, b: 0.20, a: 1.0 };
const C_LABEL: Color   = Color { r: 0.55, g: 0.55, b: 0.72, a: 1.0 };
const C_STAR_ON: Color = Color { r: 1.00, g: 0.85, b: 0.10, a: 1.0 };
const C_STAR_OFF: Color= Color { r: 0.22, g: 0.22, b: 0.38, a: 1.0 };
const C_PLAY: Color    = Color { r: 0.12, g: 0.72, b: 0.38, a: 1.0 };

// Grade picker
const PICKER_BTN_X: f32 = GUTTER;
const PICKER_BTN_W: f32 = CARD_W;
const PICKER_BTN_H: f32 = 112.0;
const PICKER_BTN_GAP: f32 = 14.0;
const PICKER_BTN_TOP: f32 = 150.0;
const PICKER_TITLE_Y: f32 = 96.0;

// All 7 grades in display order.
const ALL_GRADES: [Grade; 7] = [
    Grade::Preschool,
    Grade::Kindergarten,
    Grade::FirstGrade,
    Grade::SecondGrade,
    Grade::ThirdGrade,
    Grade::FourthGrade,
    Grade::FifthGrade,
];

// ── Public action type ────────────────────────────────────────────────────────

/// Action returned by `update()` to the main game loop each frame.
pub enum HubAction {
    None,
    LaunchGame(AcademyGame),
    OpenGradePicker,
}

// ── Hub update ────────────────────────────────────────────────────────────────

/// Process tap input on the hub screen.  Returns an action for main.rs to act
/// on.  Must be called after `use_virtual_screen()` so coordinates match.
pub fn update(_progress: &PlayerProgress) -> HubAction {
    let Some(tap) = screen::primary_tap_position() else {
        return HubAction::None;
    };

    // "Change Grade" link
    if change_grade_rect().contains(tap) {
        return HubAction::OpenGradePicker;
    }

    // Game cards (all three)
    for (i, &game) in AcademyGame::all().iter().enumerate() {
        if card_rect(i).contains(tap) {
            return HubAction::LaunchGame(game);
        }
    }

    HubAction::None
}

// ── Hub draw ──────────────────────────────────────────────────────────────────

/// Draw the complete hub screen.  Background should already be cleared.
pub fn draw(progress: &PlayerProgress) {
    let grade = progress.grade();
    let accent = grade.enemy_color();

    draw_header(grade, accent);
    draw_star_meter(progress, accent);
    draw_game_cards(progress, accent);
}

fn draw_header(grade: Grade, accent: Color) {
    // Title
    centered_text("STAR  ACADEMY", TITLE_Y, 38, C_STAR_ON);

    // Grade badge (filled pill)
    let bx = CX - BADGE_W / 2.0;
    filled_pill(bx, BADGE_Y, BADGE_W, BADGE_H, 29.0, accent);
    let label = grade.display_name();
    let ls = fit_text(label, 34, BADGE_W - 24.0);
    centered_text_color(label, BADGE_Y + BADGE_H * 0.68, ls, dark_tint(0.05));

    // "Change Grade" link
    let cg_rect = change_grade_rect();
    // subtle hover / tap target background
    filled_pill(
        cg_rect.x,
        cg_rect.y,
        cg_rect.w,
        cg_rect.h,
        cg_rect.h / 2.0,
        Color { a: 0.12, ..accent },
    );
    centered_text_color("Change Grade  \u{25B8}", CHANGE_Y + 8.0, 26, C_LABEL);
}

fn draw_star_meter(progress: &PlayerProgress, accent: Color) {
    let stars = progress.current_stars() as usize;
    let total = STARS_TO_ADVANCE as usize;

    // 10 pips, centred
    let pip_d = PIP_R * 2.0;
    let total_w = pip_d * total as f32 + PIP_GAP * (total as f32 - 1.0);
    let start_x = CX - total_w / 2.0 + PIP_R;

    for i in 0..total {
        let cx = start_x + i as f32 * (pip_d + PIP_GAP);
        if i < stars {
            draw_circle(cx, METER_CY, PIP_R, C_STAR_ON);
            // inner shine
            draw_circle(cx - 4.0, METER_CY - 4.0, 5.0, Color { r: 1.0, g: 1.0, b: 0.9, a: 0.35 });
        } else {
            draw_circle(cx, METER_CY, PIP_R, C_STAR_OFF);
        }
    }

    // Label below meter
    let label = if stars >= total {
        "Grade meter full! Keep playing to advance.".to_string()
    } else {
        format!("{}/{} stars — fill to advance grade", stars, total)
    };
    centered_text_color(&label, METER_LABEL_Y, 20, C_LABEL);

    let _ = accent; // available for future tinting
}

fn draw_game_cards(progress: &PlayerProgress, accent: Color) {
    // Section label
    centered_text_color("MATH  GAMES", CARD_SECTION_LABEL_Y, 22, C_LABEL);

    for (i, &game) in AcademyGame::all().iter().enumerate() {
        let best = progress.best_for(game);
        draw_card(i, game, best, accent);
    }
}

fn draw_card(index: usize, game: AcademyGame, best: u8, accent: Color) {
    let r = card_rect(index);

    // Outer glow / border
    filled_pill(
        r.x - 2.0,
        r.y - 2.0,
        r.w + 4.0,
        r.h + 4.0,
        20.0,
        Color { a: 0.22, ..accent },
    );
    // Card body
    filled_pill(r.x, r.y, r.w, r.h, 18.0, C_CARD);

    // Left accent stripe
    let stripe = 7.0;
    // Draw stripe as a small rectangle + circle caps
    draw_rectangle(r.x, r.y + 18.0, stripe, r.h - 36.0, accent);
    draw_circle(r.x + stripe / 2.0, r.y + 18.0, stripe / 2.0, accent);
    draw_circle(r.x + stripe / 2.0, r.y + r.h - 18.0, stripe / 2.0, accent);

    // Icon circle
    let icon_cx = r.x + stripe + 16.0 + 34.0;
    let icon_cy = r.y + r.h / 2.0;
    draw_circle(icon_cx, icon_cy, 34.0, Color { a: 0.18, ..accent });
    draw_circle_lines(icon_cx, icon_cy, 34.0, 2.5, Color { a: 0.7, ..accent });
    // Initial letter as icon
    let initial = &game.display_name()[..1];
    let im = measure_text(initial, None, 30, 1.0);
    draw_text_ex(
        initial,
        icon_cx - im.width / 2.0,
        icon_cy + im.offset_y / 2.0,
        TextParams { font_size: 30, color: accent, ..Default::default() },
    );

    // Text column
    let tx = icon_cx + 46.0;
    let name_size = fit_text(game.display_name(), 30, r.x + r.w - tx - 110.0);
    draw_text_ex(
        game.display_name(),
        tx,
        r.y + 50.0,
        TextParams { font_size: name_size, color: WHITE, ..Default::default() },
    );
    let tag_size = fit_text(game.tagline(), 21, r.x + r.w - tx - 110.0);
    draw_text_ex(
        game.tagline(),
        tx,
        r.y + 82.0,
        TextParams { font_size: tag_size, color: C_LABEL, ..Default::default() },
    );

    // Right column: best stars + PLAY badge
    let right_x = r.x + r.w - 108.0;

    // Best stars (3 circles: filled = earned)
    let pip_spacing = 28.0;
    let star_cy = r.y + 44.0;
    for s in 0..3u8 {
        let sx = right_x + s as f32 * pip_spacing;
        let color = if s < best { C_STAR_ON } else { C_STAR_OFF };
        draw_circle(sx + 10.0, star_cy, 10.0, color);
    }

    // PLAY badge
    let pb_y = r.y + r.h - 46.0;
    let pb_w = 94.0;
    let pb_h = 34.0;
    filled_pill(right_x, pb_y, pb_w, pb_h, pb_h / 2.0, C_PLAY);
    let pm = measure_text("PLAY", None, 22, 1.0);
    draw_text_ex(
        "PLAY",
        right_x + (pb_w - pm.width) / 2.0,
        pb_y + pb_h * 0.72,
        TextParams { font_size: 22, color: WHITE, ..Default::default() },
    );
}

// ── Grade picker ──────────────────────────────────────────────────────────────

/// Draw the full-screen grade picker overlay (drawn on top of hub).
pub fn draw_grade_picker(current: Grade) {
    // Dim backdrop
    draw_rectangle(0.0, 0.0, SW, SH, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.82 });

    // Title
    centered_text("SELECT  GRADE", PICKER_TITLE_Y, 36, WHITE);

    for (i, &grade) in ALL_GRADES.iter().enumerate() {
        let y = picker_btn_y(i);
        let is_cur = grade == current;
        let accent = grade.enemy_color();

        if is_cur {
            // Filled with accent
            filled_pill(PICKER_BTN_X, y, PICKER_BTN_W, PICKER_BTN_H, 16.0, accent);
        } else {
            // Dark fill + subtle accent border
            filled_pill(PICKER_BTN_X, y, PICKER_BTN_W, PICKER_BTN_H, 16.0,
                Color { r: 0.10, g: 0.10, b: 0.22, a: 1.0 });
            // border: draw a slightly larger pill with accent then redraw fill on top
            // (approximated with rectangle_lines)
            draw_rectangle_lines(
                PICKER_BTN_X + 1.0, y + 1.0,
                PICKER_BTN_W - 2.0, PICKER_BTN_H - 2.0,
                2.0,
                Color { a: 0.45, ..accent },
            );
        }

        let text_color = if is_cur { dark_tint(0.05) } else { WHITE };
        centered_text_color(grade.display_name(), y + PICKER_BTN_H * 0.66, 32, text_color);
    }

    // Dismiss hint at bottom
    centered_text_color("Tap outside to cancel", SH - 48.0, 22, C_LABEL);
}

/// Returns the grade the player tapped, or None (including taps outside all buttons).
pub fn grade_picker_tap(tap: Vec2) -> Option<Grade> {
    for (i, &grade) in ALL_GRADES.iter().enumerate() {
        let y = picker_btn_y(i);
        if Rect::new(PICKER_BTN_X, y, PICKER_BTN_W, PICKER_BTN_H).contains(tap) {
            return Some(grade);
        }
    }
    None
}

// ── Helpers: geometry ─────────────────────────────────────────────────────────

fn card_rect(index: usize) -> Rect {
    let y = match index {
        0 => CARD_1_Y,
        1 => CARD_2_Y,
        _ => CARD_3_Y,
    };
    Rect::new(CARD_X, y, CARD_W, CARD_H)
}

fn change_grade_rect() -> Rect {
    Rect::new(CX - 160.0, CHANGE_Y - CHANGE_H / 2.0, 320.0, CHANGE_H)
}

fn picker_btn_y(index: usize) -> f32 {
    PICKER_BTN_TOP + index as f32 * (PICKER_BTN_H + PICKER_BTN_GAP)
}

// ── Helpers: drawing ──────────────────────────────────────────────────────────

/// Filled rounded rectangle (all corners equal radius).
fn filled_pill(x: f32, y: f32, w: f32, h: f32, radius: f32, color: Color) {
    let r = radius.min(w / 2.0).min(h / 2.0);
    draw_rectangle(x + r, y, w - r * 2.0, h, color);
    draw_rectangle(x, y + r, w, h - r * 2.0, color);
    draw_circle(x + r,     y + r,     r, color);
    draw_circle(x + w - r, y + r,     r, color);
    draw_circle(x + r,     y + h - r, r, color);
    draw_circle(x + w - r, y + h - r, r, color);
}

/// Draw white text centred horizontally at the given y baseline.
fn centered_text(text: &str, y: f32, size: u16, color: Color) {
    centered_text_color(text, y, size, color);
}

fn centered_text_color(text: &str, y: f32, size: u16, color: Color) {
    if text.is_empty() || size == 0 { return; }
    let m = measure_text(text, None, size, 1.0);
    draw_text_ex(
        text,
        CX - m.width / 2.0,
        y,
        TextParams { font_size: size, color, ..Default::default() },
    );
}

/// Shrink font size until the text fits within `max_w`.  Never goes below 14.
fn fit_text(text: &str, desired: u16, max_w: f32) -> u16 {
    let mut size = desired;
    while size > 14 && measure_text(text, None, size, 1.0).width > max_w {
        size -= 1;
    }
    size
}

/// A very dark colour tinted toward black for text on bright badge fills.
fn dark_tint(l: f32) -> Color {
    Color { r: l, g: l, b: l * 1.5, a: 1.0 }
}
