// Star Academy game #2: Number Rain
//
// Numbers fall from the top of the screen.  Tap the one that matches the
// math question before it hits the bottom.
//
// Tap correct  → +1 correct count, new question + wave.
// Tap wrong    → -1 life (red flash on that drop).
// Correct hits bottom without being tapped → -1 life, new question + wave.
// Wrong drop hits bottom → removed silently (no penalty).
//
// Win : WIN_CORRECT correct taps   → 3 stars.
// Lose: lives drop to 0            → partial stars by correct count.
//
// Stars: win→3, lose with ≥3→2, lose with 1-2→1, 0→0.

use crate::levels::Grade;
use crate::question::{generate_question, Question};
use crate::random;
use crate::screen;
use macroquad::prelude::*;

// ── Tunables ──────────────────────────────────────────────────────────────────

pub const WIN_CORRECT: u8 = 5;
pub const MAX_LIVES: u8 = 3;
/// Number of drops per wave (1 correct + rest wrong).
const WAVE_SIZE: usize = 4;
/// Seconds between consecutive drop spawns within a wave.
const SPAWN_GAP: f32 = 0.45;

const SW: f32 = 720.0;
const SH: f32 = 1280.0;
const CX: f32 = SW / 2.0;

/// Radius of each falling number circle.
const DROP_R: f32 = 46.0;
/// Y coordinate of the "bottom kill line" (drops removed here).
const BOTTOM_Y: f32 = 1140.0;
/// Horizontal play margin so drops don't hug the wall.
const PLAY_MARGIN: f32 = DROP_R + 20.0;

// Shield / home
const HOME_X: f32 = 24.0;
const HOME_Y: f32 = 24.0;
const HOME_W: f32 = 110.0;
const HOME_H: f32 = 56.0;

// Colors
const C_DROP: Color       = Color { r: 0.22, g: 0.45, b: 0.92, a: 1.0 };
const C_DROP_GLOW: Color  = Color { r: 0.30, g: 0.55, b: 1.00, a: 0.28 };
const C_CORRECT: Color    = Color { r: 0.18, g: 0.85, b: 0.42, a: 1.0 };
const C_WRONG: Color      = Color { r: 0.95, g: 0.25, b: 0.25, a: 1.0 };
const C_MISSED: Color     = Color { r: 0.95, g: 0.60, b: 0.10, a: 1.0 };
const C_LABEL: Color      = Color { r: 0.60, g: 0.60, b: 0.78, a: 1.0 };
const C_QUESTION: Color   = Color { r: 1.00, g: 0.92, b: 0.45, a: 1.0 };
const C_HEADER_BG: Color  = Color { r: 0.07, g: 0.07, b: 0.18, a: 0.93 };
const C_HOME_BG: Color    = Color { r: 0.16, g: 0.16, b: 0.28, a: 0.95 };
const C_HEART_ON: Color   = Color { r: 1.00, g: 0.32, b: 0.45, a: 1.0 };
const C_HEART_OFF: Color  = Color { r: 0.25, g: 0.18, b: 0.25, a: 1.0 };
const C_STREAK: Color     = Color { r: 0.35, g: 0.65, b: 1.00, a: 0.18 };

// ── Public action ─────────────────────────────────────────────────────────────

pub enum NumberRainAction {
    None,
    ExitToHub,
    Completed { stars: u8 },
}

// ── Drop ──────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
enum DropState {
    Falling,
    TappedCorrect, // correct tap — flash green
    TappedWrong,   // wrong tap   — flash red
    Missed,        // correct drop hit bottom without tap — flash orange
    Gone,          // wrong drop off-screen — remove silently
}

struct Drop {
    x: f32,
    y: f32,
    value: i64,
    is_correct: bool,
    speed: f32,
    state: DropState,
    flash_until: f64,
}

impl Drop {
    fn contains(&self, point: Vec2) -> bool {
        let dx = point.x - self.x;
        let dy = point.y - self.y;
        dx * dx + dy * dy <= DROP_R * DROP_R
    }
}

// ── Main struct ───────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
enum Phase {
    Playing,
    EndScreen,
}

pub struct NumberRain {
    grade: Grade,
    question: Question,
    /// Ordered list of (value, is_correct) yet to be spawned this wave.
    spawn_queue: Vec<(i64, bool)>,
    spawn_timer: f32,
    drops: Vec<Drop>,
    correct_count: u8,
    wrong_count: u8,
    phase: Phase,
    end_time: f64,
}

impl NumberRain {
    pub fn new(grade: Grade) -> Self {
        let question = generate_question(grade);
        let mut game = Self {
            grade,
            question,
            spawn_queue: Vec::new(),
            spawn_timer: SPAWN_GAP, // spawn first drop immediately
            drops: Vec::new(),
            correct_count: 0,
            wrong_count: 0,
            phase: Phase::Playing,
            end_time: 0.0,
        };
        game.queue_wave();
        game
    }

    fn lives_remaining(&self) -> u8 {
        MAX_LIVES.saturating_sub(self.wrong_count)
    }

    /// Fall speed in px/frame for this grade (60-fps normalized via frame_step).
    fn drop_speed_for(&self, is_correct: bool) -> f32 {
        let g = self.grade.index() as f32; // 0..=6
        let base = 2.0 + g * 0.25;
        // Give the correct drop a slight speed advantage so it isn't always
        // easiest to dodge — varies by ±10 % with a random offset.
        let jitter = random::f32_range(-0.2, 0.3);
        let factor = if is_correct { 1.05 } else { 1.0 };
        (base + jitter) * factor
    }

    /// Build spawn queue for the current question: 1 correct + (WAVE_SIZE-1)
    /// distractors, shuffled into random spawn order.
    fn queue_wave(&mut self) {
        self.spawn_queue.clear();
        self.spawn_timer = SPAWN_GAP;

        let mut wrongs: Vec<i64> = self.question.wrong_answers.clone();
        random::shuffle(&mut wrongs);
        wrongs.truncate(WAVE_SIZE - 1);

        // Pad if question didn't supply enough wrong answers.
        while wrongs.len() < WAVE_SIZE - 1 {
            let delta = random::i32_inclusive(1, 5) as i64
                * if random::bool(0.5) { 1 } else { -1 };
            let candidate = self.question.correct_answer + delta;
            if !wrongs.contains(&candidate) {
                wrongs.push(candidate);
            }
        }

        let mut wave: Vec<(i64, bool)> = vec![(self.question.correct_answer, true)];
        for v in wrongs {
            wave.push((v, false));
        }
        random::shuffle(&mut wave);
        self.spawn_queue = wave;
    }

    /// Pick an x position that avoids the top cluster of active drops.
    fn pick_x(&self) -> f32 {
        let min_x = PLAY_MARGIN;
        let max_x = SW - PLAY_MARGIN;
        for _ in 0..10 {
            let x = random::f32_range(min_x, max_x);
            let clash = self.drops.iter().any(|d| {
                d.y < DROP_R * 3.5 && (d.x - x).abs() < DROP_R * 2.2
            });
            if !clash {
                return x;
            }
        }
        random::f32_range(min_x, max_x)
    }

    // ── Update ────────────────────────────────────────────────────────────────

    pub fn update(&mut self) -> NumberRainAction {
        // HOME button — any phase
        if is_key_pressed(KeyCode::Escape) {
            return NumberRainAction::ExitToHub;
        }
        if let Some(tap) = screen::primary_tap_position() {
            if home_button_rect().contains(tap) {
                return NumberRainAction::ExitToHub;
            }
        }

        match self.phase {
            Phase::Playing   => self.update_playing(),
            Phase::EndScreen => self.update_end_screen(),
        }
    }

    fn update_playing(&mut self) -> NumberRainAction {
        self.handle_tap();
        self.spawn_drops();
        self.move_drops();
        self.resolve_drops();
        self.check_transition();
        NumberRainAction::None
    }

    fn handle_tap(&mut self) {
        let Some(tap) = screen::primary_tap_position() else { return };
        // HOME already handled above; ignore it here.
        if home_button_rect().contains(tap) {
            return;
        }
        // Find the topmost drop (lowest y number) the tap hits.
        let hit = self
            .drops
            .iter_mut()
            .filter(|d| d.state == DropState::Falling && d.contains(tap))
            .min_by(|a, b| a.y.partial_cmp(&b.y).unwrap());

        if let Some(d) = hit {
            if d.is_correct {
                d.state = DropState::TappedCorrect;
                d.flash_until = get_time() + 0.30;
            } else {
                d.state = DropState::TappedWrong;
                d.flash_until = get_time() + 0.25;
            }
        }
    }

    fn spawn_drops(&mut self) {
        if self.spawn_queue.is_empty() {
            return;
        }
        self.spawn_timer += get_frame_time();
        if self.spawn_timer >= SPAWN_GAP {
            self.spawn_timer = 0.0;
            let (value, is_correct) = self.spawn_queue.remove(0);
            let x = self.pick_x();
            self.drops.push(Drop {
                x,
                y: -DROP_R,
                value,
                is_correct,
                speed: self.drop_speed_for(is_correct),
                state: DropState::Falling,
                flash_until: 0.0,
            });
        }
    }

    fn move_drops(&mut self) {
        let step = screen::frame_step();
        for d in &mut self.drops {
            if d.state == DropState::Falling {
                d.y += d.speed * step;
                // Correct drop hit bottom while still falling → missed
                if d.y - DROP_R > BOTTOM_Y {
                    if d.is_correct {
                        d.state = DropState::Missed;
                        d.flash_until = get_time() + 0.40;
                    } else {
                        d.state = DropState::Gone;
                    }
                }
            }
        }
    }

    fn resolve_drops(&mut self) {
        let now = get_time();
        let mut correct_taps = 0u8;
        let mut life_losses = 0u8;
        let mut need_new_wave = false;

        self.drops.retain(|d| {
            match d.state {
                DropState::Falling => true,
                DropState::Gone    => false, // harmless wrong drop off-screen
                DropState::TappedCorrect if now >= d.flash_until => {
                    correct_taps += 1;
                    need_new_wave = true;
                    false
                }
                DropState::TappedWrong if now >= d.flash_until => {
                    life_losses += 1;
                    false
                }
                DropState::Missed if now >= d.flash_until => {
                    life_losses += 1;
                    need_new_wave = true;
                    false
                }
                _ => true, // still flashing
            }
        });

        self.correct_count = self.correct_count.saturating_add(correct_taps);
        self.wrong_count   = self.wrong_count.saturating_add(life_losses);

        if need_new_wave && self.phase == Phase::Playing {
            // Remove any lingering drops from the old wave, then start fresh.
            self.drops.retain(|d| d.state == DropState::Falling && !d.is_correct);
            self.question = generate_question(self.grade);
            self.queue_wave();
        }
    }

    fn check_transition(&mut self) {
        if self.correct_count >= WIN_CORRECT || self.wrong_count >= MAX_LIVES {
            self.drops.clear();
            self.phase = Phase::EndScreen;
            self.end_time = get_time();
        }
    }

    fn update_end_screen(&mut self) -> NumberRainAction {
        if get_time() - self.end_time < 0.6 {
            return NumberRainAction::None;
        }
        let tap = screen::primary_tap_position().is_some();
        let key = is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space);
        if tap || key {
            return NumberRainAction::Completed { stars: self.compute_stars() };
        }
        NumberRainAction::None
    }

    pub fn compute_stars(&self) -> u8 {
        if self.correct_count >= WIN_CORRECT {
            3
        } else if self.correct_count >= 3 {
            2
        } else if self.correct_count >= 1 {
            1
        } else {
            0
        }
    }

    // ── Draw ──────────────────────────────────────────────────────────────────

    pub fn draw(&self) {
        // Background
        draw_rectangle(0.0, 0.0, SW, SH, Color { r: 0.03, g: 0.04, b: 0.14, a: 1.0 });
        self.draw_rain_bg();
        self.draw_starfield();
        self.draw_header();

        for d in &self.drops {
            self.draw_drop(d);
        }

        self.draw_home_button();

        if self.phase == Phase::EndScreen {
            self.draw_end_screen();
        }
    }

    /// Subtle vertical rain-streak texture.
    fn draw_rain_bg(&self) {
        for i in 0..18u32 {
            let x = (i as f32 / 18.0) * SW + 14.0;
            draw_rectangle(x, 0.0, 2.0, SH, C_STREAK);
        }
    }

    fn draw_starfield(&self) {
        for i in 0..60 {
            let x = ((i * 83 + 11) % SW as i32) as f32;
            let y = ((i * 53 + 29) % (SH as i32 - 40)) as f32;
            draw_circle(x, y, 1.2, Color { r: 1.0, g: 1.0, b: 1.0, a: 0.35 });
        }
    }

    fn draw_header(&self) {
        // Panel
        filled_pill(20.0, 96.0, SW - 40.0, 110.0, 16.0, C_HEADER_BG);

        // Question
        let q = &self.question.text;
        let qs = fit_text(q, 44, SW - 100.0, 22);
        let m = measure_text(q, None, qs, 1.0);
        draw_text_ex(q, CX - m.width / 2.0, 166.0,
            TextParams { font_size: qs, color: C_QUESTION, ..Default::default() });

        // Score top-right
        let score = format!("{}/{}", self.correct_count, WIN_CORRECT);
        let sm = measure_text(&score, None, 28, 1.0);
        draw_text_ex(&score, SW - 30.0 - sm.width, 72.0,
            TextParams { font_size: 28, color: C_LABEL, ..Default::default() });

        // Hearts
        let heart_y = 196.0;
        let heart_r = 9.0;
        let heart_gap = 26.0;
        let start_x = CX - heart_gap;
        for i in 0..MAX_LIVES {
            let cx = start_x + i as f32 * heart_gap;
            let color = if i < self.lives_remaining() { C_HEART_ON } else { C_HEART_OFF };
            draw_circle(cx - 4.0, heart_y, heart_r, color);
            draw_circle(cx + 4.0, heart_y, heart_r, color);
            draw_triangle(
                Vec2::new(cx - 10.0, heart_y + 3.0),
                Vec2::new(cx + 10.0, heart_y + 3.0),
                Vec2::new(cx, heart_y + 14.0),
                color,
            );
        }

        // "TAP THE ANSWER" hint until first correct
        if self.correct_count == 0 && self.wrong_count == 0 {
            let hint = "Tap the correct answer!";
            let hm = measure_text(hint, None, 22, 1.0);
            draw_text_ex(hint, CX - hm.width / 2.0, 232.0,
                TextParams { font_size: 22, color: C_LABEL, ..Default::default() });
        }
    }

    fn draw_drop(&self, d: &Drop) {
        let (fill, glow) = match d.state {
            DropState::TappedCorrect => (C_CORRECT, Color { a: 0.40, ..C_CORRECT }),
            DropState::TappedWrong   => (C_WRONG,   Color { a: 0.35, ..C_WRONG   }),
            DropState::Missed        => (C_MISSED,  Color { a: 0.35, ..C_MISSED  }),
            _                        => (C_DROP,    C_DROP_GLOW),
        };

        // Motion streak above drop
        if d.state == DropState::Falling {
            for i in 1..=5u8 {
                let streak_y = d.y - DROP_R * 0.6 - i as f32 * 9.0;
                let alpha = 0.18 - i as f32 * 0.03;
                draw_circle(d.x, streak_y, DROP_R * 0.35, Color { r: glow.r, g: glow.g, b: glow.b, a: alpha.max(0.0) });
            }
        }

        // Glow halo
        draw_circle(d.x, d.y, DROP_R + 9.0, glow);
        // Body
        draw_circle(d.x, d.y, DROP_R, fill);
        // Rim
        draw_circle_lines(d.x, d.y, DROP_R, 2.5,
            Color { r: 1.0, g: 1.0, b: 1.0, a: 0.30 });
        // Shine
        draw_circle(d.x - DROP_R * 0.28, d.y - DROP_R * 0.28, DROP_R * 0.20,
            Color { r: 1.0, g: 1.0, b: 1.0, a: 0.22 });

        // Value text
        let text = d.value.to_string();
        let sz = fit_text(&text, 38, DROP_R * 1.65, 20);
        let tm = measure_text(&text, None, sz, 1.0);
        draw_text_ex(&text, d.x - tm.width / 2.0, d.y + tm.offset_y / 2.0,
            TextParams { font_size: sz, color: WHITE, ..Default::default() });
    }

    fn draw_home_button(&self) {
        let r = home_button_rect();
        filled_pill(r.x, r.y, r.w, r.h, r.h / 2.0, C_HOME_BG);
        let m = measure_text("HOME", None, 22, 1.0);
        draw_text_ex("HOME", r.x + (r.w - m.width) / 2.0, r.y + r.h * 0.66,
            TextParams { font_size: 22, color: WHITE, ..Default::default() });
    }

    fn draw_end_screen(&self) {
        draw_rectangle(0.0, 0.0, SW, SH, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.80 });

        let won = self.correct_count >= WIN_CORRECT;
        let title = if won { "WAVE COMPLETE!" } else { "GAME OVER" };
        let color = if won { C_CORRECT } else { C_WRONG };
        let tm = measure_text(title, None, 56, 1.0);
        draw_text_ex(title, CX - tm.width / 2.0, SH * 0.38,
            TextParams { font_size: 56, color, ..Default::default() });

        // Star pips
        let stars = self.compute_stars();
        let r = 30.0;
        let gap = 24.0;
        let start_x = CX - (r * 3.0 + gap);
        for i in 0..3u8 {
            let cx = start_x + i as f32 * (r * 2.0 + gap) + r;
            let col = if i < stars {
                Color { r: 1.0, g: 0.85, b: 0.10, a: 1.0 }
            } else {
                Color { r: 0.25, g: 0.25, b: 0.40, a: 1.0 }
            };
            draw_circle(cx, SH * 0.49, r, col);
            if i < stars {
                draw_circle(cx - r * 0.28, SH * 0.49 - r * 0.28, r * 0.22,
                    Color { r: 1.0, g: 1.0, b: 0.9, a: 0.45 });
            }
        }

        let summary = format!("Correct: {}/{}", self.correct_count, WIN_CORRECT);
        let sm = measure_text(&summary, None, 30, 1.0);
        draw_text_ex(&summary, CX - sm.width / 2.0, SH * 0.62,
            TextParams { font_size: 30, color: C_LABEL, ..Default::default() });

        if get_time() - self.end_time >= 0.6 {
            let prompt = "Tap to return to hub";
            let pm = measure_text(prompt, None, 26, 1.0);
            draw_text_ex(prompt, CX - pm.width / 2.0, SH * 0.72,
                TextParams { font_size: 26, color: WHITE, ..Default::default() });
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn home_button_rect() -> Rect {
    Rect::new(HOME_X, HOME_Y, HOME_W, HOME_H)
}

fn filled_pill(x: f32, y: f32, w: f32, h: f32, radius: f32, color: Color) {
    let r = radius.min(w / 2.0).min(h / 2.0);
    draw_rectangle(x + r, y, w - r * 2.0, h, color);
    draw_rectangle(x, y + r, w, h - r * 2.0, color);
    draw_circle(x + r,     y + r,     r, color);
    draw_circle(x + w - r, y + r,     r, color);
    draw_circle(x + r,     y + h - r, r, color);
    draw_circle(x + w - r, y + h - r, r, color);
}

fn fit_text(text: &str, desired: u16, max_w: f32, min_size: u16) -> u16 {
    let mut size = desired;
    while size > min_size && measure_text(text, None, size, 1.0).width > max_w {
        size -= 1;
    }
    size
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make(correct: u8, wrong: u8) -> NumberRain {
        let mut g = NumberRain::new(Grade::Preschool);
        g.correct_count = correct;
        g.wrong_count   = wrong;
        g
    }

    #[test]
    fn star_award_won() {
        assert_eq!(make(WIN_CORRECT, 0).compute_stars(), 3);
        assert_eq!(make(WIN_CORRECT, 2).compute_stars(), 3);
    }

    #[test]
    fn star_award_partial() {
        assert_eq!(make(4, MAX_LIVES).compute_stars(), 2);
        assert_eq!(make(3, MAX_LIVES).compute_stars(), 2);
        assert_eq!(make(2, MAX_LIVES).compute_stars(), 1);
        assert_eq!(make(1, MAX_LIVES).compute_stars(), 1);
        assert_eq!(make(0, MAX_LIVES).compute_stars(), 0);
    }

    #[test]
    fn lives_clamp_to_zero() {
        let mut g = NumberRain::new(Grade::Preschool);
        g.wrong_count = 99;
        assert_eq!(g.lives_remaining(), 0);
    }

    #[test]
    fn wave_has_exactly_one_correct() {
        let g = NumberRain::new(Grade::FirstGrade);
        let correct = g.spawn_queue.iter().filter(|(_, c)| *c).count();
        assert_eq!(correct, 1);
        assert_eq!(g.spawn_queue.len(), WAVE_SIZE);
    }

    #[test]
    fn wave_distractors_differ_from_correct() {
        let g = NumberRain::new(Grade::ThirdGrade);
        let answer = g.question.correct_answer;
        for (val, is_correct) in &g.spawn_queue {
            if !*is_correct {
                assert_ne!(*val, answer, "distractor should not equal correct answer");
            }
        }
    }
}
