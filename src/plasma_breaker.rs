// Star Academy game #3: Plasma Breaker
//
// Breakout-style math game.  A plasma ball bounces around the screen;
// the player drags a paddle to keep it in play.  Numbered blocks fill
// the upper area — break the block whose value matches the math question.
//
// Break correct block → +1 correct count, full block grid respawns with
//                        a new question.
// Break wrong block   → block removed, no penalty.
// Ball falls past paddle → -1 life, ball resets above paddle.
//
// Win : WIN_CORRECT correct blocks broken → 3 stars.
// Lose: lives reach 0                     → partial stars.
//
// Stars: win→3, lose with ≥3→2, lose with 1-2→1, 0→0.

use crate::levels::Grade;
use crate::question::{generate_question, Question};
use crate::random;
use crate::screen;
use macroquad::prelude::*;

// ── Tunables ──────────────────────────────────────────────────────────────────

pub const WIN_CORRECT: u8 = 5;
pub const MAX_LIVES:   u8 = 3;

const SW: f32 = 720.0;
const SH: f32 = 1280.0;
const CX: f32 = SW / 2.0;

// Block grid
const COLS: usize = 5;
const ROWS: usize = 3;
const BLOCK_COUNT: usize = COLS * ROWS; // 15
const BLOCK_W: f32 = 126.0;
const BLOCK_H: f32 = 48.0;
const BLOCK_GAP: f32 = 8.0;
// Total grid width: COLS*(BLOCK_W+BLOCK_GAP) - BLOCK_GAP = 5*134 - 8 = 662
// Centre in SW=720 → left edge = (720-662)/2 = 29
const BLOCK_LEFT: f32 = 29.0;
const BLOCK_TOP:  f32 = 252.0;

// Ball
const BALL_R: f32 = 15.0;
const BALL_BASE_SPEED: f32 = 5.6; // px/frame at grade 0

// Walls
const TOP_WALL:  f32 = 228.0;  // bounce ceiling (below header)
const LEFT_WALL: f32 = 0.0;
const RIGHT_WALL: f32 = SW;

// Paddle
const PAD_W:  f32 = 190.0;
const PAD_H:  f32 = 26.0;
const PAD_Y:  f32 = 1110.0; // centre-y
const PAD_X_MIN: f32 = PAD_W / 2.0 + 10.0;
const PAD_X_MAX: f32 = SW - PAD_W / 2.0 - 10.0;

// Header
const HOME_X: f32 =  24.0;
const HOME_Y: f32 =  24.0;
const HOME_W: f32 = 110.0;
const HOME_H: f32 =  56.0;

// Colors
const C_BG:        Color = Color { r: 0.04, g: 0.02, b: 0.14, a: 1.0 };
const C_HEADER_BG: Color = Color { r: 0.07, g: 0.07, b: 0.18, a: 0.93 };
const C_HOME_BG:   Color = Color { r: 0.16, g: 0.16, b: 0.28, a: 0.95 };
const C_BALL:      Color = Color { r: 0.98, g: 0.92, b: 0.30, a: 1.0 };
const C_BALL_GLOW: Color = Color { r: 1.00, g: 0.95, b: 0.40, a: 0.22 };
const C_PAD:       Color = Color { r: 0.30, g: 0.80, b: 1.00, a: 1.0 };
const C_PAD_GLOW:  Color = Color { r: 0.30, g: 0.80, b: 1.00, a: 0.18 };
const C_CORRECT:   Color = Color { r: 0.18, g: 0.85, b: 0.42, a: 1.0 };
const C_WRONG:     Color = Color { r: 0.90, g: 0.25, b: 0.25, a: 1.0 };
const C_LABEL:     Color = Color { r: 0.60, g: 0.60, b: 0.78, a: 1.0 };
const C_QUESTION:  Color = Color { r: 1.00, g: 0.92, b: 0.45, a: 1.0 };
const C_HEART_ON:  Color = Color { r: 1.00, g: 0.32, b: 0.45, a: 1.0 };
const C_HEART_OFF: Color = Color { r: 0.25, g: 0.18, b: 0.25, a: 1.0 };

// ── Public action ─────────────────────────────────────────────────────────────

pub enum PlasmaAction {
    None,
    ExitToHub,
    Completed { stars: u8 },
}

// ── Block ─────────────────────────────────────────────────────────────────────

struct Block {
    col: usize,
    row: usize,
    value: i64,
    is_correct: bool,
    alive: bool,
    flash_until: f64, // >0 while hit-flash is playing
    flash_correct: bool,
}

impl Block {
    fn rect(&self) -> (f32, f32, f32, f32) {
        let x = BLOCK_LEFT + self.col as f32 * (BLOCK_W + BLOCK_GAP);
        let y = BLOCK_TOP  + self.row as f32 * (BLOCK_H + BLOCK_GAP);
        (x, y, BLOCK_W, BLOCK_H)
    }

    fn center(&self) -> (f32, f32) {
        let (x, y, w, h) = self.rect();
        (x + w / 2.0, y + h / 2.0)
    }
}

// ── Ball ──────────────────────────────────────────────────────────────────────

struct Ball {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    launched: bool,
}

impl Ball {
    fn above_paddle(pad_x: f32) -> Self {
        Self {
            x: pad_x,
            y: PAD_Y - PAD_H / 2.0 - BALL_R - 3.0,
            vx: 0.0,
            vy: 0.0,
            launched: false,
        }
    }

    fn launch(&mut self, speed: f32) {
        // Slight random angle left or right of straight up.
        let angle_deg = random::f32_range(-35.0, 35.0);
        let angle = angle_deg.to_radians();
        self.vx = speed * angle.sin();
        self.vy = -speed * angle.cos(); // negative = upward
        self.launched = true;
    }

    /// Move the ball one frame.  Returns true if the ball dropped past the paddle.
    fn step(&mut self, pad_x: f32, step: f32) -> bool {
        if !self.launched {
            // Sit on paddle, follow its x.
            self.x = pad_x;
            self.y = PAD_Y - PAD_H / 2.0 - BALL_R - 3.0;
            return false;
        }

        self.x += self.vx * step;
        self.y += self.vy * step;

        // Left / right walls
        if self.x - BALL_R < LEFT_WALL {
            self.x = LEFT_WALL + BALL_R;
            self.vx = self.vx.abs();
        }
        if self.x + BALL_R > RIGHT_WALL {
            self.x = RIGHT_WALL - BALL_R;
            self.vx = -self.vx.abs();
        }

        // Top wall
        if self.y - BALL_R < TOP_WALL {
            self.y = TOP_WALL + BALL_R;
            self.vy = self.vy.abs();
        }

        // Paddle bounce
        let pad_left  = pad_x - PAD_W / 2.0;
        let pad_right = pad_x + PAD_W / 2.0;
        let pad_top   = PAD_Y - PAD_H / 2.0;
        if self.vy > 0.0
            && self.y + BALL_R >= pad_top
            && self.y          <= PAD_Y + PAD_H / 2.0
            && self.x          >= pad_left  - BALL_R * 0.7
            && self.x          <= pad_right + BALL_R * 0.7
        {
            self.y = pad_top - BALL_R;
            // Spin: where ball hits paddle (-1 = far left, +1 = far right)
            let hit = ((self.x - pad_x) / (PAD_W / 2.0)).clamp(-1.0, 1.0);
            let speed = (self.vx * self.vx + self.vy * self.vy).sqrt();
            self.vx = hit * speed * 0.75;
            self.vy = -(speed * speed - self.vx * self.vx).sqrt().max(speed * 0.45);
        }

        // Dropped past paddle
        self.y - BALL_R > PAD_Y + PAD_H
    }
}

// ── Main struct ───────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq)]
enum Phase {
    Ready,      // ball on paddle, waiting for launch tap
    Playing,
    EndScreen,
}

pub struct PlasmaBreaker {
    grade: Grade,
    question: Question,
    blocks: Vec<Block>,
    ball: Ball,
    pad_x: f32,
    correct_count: u8,
    wrong_count: u8,
    phase: Phase,
    end_time: f64,
}

impl PlasmaBreaker {
    pub fn new(grade: Grade) -> Self {
        let question = generate_question(grade);
        let pad_x = CX;
        let blocks = Self::build_blocks(&question);
        Self {
            grade,
            question,
            blocks,
            ball: Ball::above_paddle(pad_x),
            pad_x,
            correct_count: 0,
            wrong_count: 0,
            phase: Phase::Ready,
            end_time: 0.0,
        }
    }

    fn lives_remaining(&self) -> u8 {
        MAX_LIVES.saturating_sub(self.wrong_count)
    }

    fn ball_speed(&self) -> f32 {
        let g = self.grade.index() as f32; // 0..=6
        BALL_BASE_SPEED + g * 0.28
    }

    /// Build a fresh 5×3 grid from the current question.
    fn build_blocks(question: &Question) -> Vec<Block> {
        // Gather distractors
        let mut wrongs: Vec<i64> = question.wrong_answers.clone();
        random::shuffle(&mut wrongs);

        // Pad with varied distractors.  Use a generous ±30 range and a
        // bounded attempt counter so we never loop forever (grades like
        // Preschool only supply 3 wrong_answers but we need 14 total).
        let mut budget = 500u32;
        while wrongs.len() < BLOCK_COUNT - 1 && budget > 0 {
            let delta = random::i32_inclusive(1, 30) as i64
                * if random::bool(0.5) { 1 } else { -1 };
            let v = question.correct_answer + delta;
            if !wrongs.contains(&v) && v != question.correct_answer {
                wrongs.push(v);
            }
            budget -= 1;
        }
        // Guaranteed-unique fallback: sequential offsets well away from the
        // correct answer so they are never mistaken for it.
        let mut seq = 1i64;
        while wrongs.len() < BLOCK_COUNT - 1 {
            let v = question.correct_answer + 100 + seq;
            if !wrongs.contains(&v) {
                wrongs.push(v);
            }
            seq += 1;
        }
        wrongs.truncate(BLOCK_COUNT - 1);

        // Assign values to positions; correct goes in a random slot.
        let correct_slot = random::usize_exclusive(BLOCK_COUNT);
        let mut distractor_iter = wrongs.into_iter();

        let mut blocks = Vec::with_capacity(BLOCK_COUNT);
        for row in 0..ROWS {
            for col in 0..COLS {
                let idx = row * COLS + col;
                let is_correct = idx == correct_slot;
                let value = if is_correct {
                    question.correct_answer
                } else {
                    distractor_iter.next().unwrap_or(question.correct_answer + 1)
                };
                blocks.push(Block { col, row, value, is_correct, alive: true,
                    flash_until: 0.0, flash_correct: false });
            }
        }
        blocks
    }

    /// Rebuild block values in place (keeps alive/dead state, only changes values).
    fn refresh_blocks(&mut self) {
        let new_correct = random::usize_exclusive(BLOCK_COUNT);
        let mut wrongs: Vec<i64> = self.question.wrong_answers.clone();
        random::shuffle(&mut wrongs);
        let mut budget = 500u32;
        while wrongs.len() < BLOCK_COUNT - 1 && budget > 0 {
            let delta = random::i32_inclusive(1, 30) as i64
                * if random::bool(0.5) { 1 } else { -1 };
            let v = self.question.correct_answer + delta;
            if !wrongs.contains(&v) && v != self.question.correct_answer {
                wrongs.push(v);
            }
            budget -= 1;
        }
        let mut seq = 1i64;
        while wrongs.len() < BLOCK_COUNT - 1 {
            let v = self.question.correct_answer + 100 + seq;
            if !wrongs.contains(&v) {
                wrongs.push(v);
            }
            seq += 1;
        }
        wrongs.truncate(BLOCK_COUNT - 1);

        let mut di = wrongs.into_iter();
        for (i, b) in self.blocks.iter_mut().enumerate() {
            b.alive = true; // respawn all blocks
            b.is_correct = i == new_correct;
            b.value = if b.is_correct {
                self.question.correct_answer
            } else {
                di.next().unwrap_or(self.question.correct_answer + 1)
            };
        }
    }

    // ── Update ────────────────────────────────────────────────────────────────

    pub fn update(&mut self) -> PlasmaAction {
        if is_key_pressed(KeyCode::Escape) {
            return PlasmaAction::ExitToHub;
        }
        if let Some(tap) = screen::primary_tap_position() {
            if home_rect().contains(tap) {
                return PlasmaAction::ExitToHub;
            }
        }

        match self.phase {
            Phase::Ready    => self.update_ready(),
            Phase::Playing  => self.update_playing(),
            Phase::EndScreen => self.update_end_screen(),
        }
    }

    fn update_ready(&mut self) -> PlasmaAction {
        self.move_paddle();
        // Launch on tap (not HOME) or Space/Enter
        let tapped = screen::primary_tap_position()
            .map(|t| !home_rect().contains(t))
            .unwrap_or(false);
        if tapped || is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Enter) {
            self.ball.launch(self.ball_speed());
            self.phase = Phase::Playing;
        }
        PlasmaAction::None
    }

    fn update_playing(&mut self) -> PlasmaAction {
        self.move_paddle();

        let dropped = self.ball.step(self.pad_x, screen::frame_step());
        if dropped {
            self.wrong_count = self.wrong_count.saturating_add(1);
            self.ball = Ball::above_paddle(self.pad_x);
            self.phase = Phase::Ready;
        } else {
            self.check_block_collisions();
            self.expire_block_flashes();
        }

        self.check_transition();
        PlasmaAction::None
    }

    fn move_paddle(&mut self) {
        if let Some(p) = screen::primary_pointer_position() {
            self.pad_x = p.x.clamp(PAD_X_MIN, PAD_X_MAX);
        }
        // Keyboard
        let step = 12.0 * screen::frame_step();
        if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
            self.pad_x = (self.pad_x - step).max(PAD_X_MIN);
        }
        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
            self.pad_x = (self.pad_x + step).min(PAD_X_MAX);
        }
    }

    fn check_block_collisions(&mut self) {
        let bx = self.ball.x;
        let by = self.ball.y;
        let now = get_time();

        let mut hit_correct = false;
        let mut hit_wrong   = false;

        for block in &mut self.blocks {
            if !block.alive || block.flash_until > now {
                continue;
            }
            let (rx, ry, rw, rh) = block.rect();

            // Closest point on block AABB to ball center
            let cx = bx.clamp(rx, rx + rw);
            let cy = by.clamp(ry, ry + rh);
            let dx = bx - cx;
            let dy = by - cy;
            if dx * dx + dy * dy > BALL_R * BALL_R {
                continue;
            }

            // Deflect ball
            let bx_center = rx + rw / 2.0;
            let by_center = ry + rh / 2.0;
            let norm_x = ((bx - bx_center) / (rw / 2.0)).abs();
            let norm_y = ((by - by_center) / (rh / 2.0)).abs();
            if norm_x > norm_y {
                self.ball.vx = -self.ball.vx;
            } else {
                self.ball.vy = -self.ball.vy;
            }
            // Push ball out of block
            if self.ball.vy < 0.0 { self.ball.y = ry + rh + BALL_R; }
            else                   { self.ball.y = ry - BALL_R; }

            // Score
            block.flash_until = now + 0.22;
            block.flash_correct = block.is_correct;
            block.alive = false;

            if block.is_correct { hit_correct = true; }
            else                 { hit_wrong   = true; }

            break; // one block per frame keeps physics clean
        }

        if hit_correct {
            self.correct_count = self.correct_count.saturating_add(1);
            if self.correct_count < WIN_CORRECT {
                self.question = generate_question(self.grade);
                self.refresh_blocks();
            }
        }
        let _ = hit_wrong; // wrong blocks just disappear
    }

    fn expire_block_flashes(&mut self) {
        // Blocks are set alive=false on hit; flash_until controls visual.
        // After flash expires they simply stay dead — nothing else needed.
        let now = get_time();
        for b in &mut self.blocks {
            if !b.alive && b.flash_until > 0.0 && now >= b.flash_until {
                b.flash_until = 0.0;
            }
        }
        // If every block is dead (all wrong cleared) respawn the grid.
        if self.blocks.iter().all(|b| !b.alive && b.flash_until == 0.0) {
            self.question = generate_question(self.grade);
            self.refresh_blocks();
        }
    }

    fn check_transition(&mut self) {
        if self.correct_count >= WIN_CORRECT || self.wrong_count >= MAX_LIVES {
            self.phase = Phase::EndScreen;
            self.end_time = get_time();
        }
    }

    fn update_end_screen(&mut self) -> PlasmaAction {
        if get_time() - self.end_time < 0.6 {
            return PlasmaAction::None;
        }
        let tap = screen::primary_tap_position().is_some();
        let key = is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space);
        if tap || key {
            return PlasmaAction::Completed { stars: self.compute_stars() };
        }
        PlasmaAction::None
    }

    pub fn compute_stars(&self) -> u8 {
        if self.correct_count >= WIN_CORRECT { 3 }
        else if self.correct_count >= 3      { 2 }
        else if self.correct_count >= 1      { 1 }
        else                                 { 0 }
    }

    // ── Draw ──────────────────────────────────────────────────────────────────

    pub fn draw(&self) {
        draw_rectangle(0.0, 0.0, SW, SH, C_BG);
        self.draw_grid_lines();
        self.draw_starfield();
        self.draw_header();
        self.draw_blocks();
        self.draw_paddle();
        self.draw_ball();
        self.draw_home_button();
        if self.phase == Phase::Ready {
            self.draw_launch_hint();
        }
        if self.phase == Phase::EndScreen {
            self.draw_end_screen();
        }
    }

    fn draw_grid_lines(&self) {
        // Subtle vertical plasma columns
        for i in 0..8u32 {
            let x = i as f32 * (SW / 7.0);
            draw_rectangle(x, TOP_WALL, 1.0, SH - TOP_WALL, Color { r: 0.35, g: 0.15, b: 0.55, a: 0.12 });
        }
    }

    fn draw_starfield(&self) {
        for i in 0..55 {
            let x = ((i * 97 + 13) % SW as i32) as f32;
            let y = ((i * 61 + 41) % (SH as i32 - 40)) as f32;
            draw_circle(x, y, 1.2, Color { r: 0.8, g: 0.7, b: 1.0, a: 0.30 });
        }
    }

    fn draw_header(&self) {
        filled_pill(20.0, 96.0, SW - 40.0, 110.0, 16.0, C_HEADER_BG);

        // Question
        let q = &self.question.text;
        let qs = fit_text(q, 44, SW - 100.0, 22);
        let m = measure_text(q, None, qs, 1.0);
        draw_text_ex(q, CX - m.width / 2.0, 166.0,
            TextParams { font_size: qs, color: C_QUESTION, ..Default::default() });

        // Score
        let score = format!("{}/{}", self.correct_count, WIN_CORRECT);
        let sm = measure_text(&score, None, 28, 1.0);
        draw_text_ex(&score, SW - 30.0 - sm.width, 72.0,
            TextParams { font_size: 28, color: C_LABEL, ..Default::default() });

        // Hearts
        let hy = 196.0;
        let hr = 9.0;
        let hgap = 26.0;
        let hx = CX - hgap;
        for i in 0..MAX_LIVES {
            let cx = hx + i as f32 * hgap;
            let col = if i < self.lives_remaining() { C_HEART_ON } else { C_HEART_OFF };
            draw_circle(cx - 4.0, hy, hr, col);
            draw_circle(cx + 4.0, hy, hr, col);
            draw_triangle(Vec2::new(cx - 10.0, hy + 3.0), Vec2::new(cx + 10.0, hy + 3.0),
                Vec2::new(cx, hy + 14.0), col);
        }
    }

    fn draw_blocks(&self) {
        let now = get_time();
        let accent = self.grade.enemy_color();

        for b in &self.blocks {
            let (bx, by, bw, bh) = b.rect();
            let flashing = b.flash_until > now;

            if !b.alive && !flashing { continue; }

            let fill = if flashing {
                if b.flash_correct { C_CORRECT } else { C_WRONG }
            } else if b.is_correct {
                // Correct block: subtle accent tint to hint without giving away
                Color { r: accent.r * 0.30 + 0.12, g: accent.g * 0.30 + 0.08,
                        b: accent.b * 0.30 + 0.22, a: 1.0 }
            } else {
                Color { r: 0.14, g: 0.10, b: 0.28, a: 1.0 }
            };

            let border = if b.is_correct && !flashing {
                Color { a: 0.7, ..accent }
            } else {
                Color { r: 0.45, g: 0.30, b: 0.70, a: 0.55 }
            };

            // Fill
            draw_rectangle(bx, by, bw, bh, fill);
            // Border
            draw_rectangle_lines(bx, by, bw, bh, 2.5, border);

            // Value text
            let text = b.value.to_string();
            let ts = fit_text(&text, 28, bw - 10.0, 14);
            let tm = measure_text(&text, None, ts, 1.0);
            let text_col = if flashing { WHITE }
                           else if b.is_correct { WHITE }
                           else { Color { r: 0.82, g: 0.78, b: 0.95, a: 1.0 } };
            draw_text_ex(&text, bx + (bw - tm.width) / 2.0, by + bh * 0.68,
                TextParams { font_size: ts, color: text_col, ..Default::default() });
        }
    }

    fn draw_paddle(&self) {
        let px = self.pad_x - PAD_W / 2.0;
        let py = PAD_Y - PAD_H / 2.0;

        // Glow
        filled_pill(px - 5.0, py - 5.0, PAD_W + 10.0, PAD_H + 10.0,
            (PAD_H + 10.0) / 2.0, C_PAD_GLOW);
        // Body
        filled_pill(px, py, PAD_W, PAD_H, PAD_H / 2.0, C_PAD);
        // Highlight
        filled_pill(px + 8.0, py + 4.0, PAD_W - 16.0, 6.0, 3.0,
            Color { r: 1.0, g: 1.0, b: 1.0, a: 0.30 });
    }

    fn draw_ball(&self) {
        let bx = self.ball.x;
        let by = self.ball.y;

        // Motion trail (small circles behind ball)
        if self.ball.launched {
            let trail_dx = -self.ball.vx * 0.6;
            let trail_dy = -self.ball.vy * 0.6;
            for i in 1..=4u8 {
                let t = i as f32;
                let alpha = 0.20 - t * 0.04;
                draw_circle(bx + trail_dx * t, by + trail_dy * t, BALL_R * (1.0 - t * 0.15),
                    Color { r: C_BALL.r, g: C_BALL.g, b: C_BALL.b, a: alpha.max(0.0) });
            }
        }

        // Glow
        draw_circle(bx, by, BALL_R + 9.0, C_BALL_GLOW);
        // Body
        draw_circle(bx, by, BALL_R, C_BALL);
        // Shine
        draw_circle(bx - BALL_R * 0.30, by - BALL_R * 0.30, BALL_R * 0.22,
            Color { r: 1.0, g: 1.0, b: 1.0, a: 0.45 });
    }

    fn draw_home_button(&self) {
        let r = home_rect();
        filled_pill(r.x, r.y, r.w, r.h, r.h / 2.0, C_HOME_BG);
        let m = measure_text("HOME", None, 22, 1.0);
        draw_text_ex("HOME", r.x + (r.w - m.width) / 2.0, r.y + r.h * 0.66,
            TextParams { font_size: 22, color: WHITE, ..Default::default() });
    }

    fn draw_launch_hint(&self) {
        let label = "Tap to launch!";
        let m = measure_text(label, None, 26, 1.0);
        draw_text_ex(label, CX - m.width / 2.0, PAD_Y + 56.0,
            TextParams { font_size: 26, color: C_LABEL, ..Default::default() });
    }

    fn draw_end_screen(&self) {
        draw_rectangle(0.0, 0.0, SW, SH, Color { r: 0.0, g: 0.0, b: 0.0, a: 0.80 });

        let won = self.correct_count >= WIN_CORRECT;
        let title = if won { "WAVE COMPLETE!" } else { "GAME OVER" };
        let col   = if won { C_CORRECT } else { C_WRONG };
        let tm = measure_text(title, None, 56, 1.0);
        draw_text_ex(title, CX - tm.width / 2.0, SH * 0.38,
            TextParams { font_size: 56, color: col, ..Default::default() });

        let stars = self.compute_stars();
        let r = 30.0;
        let gap = 24.0;
        let start_x = CX - (r * 3.0 + gap);
        for i in 0..3u8 {
            let cx = start_x + i as f32 * (r * 2.0 + gap) + r;
            let sc = if i < stars {
                Color { r: 1.0, g: 0.85, b: 0.10, a: 1.0 }
            } else {
                Color { r: 0.25, g: 0.25, b: 0.40, a: 1.0 }
            };
            draw_circle(cx, SH * 0.49, r, sc);
            if i < stars {
                draw_circle(cx - r * 0.28, SH * 0.49 - r * 0.28, r * 0.22,
                    Color { r: 1.0, g: 1.0, b: 0.9, a: 0.45 });
            }
        }

        let summary = format!("Correct blocks: {}/{}", self.correct_count, WIN_CORRECT);
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

fn home_rect() -> Rect {
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

    fn make(correct: u8, wrong: u8) -> PlasmaBreaker {
        let mut g = PlasmaBreaker::new(Grade::Preschool);
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
        assert_eq!(make(1, MAX_LIVES).compute_stars(), 1);
        assert_eq!(make(0, MAX_LIVES).compute_stars(), 0);
    }

    #[test]
    fn block_grid_has_exactly_one_correct() {
        let g = PlasmaBreaker::new(Grade::SecondGrade);
        let n = g.blocks.iter().filter(|b| b.is_correct).count();
        assert_eq!(n, 1);
        assert_eq!(g.blocks.len(), BLOCK_COUNT);
    }

    #[test]
    fn correct_block_value_matches_question() {
        let g = PlasmaBreaker::new(Grade::ThirdGrade);
        let correct_block = g.blocks.iter().find(|b| b.is_correct).unwrap();
        assert_eq!(correct_block.value, g.question.correct_answer);
    }

    #[test]
    fn ball_starts_unlaunched() {
        let g = PlasmaBreaker::new(Grade::Preschool);
        assert!(!g.ball.launched);
        assert_eq!(g.phase, Phase::Ready);
    }

    #[test]
    fn ball_speed_scales_with_grade() {
        let low  = BALL_BASE_SPEED + 0.0 * 0.28;
        let high = BALL_BASE_SPEED + 6.0 * 0.28;
        assert!(high > low);
    }
}
