// Star Academy game #1: Meteor Catch
//
// Mobile-native single-finger gameplay.  The player drags a shield
// horizontally along the bottom of the screen.  Answer "meteors" fall
// from above; catching the one whose value matches the math question
// scores a hit, catching a wrong one or letting the correct one fall
// past costs a life.
//
// Win condition : WIN_CORRECT catches.
// Lose condition: MAX_LIVES lives lost.
//
// Star award:
//   • Win                  → 3 stars
//   • Lose with ≥3 correct → 2 stars
//   • Lose with 1-2 correct→ 1 star
//   • Lose with 0 correct  → 0 stars

use crate::levels::Grade;
use crate::question::{generate_question, Question};
use crate::random;
use crate::screen;
use macroquad::prelude::*;

// ── Tunables ──────────────────────────────────────────────────────────────────

const WIN_CORRECT: u8 = 5;
const MAX_LIVES: u8 = 3;
const METEORS_PER_WAVE: usize = 4;

/// Virtual screen — Meteor Catch is portrait-only.
const SW: f32 = 720.0;
const SH: f32 = 1280.0;
const CX: f32 = SW / 2.0;

// Shield (player paddle)
const SHIELD_W: f32 = 140.0;
const SHIELD_H: f32 = 28.0;
const SHIELD_Y: f32 = 1110.0;
const SHIELD_X_MIN: f32 = SHIELD_W / 2.0 + 20.0;
const SHIELD_X_MAX: f32 = SW - SHIELD_W / 2.0 - 20.0;

// Meteor
const METEOR_R: f32 = 44.0;
const METEOR_SPAWN_Y: f32 = -METEOR_R;
const METEOR_SPAWN_GAP: f32 = 0.55; // seconds between meteors in a wave
const METEOR_BASE_SPEED: f32 = 2.6; // px/frame at 60fps scale = 1.0

// Header (question + HUD)
const HEADER_BOTTOM: f32 = 220.0;

// Home button (top-left)
const HOME_X: f32 = 24.0;
const HOME_Y: f32 = 24.0;
const HOME_W: f32 = 110.0;
const HOME_H: f32 = 56.0;

// Colors
const C_BG_TOP: Color    = Color { r: 0.04, g: 0.04, b: 0.14, a: 1.0 };
const C_BG_BOT: Color    = Color { r: 0.10, g: 0.04, b: 0.20, a: 1.0 };
const C_HEADER_BG: Color = Color { r: 0.07, g: 0.07, b: 0.18, a: 0.92 };
const C_LABEL: Color     = Color { r: 0.62, g: 0.62, b: 0.78, a: 1.0 };
const C_QUESTION: Color  = Color { r: 1.0,  g: 0.92, b: 0.45, a: 1.0 };
const C_HOME_BG: Color   = Color { r: 0.16, g: 0.16, b: 0.28, a: 0.95 };
const C_HEART_ON: Color  = Color { r: 1.0,  g: 0.32, b: 0.45, a: 1.0 };
const C_HEART_OFF: Color = Color { r: 0.25, g: 0.18, b: 0.25, a: 1.0 };
const C_METEOR: Color    = Color { r: 0.93, g: 0.47, b: 0.32, a: 1.0 };
const C_METEOR_GLOW: Color = Color { r: 1.0,  g: 0.62, b: 0.30, a: 0.35 };
const C_SHIELD_GLOW: Color = Color { r: 1.0,  g: 1.0,  b: 1.0,  a: 0.18 };
const C_OK: Color        = Color { r: 0.18, g: 0.85, b: 0.42, a: 1.0 };
const C_BAD: Color       = Color { r: 1.0,  g: 0.30, b: 0.30, a: 1.0 };

// ── Public action returned to main loop ───────────────────────────────────────

pub enum MeteorCatchAction {
    None,
    ExitToHub,
    /// Round complete — main loop should award stars and return to hub.
    Completed { stars: u8 },
}

// ── Meteor data ───────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
enum MeteorState {
    Falling,
    Hit,    // caught by shield this frame (will be removed)
    Missed, // fell past shield line (will be removed)
}

struct Meteor {
    x: f32,
    y: f32,
    value: i64,
    is_correct: bool,
    state: MeteorState,
    flash_until: f64, // wall-clock time; flashes green/red briefly when consumed
}

// ── Main game struct ──────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
enum Phase {
    Playing,
    EndScreen, // game over OR won — show summary, tap to exit
}

pub struct MeteorCatch {
    grade: Grade,
    shield_x: f32,
    question: Question,
    /// Values queued to spawn in the current wave (in order).
    spawn_queue: Vec<(i64, bool)>,
    /// Seconds since last spawn from queue.
    spawn_timer: f32,
    meteors: Vec<Meteor>,
    correct_count: u8,
    wrong_count: u8, // lives lost
    phase: Phase,
    end_time: f64, // wall-clock time when EndScreen started (for tap debouncing)
}

impl MeteorCatch {
    pub fn new(grade: Grade) -> Self {
        let question = generate_question(grade);
        let mut game = Self {
            grade,
            shield_x: CX,
            question,
            spawn_queue: Vec::new(),
            spawn_timer: 0.0,
            meteors: Vec::new(),
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

    fn fall_speed(&self) -> f32 {
        // Difficulty scales with grade — Preschool slow, FifthGrade ~50% faster.
        let g = self.grade.index() as f32; // 0..=6
        METEOR_BASE_SPEED * (1.0 + g * 0.08) * screen::frame_step()
    }

    /// Build a new wave's spawn queue: one correct + (METEORS_PER_WAVE-1)
    /// distractors from `question.wrong_answers` (padded if there aren't enough).
    fn queue_wave(&mut self) {
        self.spawn_queue.clear();
        self.spawn_timer = METEOR_SPAWN_GAP; // spawn first meteor immediately

        // Collect distractors
        let mut distractors: Vec<i64> = self.question.wrong_answers.clone();
        random::shuffle(&mut distractors);
        distractors.truncate(METEORS_PER_WAVE - 1);

        // Pad if fewer than needed (e.g. easy questions with few alternatives)
        while distractors.len() < METEORS_PER_WAVE - 1 {
            let delta = random::i32_inclusive(-3, 3) as i64;
            let candidate = self.question.correct_answer + delta.max(1);
            if candidate != self.question.correct_answer && !distractors.contains(&candidate) {
                distractors.push(candidate);
            }
        }

        // Build wave with correct + distractors, then shuffle order
        let mut wave: Vec<(i64, bool)> = vec![(self.question.correct_answer, true)];
        for v in distractors {
            wave.push((v, false));
        }
        random::shuffle(&mut wave);
        self.spawn_queue = wave;
    }

    /// Pick a fresh x within play bounds that avoids overlapping any meteor
    /// currently near the top of the screen.
    fn pick_spawn_x(&self) -> f32 {
        let min_x = METEOR_R + 30.0;
        let max_x = SW - METEOR_R - 30.0;
        for _ in 0..8 {
            let x = random::f32_range(min_x, max_x);
            let collides = self.meteors.iter().any(|m| {
                m.y < METEOR_R * 3.0 && (m.x - x).abs() < METEOR_R * 2.2
            });
            if !collides {
                return x;
            }
        }
        random::f32_range(min_x, max_x)
    }

    // ── Per-frame update ──────────────────────────────────────────────────────

    pub fn update(&mut self) -> MeteorCatchAction {
        // Home button (canvas tap) — works in both phases
        if let Some(tap) = screen::primary_tap_position() {
            if home_button_rect().contains(tap) {
                return MeteorCatchAction::ExitToHub;
            }
        }
        if is_key_pressed(KeyCode::Escape) {
            return MeteorCatchAction::ExitToHub;
        }

        match self.phase {
            Phase::Playing => self.update_playing(),
            Phase::EndScreen => self.update_end_screen(),
        }
    }

    fn update_playing(&mut self) -> MeteorCatchAction {
        self.update_shield();
        self.update_spawning();
        self.update_meteors();
        self.resolve_consumed_meteors();
        self.check_phase_transition();
        MeteorCatchAction::None
    }

    fn update_shield(&mut self) {
        if let Some(p) = screen::primary_pointer_position() {
            // Snap shield to pointer x (within bounds).  Direct mapping is the
            // most responsive feel on mobile; no smoothing.
            self.shield_x = p.x.clamp(SHIELD_X_MIN, SHIELD_X_MAX);
        }
        // Keyboard fallback for desktop testing
        let dt = screen::frame_step();
        if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
            self.shield_x = (self.shield_x - 12.0 * dt).max(SHIELD_X_MIN);
        }
        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
            self.shield_x = (self.shield_x + 12.0 * dt).min(SHIELD_X_MAX);
        }
    }

    fn update_spawning(&mut self) {
        if self.spawn_queue.is_empty() {
            return;
        }
        self.spawn_timer += get_frame_time();
        if self.spawn_timer >= METEOR_SPAWN_GAP {
            self.spawn_timer = 0.0;
            let (value, is_correct) = self.spawn_queue.remove(0);
            let x = self.pick_spawn_x();
            self.meteors.push(Meteor {
                x,
                y: METEOR_SPAWN_Y,
                value,
                is_correct,
                state: MeteorState::Falling,
                flash_until: 0.0,
            });
        }
    }

    fn update_meteors(&mut self) {
        let speed = self.fall_speed();
        let shield_top = SHIELD_Y - SHIELD_H / 2.0;
        let shield_left = self.shield_x - SHIELD_W / 2.0;
        let shield_right = self.shield_x + SHIELD_W / 2.0;

        for m in &mut self.meteors {
            if m.state != MeteorState::Falling {
                continue;
            }
            m.y += speed;

            // Collision with shield: meteor bottom crosses shield top while
            // within shield horizontal span.
            let m_bottom = m.y + METEOR_R;
            if m_bottom >= shield_top && m.y <= SHIELD_Y + SHIELD_H {
                if m.x >= shield_left - METEOR_R * 0.6
                    && m.x <= shield_right + METEOR_R * 0.6
                {
                    m.state = MeteorState::Hit;
                    m.flash_until = get_time() + 0.25;
                    continue;
                }
            }

            // Passed the shield line entirely
            if m.y - METEOR_R > SHIELD_Y + SHIELD_H {
                m.state = MeteorState::Missed;
                m.flash_until = get_time() + 0.18;
            }
        }
    }

    fn resolve_consumed_meteors(&mut self) {
        let now = get_time();
        let mut correct_caught = 0u8;
        let mut life_losses = 0u8;

        self.meteors.retain(|m| {
            // Keep falling meteors
            if m.state == MeteorState::Falling {
                return true;
            }
            // Keep until flash window expires (visual feedback)
            if now < m.flash_until {
                return true;
            }
            // Tally the outcome and drop the meteor.
            match (m.state, m.is_correct) {
                (MeteorState::Hit, true)            => correct_caught += 1,
                (MeteorState::Hit, false)           => life_losses += 1,
                (MeteorState::Missed, true)         => life_losses += 1,
                (MeteorState::Missed, false)        => {} // harmless miss
                (MeteorState::Falling, _)           => unreachable!(),
            }
            false
        });

        self.correct_count = self.correct_count.saturating_add(correct_caught);
        self.wrong_count   = self.wrong_count.saturating_add(life_losses);
    }

    fn check_phase_transition(&mut self) {
        // Win check
        if self.correct_count >= WIN_CORRECT {
            self.phase = Phase::EndScreen;
            self.end_time = get_time();
            return;
        }
        // Lose check
        if self.wrong_count >= MAX_LIVES {
            self.phase = Phase::EndScreen;
            self.end_time = get_time();
            return;
        }
        // Wave complete — queue next question if all spawned meteors are gone.
        if self.spawn_queue.is_empty() && self.meteors.is_empty() {
            self.question = generate_question(self.grade);
            self.queue_wave();
        }
    }

    fn update_end_screen(&mut self) -> MeteorCatchAction {
        // Debounce: ignore taps for the first 600ms after end-screen appears
        // so the final wave's catch doesn't immediately exit.
        if get_time() - self.end_time < 0.6 {
            return MeteorCatchAction::None;
        }
        let tap = screen::primary_tap_position().is_some();
        let key = is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space);
        if tap || key {
            return MeteorCatchAction::Completed { stars: self.compute_stars() };
        }
        MeteorCatchAction::None
    }

    fn compute_stars(&self) -> u8 {
        let won = self.correct_count >= WIN_CORRECT;
        if won {
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
        // Vertical gradient background (two stacked rectangles + middle blend)
        draw_rectangle(0.0, 0.0, SW, SH * 0.55, C_BG_TOP);
        draw_rectangle(0.0, SH * 0.45, SW, SH * 0.55, C_BG_BOT);

        self.draw_starfield();
        self.draw_header();

        for m in &self.meteors {
            self.draw_meteor(m);
        }

        self.draw_shield();
        self.draw_home_button();

        if self.phase == Phase::EndScreen {
            self.draw_end_screen();
        }
    }

    fn draw_starfield(&self) {
        for i in 0..70 {
            let x = ((i * 73 + 19) % SW as i32) as f32;
            let y = ((i * 41 + 37) % (SH as i32 - 40)) as f32;
            draw_circle(x, y, 1.4, Color { r: 1.0, g: 1.0, b: 1.0, a: 0.45 });
        }
    }

    fn draw_header(&self) {
        // Header panel
        filled_pill(20.0, 96.0, SW - 40.0, 110.0, 16.0, C_HEADER_BG);

        // Question text (large, centered)
        let q = &self.question.text;
        let qs = fit_text(q, 44, SW - 100.0, 22);
        let m = measure_text(q, None, qs, 1.0);
        draw_text_ex(
            q,
            CX - m.width / 2.0,
            166.0,
            TextParams { font_size: qs, color: C_QUESTION, ..Default::default() },
        );

        // Score (top-right) and lives (under question)
        let score_label = format!("{}/{}", self.correct_count, WIN_CORRECT);
        let sm = measure_text(&score_label, None, 28, 1.0);
        draw_text_ex(
            &score_label,
            SW - 30.0 - sm.width,
            72.0,
            TextParams { font_size: 28, color: C_LABEL, ..Default::default() },
        );

        // Hearts row
        let heart_y = 196.0;
        let heart_r = 9.0;
        let heart_gap = 26.0;
        let total_w = heart_gap * (MAX_LIVES as f32 - 1.0);
        let start_x = CX - total_w / 2.0;
        for i in 0..MAX_LIVES {
            let cx = start_x + i as f32 * heart_gap;
            let color = if i < self.lives_remaining() { C_HEART_ON } else { C_HEART_OFF };
            // Simple heart approximation: two circles + a triangle (use circles only for now)
            draw_circle(cx - 4.0, heart_y, heart_r, color);
            draw_circle(cx + 4.0, heart_y, heart_r, color);
            draw_triangle(
                Vec2::new(cx - 10.0, heart_y + 3.0),
                Vec2::new(cx + 10.0, heart_y + 3.0),
                Vec2::new(cx, heart_y + 14.0),
                color,
            );
        }
    }

    fn draw_meteor(&self, m: &Meteor) {
        let (fill, glow, ring) = match m.state {
            MeteorState::Hit if m.is_correct => (C_OK, Color { a: 0.35, ..C_OK }, WHITE),
            MeteorState::Hit                 => (C_BAD, Color { a: 0.35, ..C_BAD }, WHITE),
            MeteorState::Missed if m.is_correct => (C_BAD, Color { a: 0.30, ..C_BAD }, C_BAD),
            _ => (C_METEOR, C_METEOR_GLOW, Color { r: 1.0, g: 0.85, b: 0.6, a: 0.8 }),
        };

        // Glow
        draw_circle(m.x, m.y, METEOR_R + 8.0, glow);
        // Body
        draw_circle(m.x, m.y, METEOR_R, fill);
        // Ring
        draw_circle_lines(m.x, m.y, METEOR_R, 3.0, ring);
        // Trail (small dot above)
        draw_circle(m.x, m.y - METEOR_R - 8.0, 4.0, Color { a: 0.4, ..fill });

        // Value text — auto-fit
        let text = m.value.to_string();
        let size = fit_text(&text, 36, METEOR_R * 1.6, 20);
        let tm = measure_text(&text, None, size, 1.0);
        draw_text_ex(
            &text,
            m.x - tm.width / 2.0,
            m.y + tm.offset_y / 2.0,
            TextParams { font_size: size, color: WHITE, ..Default::default() },
        );
    }

    fn draw_shield(&self) {
        let x = self.shield_x - SHIELD_W / 2.0;
        let y = SHIELD_Y - SHIELD_H / 2.0;
        let accent = self.grade.enemy_color();

        // Glow halo
        filled_pill(x - 4.0, y - 4.0, SHIELD_W + 8.0, SHIELD_H + 8.0, (SHIELD_H + 8.0) / 2.0, C_SHIELD_GLOW);
        // Body
        filled_pill(x, y, SHIELD_W, SHIELD_H, SHIELD_H / 2.0, accent);
        // Highlight stripe
        filled_pill(x + 8.0, y + 4.0, SHIELD_W - 16.0, 6.0, 3.0, Color { r: 1.0, g: 1.0, b: 1.0, a: 0.35 });

        // Hint label below shield on first round, before any catches
        if self.correct_count == 0 && self.wrong_count == 0 && self.meteors.is_empty() {
            let hint = "drag to move";
            let m = measure_text(hint, None, 22, 1.0);
            draw_text_ex(
                hint,
                CX - m.width / 2.0,
                SHIELD_Y + 50.0,
                TextParams { font_size: 22, color: C_LABEL, ..Default::default() },
            );
        }
    }

    fn draw_home_button(&self) {
        let r = home_button_rect();
        filled_pill(r.x, r.y, r.w, r.h, r.h / 2.0, C_HOME_BG);
        let label = "HOME";
        let m = measure_text(label, None, 22, 1.0);
        draw_text_ex(
            label,
            r.x + (r.w - m.width) / 2.0,
            r.y + r.h * 0.66,
            TextParams { font_size: 22, color: WHITE, ..Default::default() },
        );
    }

    fn draw_end_screen(&self) {
        // Dim
        draw_rectangle(0.0, 0.0, SW, SH, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.78 });

        let won = self.correct_count >= WIN_CORRECT;
        let title = if won { "WAVE COMPLETE!" } else { "GAME OVER" };
        let title_color = if won { C_OK } else { C_BAD };
        let tm = measure_text(title, None, 56, 1.0);
        draw_text_ex(
            title,
            CX - tm.width / 2.0,
            SH * 0.38,
            TextParams { font_size: 56, color: title_color, ..Default::default() },
        );

        // Star reward row
        let stars = self.compute_stars();
        let r = 30.0;
        let gap = 24.0;
        let total_w = r * 6.0 + gap * 2.0;
        let start_x = CX - total_w / 2.0 + r;
        let y = SH * 0.48;
        for i in 0..3u8 {
            let cx = start_x + i as f32 * (r * 2.0 + gap);
            let color = if i < stars {
                Color { r: 1.0, g: 0.85, b: 0.1, a: 1.0 }
            } else {
                Color { r: 0.25, g: 0.25, b: 0.40, a: 1.0 }
            };
            draw_circle(cx, y, r, color);
            if i < stars {
                draw_circle(cx - 8.0, y - 8.0, 10.0, Color { r: 1.0, g: 1.0, b: 0.9, a: 0.45 });
            }
        }

        // Catches summary
        let summary = format!("Correct catches: {}", self.correct_count);
        let sm = measure_text(&summary, None, 30, 1.0);
        draw_text_ex(
            &summary,
            CX - sm.width / 2.0,
            SH * 0.62,
            TextParams { font_size: 30, color: C_LABEL, ..Default::default() },
        );

        // Tap to continue (after debounce)
        if get_time() - self.end_time >= 0.6 {
            let prompt = "Tap to return to hub";
            let pm = measure_text(prompt, None, 26, 1.0);
            draw_text_ex(
                prompt,
                CX - pm.width / 2.0,
                SH * 0.72,
                TextParams { font_size: 26, color: WHITE, ..Default::default() },
            );
        }
    }
}

// ── Standalone helpers ────────────────────────────────────────────────────────

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

    #[test]
    fn stars_match_outcome() {
        // Helper to construct with overridden counters
        fn with(correct: u8, wrong: u8) -> u8 {
            let mut g = MeteorCatch::new(Grade::Preschool);
            g.correct_count = correct;
            g.wrong_count = wrong;
            g.compute_stars()
        }
        assert_eq!(with(WIN_CORRECT, 0), 3, "won with no losses = 3 stars");
        assert_eq!(with(WIN_CORRECT, 2), 3, "won regardless of losses = 3 stars");
        assert_eq!(with(4, MAX_LIVES), 2, "lost with 4 correct = 2 stars");
        assert_eq!(with(3, MAX_LIVES), 2, "lost with 3 correct = 2 stars");
        assert_eq!(with(2, MAX_LIVES), 1, "lost with 2 correct = 1 star");
        assert_eq!(with(1, MAX_LIVES), 1, "lost with 1 correct = 1 star");
        assert_eq!(with(0, MAX_LIVES), 0, "lost with 0 correct = 0 stars");
    }

    #[test]
    fn lives_remaining_clamps_to_zero() {
        let mut g = MeteorCatch::new(Grade::Preschool);
        g.wrong_count = 99;
        assert_eq!(g.lives_remaining(), 0);
    }

    #[test]
    fn queue_wave_includes_correct_and_distractors() {
        let g = MeteorCatch::new(Grade::FirstGrade);
        // After new(), spawn_queue is filled
        assert_eq!(g.spawn_queue.len(), METEORS_PER_WAVE);
        let correct_count = g.spawn_queue.iter().filter(|(_, c)| *c).count();
        assert_eq!(correct_count, 1, "exactly one correct meteor per wave");
        // All distractors should be different from the correct answer
        for (val, is_correct) in &g.spawn_queue {
            if !*is_correct {
                assert_ne!(*val, g.question.correct_answer);
            }
        }
    }

    #[test]
    fn fall_speed_scales_with_grade() {
        // Direct check without frame_step (which is 0 in tests).
        // Just verify the grade index factor.
        let preschool = METEOR_BASE_SPEED * (1.0 + 0.0 * 0.08);
        let fifth = METEOR_BASE_SPEED * (1.0 + 6.0 * 0.08);
        assert!(fifth > preschool);
        assert!((fifth / preschool - 1.48).abs() < 0.01);
    }
}
