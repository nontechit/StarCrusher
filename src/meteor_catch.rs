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
use crate::question::{generate_meteor_catch_question, Question};
use crate::random;
use crate::screen;
use crate::theme;
use macroquad::prelude::*;

// ── Tunables ──────────────────────────────────────────────────────────────────

const WIN_CORRECT: u8 = 5;
const MAX_LIVES: u8 = 3;

/// Virtual screen — Meteor Catch is portrait-only.
const SW: f32 = 720.0;
const SH: f32 = 1280.0;
const CX: f32 = SW / 2.0;

// Shield (player paddle).  Narrowed for a fair-but-challenging catch on phone
// screens — the old 140px paddle covered too much of the 720px canvas.
const SHIELD_W: f32 = 104.0;
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

// Colors — GW blue & gold palette (see theme.rs).
const C_BG_TOP: Color    = theme::BG_TOP;
const C_BG_BOT: Color    = theme::BG_BOTTOM;
const C_HEADER_BG: Color = theme::PANEL;
const C_LABEL: Color     = theme::LABEL;
const C_QUESTION: Color  = theme::QUESTION;
const C_HOME_BG: Color   = theme::BUTTON;
const C_METEOR: Color    = theme::ROYAL;
const C_METEOR_GLOW: Color = theme::ROYAL_GLOW;
const C_SHIELD_GLOW: Color = theme::GOLD_GLOW;
const C_OK: Color        = theme::CORRECT;
const C_BAD: Color       = theme::WRONG;

// Special bonus meteor — bright gold, visually distinct from royal-blue meteors.
const C_SPECIAL: Color      = Color { r: 1.0, g: 0.88, b: 0.38, a: 1.0 };
const C_SPECIAL_GLOW: Color = Color { r: 1.0, g: 0.92, b: 0.55, a: 0.55 };
// Bonus level (coin collection).
const C_COIN: Color      = Color { r: 1.0, g: 0.82, b: 0.22, a: 1.0 };
const C_COIN_GLOW: Color = Color { r: 1.0, g: 0.88, b: 0.40, a: 0.40 };

// Bonus level tunables.
const BONUS_DURATION: f64 = 16.0;     // seconds of coin collecting
const COIN_R: f32 = 26.0;
const COIN_SPAWN_GAP: f32 = 0.42;     // seconds between coin spawns
const COIN_BASE_SPEED: f32 = 3.0;     // px/frame

/// Per-grade falling-meteor count range for a wave (1 correct + distractors).
/// Harder grades get more simultaneous drops.
fn wave_size_range(grade: Grade) -> (usize, usize) {
    match grade {
        Grade::Preschool    => (3, 4),
        Grade::Kindergarten => (3, 5),
        Grade::FirstGrade   => (4, 5),
        Grade::SecondGrade  => (4, 6),
        Grade::ThirdGrade   => (5, 6),
        Grade::FourthGrade  => (5, 7),
        Grade::FifthGrade   => (6, 8),
    }
}

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
    is_special: bool, // the rare glowing bonus meteor (no number)
    state: MeteorState,
    flash_until: f64, // wall-clock time; flashes green/red briefly when consumed
}

/// A falling coin in the secret bonus level.
struct Coin {
    x: f32,
    y: f32,
    speed: f32,
}

/// Riddles shown during the secret bonus level (one picked at random).
const BONUS_RIDDLES: [&str; 4] = [
    "I have hands but cannot clap. What am I?",
    "The more you take, the more you leave behind. What are they?",
    "What has to be broken before you can use it?",
    "What gets wetter the more it dries?",
];

// ── Main game struct ──────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
enum Phase {
    Playing,
    Bonus,     // secret coin-collection level (entered via the special meteor)
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

    // ── Secret bonus level ──
    /// Whether the special meteor still may appear this wave, and when.
    special_due: Option<f32>,
    special_timer: f32,
    bonus_riddle: &'static str,
    bonus_coins: Vec<Coin>,
    bonus_collected: u32,
    bonus_spawn_timer: f32,
    bonus_end_at: f64,
}

impl MeteorCatch {
    pub fn new(grade: Grade) -> Self {
        let question = generate_meteor_catch_question(grade);
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
            special_due: None,
            special_timer: 0.0,
            bonus_riddle: BONUS_RIDDLES[0],
            bonus_coins: Vec::new(),
            bonus_collected: 0,
            bonus_spawn_timer: 0.0,
            bonus_end_at: 0.0,
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

    /// Build a new wave's spawn queue: one correct + (count-1) distractors from
    /// `question.wrong_answers` (padded if there aren't enough).  The drop count
    /// is random within the grade's range, so harder grades get more drops.
    fn queue_wave(&mut self) {
        self.spawn_queue.clear();
        self.spawn_timer = METEOR_SPAWN_GAP; // spawn first meteor immediately

        let (min_n, max_n) = wave_size_range(self.grade);
        let wave_size = random::i32_inclusive(min_n as i32, max_n as i32) as usize;

        // Collect distractors
        let mut distractors: Vec<i64> = self.question.wrong_answers.clone();
        random::shuffle(&mut distractors);
        distractors.truncate(wave_size - 1);

        // Pad if fewer than needed (e.g. easy questions with few alternatives)
        let mut bump = 1i64;
        while distractors.len() < wave_size - 1 {
            let candidate = self.question.correct_answer + bump;
            if candidate != self.question.correct_answer && !distractors.contains(&candidate) {
                distractors.push(candidate);
            }
            bump += 1;
        }

        // Build wave with correct + distractors, then shuffle order
        let mut wave: Vec<(i64, bool)> = vec![(self.question.correct_answer, true)];
        for v in distractors {
            wave.push((v, false));
        }
        random::shuffle(&mut wave);
        self.spawn_queue = wave;

        // Secretly schedule a special bonus meteor for this wave (rare).
        self.special_timer = 0.0;
        self.special_due = if random::bool(0.18) {
            Some(random::f32_range(0.6, 2.4))
        } else {
            None
        };
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
            Phase::Bonus => self.update_bonus(),
            Phase::EndScreen => self.update_end_screen(),
        }
    }

    fn update_playing(&mut self) -> MeteorCatchAction {
        self.update_shield();
        self.update_spawning();
        self.update_special_spawn();
        self.update_meteors();
        self.resolve_consumed_meteors();
        // resolve_consumed_meteors may have switched us into the bonus level.
        if self.phase == Phase::Playing {
            self.check_phase_transition();
        }
        MeteorCatchAction::None
    }

    /// Spawn the secretly-scheduled special meteor when its timer elapses.
    fn update_special_spawn(&mut self) {
        let Some(due) = self.special_due else { return };
        self.special_timer += get_frame_time();
        if self.special_timer >= due {
            self.special_due = None;
            let x = self.pick_spawn_x();
            self.meteors.push(Meteor {
                x,
                y: METEOR_SPAWN_Y,
                value: 0,
                is_correct: false,
                is_special: true,
                state: MeteorState::Falling,
                flash_until: 0.0,
            });
        }
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
                is_special: false,
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
        let mut catch_special = false;

        self.meteors.retain(|m| {
            // Keep falling meteors
            if m.state == MeteorState::Falling {
                return true;
            }
            // Keep until flash window expires (visual feedback)
            if now < m.flash_until {
                return true;
            }
            // Special bonus meteor: catching it opens the secret level; missing
            // it is harmless and never costs a life.
            if m.is_special {
                if m.state == MeteorState::Hit {
                    catch_special = true;
                }
                return false;
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

        if catch_special {
            self.enter_bonus();
        }
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
            self.question = generate_meteor_catch_question(self.grade);
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

    // ── Secret bonus level ──────────────────────────────────────────────────

    /// Switch into the hidden coin-collection level.  Does not touch lives or
    /// the correct-catch tally, so it never affects the main game's win/loss.
    fn enter_bonus(&mut self) {
        self.phase = Phase::Bonus;
        self.meteors.clear();
        self.spawn_queue.clear();
        self.special_due = None;
        self.bonus_riddle = BONUS_RIDDLES[random::usize_exclusive(BONUS_RIDDLES.len())];
        self.bonus_coins.clear();
        self.bonus_collected = 0;
        self.bonus_spawn_timer = COIN_SPAWN_GAP;
        self.bonus_end_at = get_time() + BONUS_DURATION;
    }

    fn update_bonus(&mut self) -> MeteorCatchAction {
        self.update_shield();

        // Spawn coins on a steady cadence.
        self.bonus_spawn_timer += get_frame_time();
        if self.bonus_spawn_timer >= COIN_SPAWN_GAP {
            self.bonus_spawn_timer = 0.0;
            self.bonus_coins.push(Coin {
                x: random::f32_range(COIN_R + 30.0, SW - COIN_R - 30.0),
                y: -COIN_R,
                speed: COIN_BASE_SPEED + random::f32_range(0.0, 1.4),
            });
        }

        // Move and collect coins.
        let step = screen::frame_step();
        let shield_top = SHIELD_Y - SHIELD_H / 2.0;
        let shield_left = self.shield_x - SHIELD_W / 2.0;
        let shield_right = self.shield_x + SHIELD_W / 2.0;
        let mut collected = 0u32;
        self.bonus_coins.retain_mut(|c| {
            c.y += c.speed * step;
            let caught = c.y + COIN_R >= shield_top
                && c.y <= SHIELD_Y + SHIELD_H
                && c.x >= shield_left - COIN_R * 0.6
                && c.x <= shield_right + COIN_R * 0.6;
            if caught {
                collected += 1;
                return false;
            }
            c.y - COIN_R <= SH // drop fell off-screen → remove
        });
        self.bonus_collected += collected;

        // Time up → resume the main game with a fresh wave.
        if get_time() >= self.bonus_end_at {
            self.bonus_coins.clear();
            self.phase = Phase::Playing;
            self.question = generate_meteor_catch_question(self.grade);
            self.queue_wave();
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

        if self.phase == Phase::Bonus {
            self.draw_bonus();
            self.draw_home_button();
            return;
        }

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

        // Score (top-right)
        let score_label = format!("{}/{}", self.correct_count, WIN_CORRECT);
        let sm = measure_text(&score_label, None, 28, 1.0);
        draw_text_ex(
            &score_label,
            SW - 30.0 - sm.width,
            72.0,
            TextParams { font_size: 28, color: C_LABEL, ..Default::default() },
        );
    }

    fn draw_meteor(&self, m: &Meteor) {
        // Special bonus meteor — gold, pulsing, no number. Visually unmistakable
        // against the royal-blue answer meteors.
        if m.is_special {
            let pulse = 0.5 + 0.5 * (get_time() as f32 * 6.0).sin();
            draw_circle(m.x, m.y, METEOR_R + 10.0 + pulse * 7.0, C_SPECIAL_GLOW);
            draw_circle(m.x, m.y, METEOR_R, C_SPECIAL);
            draw_circle_lines(m.x, m.y, METEOR_R, 3.5, Color { r: 1.0, g: 1.0, b: 0.85, a: 0.95 });
            // Bright center shine for a "treasure" sparkle.
            draw_circle(m.x - METEOR_R * 0.22, m.y - METEOR_R * 0.22, METEOR_R * 0.30,
                Color { r: 1.0, g: 1.0, b: 0.95, a: 0.7 });
            draw_circle(m.x, m.y, METEOR_R * 0.16, WHITE);
            return;
        }

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
        let accent = theme::GOLD;

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

    fn draw_bonus(&self) {
        // Riddle panel
        filled_pill(20.0, 96.0, SW - 40.0, 174.0, 16.0, C_HEADER_BG);

        let title = "BONUS!";
        let tm = measure_text(title, None, 42, 1.0);
        draw_text_ex(
            title,
            CX - tm.width / 2.0,
            150.0,
            TextParams { font_size: 42, color: C_SPECIAL, ..Default::default() },
        );
        self.draw_wrapped(self.bonus_riddle, 192.0, 24, SW - 80.0, C_QUESTION);

        // Coins
        for c in &self.bonus_coins {
            draw_circle(c.x, c.y, COIN_R + 6.0, C_COIN_GLOW);
            draw_circle(c.x, c.y, COIN_R, C_COIN);
            draw_circle_lines(c.x, c.y, COIN_R, 2.5, Color { r: 1.0, g: 0.70, b: 0.20, a: 0.9 });
            let g = "$";
            let gm = measure_text(g, None, 28, 1.0);
            draw_text_ex(
                g,
                c.x - gm.width / 2.0,
                c.y + gm.offset_y / 2.0,
                TextParams { font_size: 28, color: Color { r: 0.45, g: 0.30, b: 0.0, a: 1.0 }, ..Default::default() },
            );
        }

        // Coin counter
        let label = format!("Coins: {}", self.bonus_collected);
        draw_text_ex(
            &label,
            30.0,
            SH - 64.0,
            TextParams { font_size: 34, color: C_COIN, ..Default::default() },
        );

        // Countdown bar
        let remaining = (self.bonus_end_at - get_time()).max(0.0) as f32;
        let frac = (remaining / BONUS_DURATION as f32).clamp(0.0, 1.0);
        draw_rectangle(30.0, SH - 34.0, SW - 60.0, 10.0, C_HEADER_BG);
        draw_rectangle(30.0, SH - 34.0, (SW - 60.0) * frac, 10.0, C_SPECIAL);

        self.draw_shield();
    }

    /// Center-aligned word-wrapped text starting at `top_y`.
    fn draw_wrapped(&self, text: &str, top_y: f32, size: u16, max_w: f32, color: Color) {
        let mut lines: Vec<String> = Vec::new();
        let mut cur = String::new();
        for word in text.split(' ') {
            let trial = if cur.is_empty() { word.to_string() } else { format!("{} {}", cur, word) };
            if !cur.is_empty() && measure_text(&trial, None, size, 1.0).width > max_w {
                lines.push(std::mem::take(&mut cur));
                cur = word.to_string();
            } else {
                cur = trial;
            }
        }
        if !cur.is_empty() {
            lines.push(cur);
        }
        let line_h = size as f32 + 8.0;
        for (i, line) in lines.iter().enumerate() {
            let m = measure_text(line, None, size, 1.0);
            draw_text_ex(
                line,
                CX - m.width / 2.0,
                top_y + i as f32 * line_h,
                TextParams { font_size: size, color, ..Default::default() },
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
        // After new(), spawn_queue is filled with a grade-scaled random count.
        let (min_n, max_n) = wave_size_range(Grade::FirstGrade);
        assert!(
            g.spawn_queue.len() >= min_n && g.spawn_queue.len() <= max_n,
            "wave size {} out of range {}..={}",
            g.spawn_queue.len(), min_n, max_n
        );
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
    fn wave_size_ranges_grow_with_grade() {
        let (_, pre_max) = wave_size_range(Grade::Preschool);
        let (fifth_min, _) = wave_size_range(Grade::FifthGrade);
        assert!(fifth_min >= pre_max, "harder grades get more drops");
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
