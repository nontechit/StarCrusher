// Frog Lane — Star Academy edition.
//
// Portrait 720x1280 port of the educational arcade Frog Lane game built under
// games/junior-025-frog-lane. The standalone version targets desktop 960x540
// landscape; this module retargets the same gameplay for the mobile-first
// Star Academy hub and pulls its lesson content from `lesson_plans`.
//
// Lifecycle (matches MeteorCatch / NumberRain / PlasmaBreaker):
//
//   1. `FrogLane::new(grade)` — boots into the lesson-select screen with the
//      lesson set for `grade`.
//   2. `update()` returns one of [`FrogLaneAction`].
//   3. `draw()` is called every frame.
//
// On the hub: main.rs awards stars (0-3) on `Completed`, persists to
// PlayerProgress, and routes back to the hub.

use crate::lesson_plans::{
    self, LessonPlan, MathConcept, MathLessonData,
};
use crate::levels::Grade;
use crate::random;
use crate::screen;
use macroquad::prelude::*;
use std::f32::consts::{PI, TAU};

// ── Layout ───────────────────────────────────────────────────────────────────
const SW: f32 = 720.0;
const SH: f32 = 1280.0;
const HEADER_H: f32 = 240.0;
const LANES: i32 = 7;            // lane 0 = goal, 6 = start
const LANE_H: f32 = 120.0;
const LANE_TOP: f32 = HEADER_H;
const FOOTER_TOP: f32 = LANE_TOP + LANE_H * LANES as f32; // 1080

// ── Home / back button (matches PlasmaBreaker convention) ────────────────────
const HOME_X: f32 = 24.0;
const HOME_Y: f32 = 24.0;
const HOME_W: f32 = 110.0;
const HOME_H: f32 = 56.0;

const STARTING_LIVES: u32 = 3;
const RESPAWN_INVINCIBLE_SEC: f32 = 1.6;

/// Mobile gameplay slowdown. On touch devices (detected via the portrait
/// layout used everywhere else in the app) the action runs at this fraction
/// of desktop speed so PreK–6th players can keep up. Desktop is unscaled.
const MOBILE_SPEED_SCALE: f32 = 0.5;

/// Returns the per-frame time step scaled for the current platform: full speed
/// on desktop, `MOBILE_SPEED_SCALE` on portrait/touch.
fn play_speed_scale() -> f32 {
    if screen::portrait_layout() {
        MOBILE_SPEED_SCALE
    } else {
        1.0
    }
}

// ── Action returned to main.rs ───────────────────────────────────────────────
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrogLaneAction {
    None,
    ExitToHub,
    Completed { stars: u8 },
}

// ── Phase ────────────────────────────────────────────────────────────────────
#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase { Select, Playing, Won, Lost }

// ── Game types ───────────────────────────────────────────────────────────────
#[derive(Clone)]
struct Hazard {
    x: f32, y: f32, w: f32, speed: f32,
    kind: u8,        // 0 = road vehicle, 1 = river log
    lane: i32,
    size_big: bool,
    pat_idx: u8,
}

struct HopLabel { num: u32, x: f32, y: f32, ttl: f32 }

#[derive(Clone)]
struct Particle {
    x: f32, y: f32, vx: f32, vy: f32,
    ttl: f32, ttl_max: f32,
    color: Color, size: f32,
}

#[derive(Clone)]
struct GrassTuft { x: f32, y: f32, h: f32 }

// ── Game struct ──────────────────────────────────────────────────────────────
pub struct FrogLane {
    lessons: &'static [LessonPlan],
    phase: Phase,
    lesson_idx: usize,
    cursor: usize,
    grade: Grade,

    // Frog state
    px: f32, py: f32,
    lane: i32,
    frog_squash: f32,
    frog_face_dir: f32,

    // Run state
    hazards: Vec<Hazard>,
    on_log: Option<f32>,
    hops_done: u32,
    hop_cd: f32,
    hop_labels: Vec<HopLabel>,
    particles: Vec<Particle>,
    grass_tufts: Vec<GrassTuft>,
    time: f32,

    // Forgiveness mechanics
    lives: u32,
    max_lives: u32,
    invincible_for: f32,

    // For double-tap detection in lesson select
    last_select_tap_time: f32,
    last_select_tap_idx: Option<usize>,
}

impl FrogLane {
    pub fn new(grade: Grade) -> Self {
        let lessons = lesson_plans::math_lessons_for_grade(grade);
        // Pre-generate grass tufts (decoration only)
        let mut grass_tufts = Vec::with_capacity(40);
        for _ in 0..40 {
            grass_tufts.push(GrassTuft {
                x: random::f32_range(0.0, SW),
                y: random::f32_range(-LANE_H * 0.35, LANE_H * 0.35),
                h: random::f32_range(5.0, 10.0),
            });
        }
        Self {
            lessons,
            phase: Phase::Select,
            lesson_idx: 0,
            cursor: 0,
            grade,
            px: SW * 0.5,
            py: Self::lane_y(LANES - 1),
            lane: LANES - 1,
            frog_squash: 0.0,
            frog_face_dir: 0.0,
            hazards: Vec::new(),
            on_log: None,
            hops_done: 0,
            hop_cd: 0.0,
            hop_labels: Vec::new(),
            particles: Vec::new(),
            grass_tufts,
            time: 0.0,
            lives: STARTING_LIVES,
            max_lives: STARTING_LIVES,
            invincible_for: 0.0,
            last_select_tap_time: -10.0,
            last_select_tap_idx: None,
        }
    }

    fn lane_y(lane: i32) -> f32 {
        LANE_TOP + LANE_H * 0.5 + lane as f32 * LANE_H
    }

    fn current_math(&self) -> MathLessonData {
        // Falls back gracefully if a lesson plan is mis-keyed (no math data).
        self.lessons[self.lesson_idx]
            .math
            .unwrap_or(MathLessonData {
                concept: MathConcept::Counting,
                goal_hops: 5,
                start_count: 0,
            })
    }

    fn start_lesson(&mut self) {
        self.lesson_idx = self.cursor;
        self.px = SW * 0.5;
        self.lane = LANES - 1;
        self.py = Self::lane_y(self.lane);
        self.hops_done = 0;
        self.hop_labels.clear();
        self.particles.clear();
        self.frog_squash = 0.0;
        self.frog_face_dir = 0.0;
        self.lives = STARTING_LIVES;
        self.invincible_for = 0.0;
        self.spawn_hazards();
        self.phase = Phase::Playing;
    }

    fn spawn_hazards(&mut self) {
        let math = self.current_math();
        self.hazards.clear();
        let mut pat_counter: u8 = 0;

        // Only spawn in danger lanes (1..LANES-1) — never goal (0) or start (6).
        for lane in 1..(LANES - 1) {
            let y = Self::lane_y(lane);
            let is_river = lane >= 2 && lane <= 4;
            let count: u8 = 3; // pre-K tuning

            // Slow pre-K speeds
            let base_speed = if is_river {
                random::f32_range(35.0, 65.0)
            } else {
                random::f32_range(45.0, 90.0)
            };

            // Even spacing so there's always a visible gap
            let lane_span = SW + 200.0;
            let spacing = lane_span / count as f32;
            let jitter = spacing * 0.18;

            for i in 0..count {
                let direction = if i % 2 == 0 { 1.0f32 } else { -1.0 };

                let size_big = match math.concept {
                    MathConcept::SizeComp | MathConcept::Sorting => i < count / 2 + 1,
                    _ => random::bool(0.5),
                };

                let w = match math.concept {
                    MathConcept::SizeComp | MathConcept::Sorting => {
                        if size_big { random::f32_range(120.0, 170.0) }
                        else { random::f32_range(48.0, 70.0) }
                    }
                    _ => {
                        if is_river { random::f32_range(190.0, 270.0) }   // wide pre-K logs
                        else { random::f32_range(80.0, 120.0) }
                    }
                };

                // Big vehicles only modestly faster
                let speed_mult = if matches!(math.concept, MathConcept::SizeComp | MathConcept::Sorting) && size_big {
                    1.20
                } else { 1.0 };
                let speed = base_speed * direction * speed_mult;

                let pat_idx = if is_river { pat_counter % 3 } else { pat_counter % 2 };
                pat_counter = pat_counter.wrapping_add(1);

                let base_x = -100.0 + i as f32 * spacing + random::f32_range(-jitter, jitter);

                self.hazards.push(Hazard {
                    x: base_x,
                    y, w, speed,
                    kind: if is_river { 1 } else { 0 },
                    lane, size_big, pat_idx,
                });
            }
        }
    }

    fn try_hop(&mut self, dl: i32) {
        if self.hop_cd > 0.0 { return; }
        let nl = (self.lane + dl).clamp(0, LANES - 1);
        if nl == self.lane { return; }
        self.lane = nl;
        self.py = Self::lane_y(nl);
        self.hop_cd = 0.20;
        self.frog_squash = 1.0;
        self.on_log = None;
        // Small dust particles
        for _ in 0..6 {
            self.particles.push(Particle {
                x: self.px + random::f32_range(-10.0, 10.0),
                y: self.py + 16.0,
                vx: random::f32_range(-40.0, 40.0),
                vy: random::f32_range(-60.0, -15.0),
                ttl: 0.5, ttl_max: 0.5,
                color: Color::from_rgba(200, 200, 180, 180),
                size: random::f32_range(2.0, 4.0),
            });
        }
        if nl == 0 {
            self.score_crossing();
        }
    }

    fn score_crossing(&mut self) {
        self.hops_done += 1;
        let start_count = self.current_math().start_count;
        self.hop_labels.push(HopLabel {
            num: self.hops_done + start_count,
            x: self.px,
            y: self.py - 16.0,
            ttl: 1.6,
        });
        // Star burst
        for _ in 0..30 {
            let a = random::f32_range(0.0, TAU);
            let sp = random::f32_range(90.0, 240.0);
            self.particles.push(Particle {
                x: self.px, y: self.py,
                vx: a.cos() * sp,
                vy: a.sin() * sp - 50.0,
                ttl: 1.0, ttl_max: 1.0,
                color: Color::from_rgba(
                    random::i32_inclusive(220, 255) as u8,
                    random::i32_inclusive(180, 240) as u8,
                    random::i32_inclusive(40, 130) as u8,
                    255,
                ),
                size: random::f32_range(2.5, 5.0),
            });
        }
        // Reset frog
        self.lane = LANES - 1;
        self.py = Self::lane_y(self.lane);
        self.px = SW * 0.5;
        self.spawn_hazards();

        let goal = self.current_math().goal_hops;
        if self.hops_done >= goal {
            self.phase = Phase::Won;
            // Big celebration burst (centered)
            for _ in 0..90 {
                let a = random::f32_range(0.0, TAU);
                let sp = random::f32_range(120.0, 380.0);
                self.particles.push(Particle {
                    x: SW * 0.5, y: SH * 0.5,
                    vx: a.cos() * sp,
                    vy: a.sin() * sp - 40.0,
                    ttl: 1.8, ttl_max: 1.8,
                    color: Color::from_rgba(
                        random::i32_inclusive(150, 255) as u8,
                        random::i32_inclusive(150, 255) as u8,
                        random::i32_inclusive(60, 220) as u8,
                        255,
                    ),
                    size: random::f32_range(2.5, 6.5),
                });
            }
        }
    }

    fn die(&mut self) {
        if self.invincible_for > 0.0 { return; }
        for _ in 0..28 {
            let a = random::f32_range(0.0, TAU);
            let sp = random::f32_range(70.0, 240.0);
            self.particles.push(Particle {
                x: self.px, y: self.py,
                vx: a.cos() * sp,
                vy: a.sin() * sp,
                ttl: 1.0, ttl_max: 1.0,
                color: Color::from_rgba(
                    random::i32_inclusive(180, 240) as u8,
                    random::i32_inclusive(40, 90) as u8,
                    random::i32_inclusive(40, 90) as u8,
                    255,
                ),
                size: random::f32_range(2.5, 5.0),
            });
        }
        self.lives = self.lives.saturating_sub(1);
        if self.lives == 0 {
            self.phase = Phase::Lost;
        } else {
            // Respawn with brief invincibility
            self.lane = LANES - 1;
            self.py = Self::lane_y(self.lane);
            self.px = SW * 0.5;
            self.invincible_for = RESPAWN_INVINCIBLE_SEC;
            self.hop_cd = 0.0;
            self.frog_squash = 0.0;
            self.spawn_hazards();
        }
    }

    fn academy_stars(&self) -> u8 {
        let goal = self.current_math().goal_hops;
        if goal == 0 { return 0; }
        // Win → 3 stars (perfect), unless lives lost
        if self.phase == Phase::Won {
            if self.lives == self.max_lives { return 3; }
            if self.lives >= 1 { return 2; }
            return 1;
        }
        // Lost (out of lives)
        if self.hops_done == 0 { return 0; }
        let frac = self.hops_done as f32 / goal as f32;
        if frac >= 0.66 { 2 } else if frac >= 0.33 { 1 } else { 0 }
    }

    // ── Update ────────────────────────────────────────────────────────────────

    pub fn update(&mut self) -> FrogLaneAction {
        let dt = screen::frame_step();
        self.time += dt;

        // Particles always animate
        for p in self.particles.iter_mut() {
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.vy += 380.0 * dt;
            p.vx *= 0.985;
            p.ttl -= dt;
        }
        self.particles.retain(|p| p.ttl > 0.0);

        // Frog squash decay
        if self.frog_squash > 0.0 {
            self.frog_squash = (self.frog_squash - dt * 6.0).max(0.0);
        }

        // Esc / HOME button always exits
        if is_key_pressed(KeyCode::Escape) {
            return FrogLaneAction::ExitToHub;
        }
        if let Some(tap) = screen::primary_tap_position() {
            if home_rect().contains(tap) {
                return FrogLaneAction::ExitToHub;
            }
        }

        match self.phase {
            Phase::Select => self.update_select(),
            Phase::Playing => self.update_playing(dt),
            Phase::Won | Phase::Lost => self.update_result(),
        }
    }

    fn update_select(&mut self) -> FrogLaneAction {
        if is_key_pressed(KeyCode::Up) && self.cursor > 0 { self.cursor -= 1; }
        if is_key_pressed(KeyCode::Down) && self.cursor + 1 < self.lessons.len() {
            self.cursor += 1;
        }
        if is_key_pressed(KeyCode::Left) && self.cursor >= 6 { self.cursor -= 6; }
        if is_key_pressed(KeyCode::Right) && self.cursor + 6 < self.lessons.len() {
            self.cursor += 6;
        }
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
            self.start_lesson();
            return FrogLaneAction::None;
        }
        if let Some(tap) = screen::primary_tap_position() {
            // PLAY button at the bottom
            if select_play_rect().contains(tap) {
                self.start_lesson();
                return FrogLaneAction::None;
            }
            // Lesson tile
            if let Some(idx) = lesson_tile_at(tap, self.lessons.len()) {
                let now = self.time;
                let same = self.last_select_tap_idx == Some(idx);
                let recent = now - self.last_select_tap_time < 0.6;
                if same && recent {
                    self.cursor = idx;
                    self.start_lesson();
                    return FrogLaneAction::None;
                }
                self.cursor = idx;
                self.last_select_tap_idx = Some(idx);
                self.last_select_tap_time = now;
            }
        }
        FrogLaneAction::None
    }

    fn update_playing(&mut self, dt: f32) -> FrogLaneAction {
        // Slow the whole simulation on mobile (hazards, log drift, timers) so the
        // pace is comfortable for young players. Desktop keeps full speed.
        let dt = dt * play_speed_scale();

        self.hop_cd = (self.hop_cd - dt).max(0.0);
        self.invincible_for = (self.invincible_for - dt).max(0.0);

        // Keyboard
        if is_key_pressed(KeyCode::W) || is_key_pressed(KeyCode::Up) { self.try_hop(-1); }
        if is_key_pressed(KeyCode::S) || is_key_pressed(KeyCode::Down) { self.try_hop(1); }
        if is_key_pressed(KeyCode::A) || is_key_pressed(KeyCode::Left) {
            self.px -= 50.0;
            self.frog_face_dir = -1.0;
        }
        if is_key_pressed(KeyCode::D) || is_key_pressed(KeyCode::Right) {
            self.px += 50.0;
            self.frog_face_dir = 1.0;
        }

        // Touch / mouse — tap relative to frog
        if let Some(p) = screen::primary_tap_position() {
            // HOME already handled in update(); below the header the tap steers the frog.
            if p.y > HEADER_H {
                let dx = p.x - self.px;
                let dy = p.y - self.py;
                if dy.abs() > dx.abs() {
                    if dy < -18.0 { self.try_hop(-1); }
                    else if dy > 18.0 { self.try_hop(1); }
                } else if dx.abs() > 16.0 {
                    if dx < 0.0 { self.px -= 56.0; self.frog_face_dir = -1.0; }
                    else { self.px += 56.0; self.frog_face_dir = 1.0; }
                }
            }
        }

        // Move hazards
        for h in self.hazards.iter_mut() {
            h.x += h.speed * dt;
            if h.x > SW + h.w { h.x = -h.w; }
            if h.x < -h.w { h.x = SW + h.w; }
        }

        // Collision
        let fy = Self::lane_y(self.lane);
        self.on_log = None;
        if self.lane >= 2 && self.lane <= 4 {
            // River — must be on a log (8px forgiveness)
            let mut on_any = false;
            for h in self.hazards.iter().filter(|h| h.kind == 1 && (h.y - fy).abs() < 1.0) {
                if self.px >= h.x - 10.0 && self.px <= h.x + h.w + 10.0 {
                    self.px += h.speed * dt;
                    self.on_log = Some(h.speed);
                    on_any = true;
                }
            }
            if !on_any {
                self.die();
                if self.phase != Phase::Playing { return FrogLaneAction::None; }
            }
        } else if self.lane != 0 && self.lane != LANES - 1 {
            // Road — avoid vehicles (hitbox 6px shrunk on each side)
            for h in self.hazards.iter().filter(|h| h.kind == 0 && (h.y - fy).abs() < 1.0) {
                if self.px >= h.x + 6.0 && self.px <= h.x + h.w - 6.0 {
                    self.die();
                    if self.phase != Phase::Playing { return FrogLaneAction::None; }
                    break;
                }
            }
        }
        self.px = self.px.clamp(28.0, SW - 28.0);

        for lbl in self.hop_labels.iter_mut() {
            lbl.y -= 60.0 * dt;
            lbl.ttl -= dt;
        }
        self.hop_labels.retain(|lbl| lbl.ttl > 0.0);

        FrogLaneAction::None
    }

    fn update_result(&mut self) -> FrogLaneAction {
        // Any tap or Enter advances to hub with stars
        let advance = is_key_pressed(KeyCode::Enter)
            || is_key_pressed(KeyCode::Space)
            || screen::primary_tap_position().is_some();
        if advance {
            let stars = self.academy_stars();
            FrogLaneAction::Completed { stars }
        } else {
            FrogLaneAction::None
        }
    }

    // ── Draw ──────────────────────────────────────────────────────────────────

    pub fn draw(&self) {
        clear_background(Color::from_rgba(8, 12, 22, 255));
        match self.phase {
            Phase::Select => self.draw_select(),
            Phase::Playing => self.draw_playing(),
            Phase::Won => { self.draw_playing(); self.draw_result_overlay(true); }
            Phase::Lost => { self.draw_playing(); self.draw_result_overlay(false); }
        }
        self.draw_particles();
        draw_home_button();
    }

    fn draw_particles(&self) {
        for p in &self.particles {
            let frac = (p.ttl / p.ttl_max).clamp(0.0, 1.0);
            let mut c = p.color;
            c.a = frac;
            draw_circle(p.x, p.y, p.size * frac.max(0.3), c);
        }
    }

    fn draw_select(&self) {
        // Title
        draw_centered_text("PICK A LESSON", 92.0, 40, Color::from_rgba(255, 220, 80, 255));
        let grade_str = format!("{}  ·  Pre-K Mathematics", self.grade.display_name());
        draw_centered_text(&grade_str, 130.0, 22, Color::from_rgba(160, 200, 170, 255));

        // 12 lesson tiles in 2 columns x 6 rows
        for (i, lesson) in self.lessons.iter().enumerate() {
            let rect = lesson_tile_rect(i);
            let selected = i == self.cursor;
            let bg = if selected {
                Color::from_rgba(40, 90, 60, 230)
            } else {
                Color::from_rgba(18, 32, 28, 230)
            };
            filled_pill(rect.x, rect.y, rect.w, rect.h, 14.0, bg);
            if selected {
                draw_pill_outline(rect.x, rect.y, rect.w, rect.h, 14.0,
                    Color::from_rgba(110, 230, 150, 230), 3.0);
            }
            // Icon
            draw_lesson_icon(
                rect.x + 30.0,
                rect.y + rect.h * 0.5,
                lesson.math.map(|m| m.concept).unwrap_or(MathConcept::Counting),
            );
            // ID + title
            draw_text_ex(
                lesson.id,
                rect.x + 60.0,
                rect.y + 30.0,
                TextParams { font_size: 18, color: Color::from_rgba(120, 220, 160, 255), ..Default::default() },
            );
            let title_color = if selected {
                Color::from_rgba(230, 255, 220, 255)
            } else {
                Color::from_rgba(190, 210, 200, 255)
            };
            draw_text_ex(
                lesson.title,
                rect.x + 60.0,
                rect.y + 56.0,
                TextParams { font_size: 19, color: title_color, ..Default::default() },
            );
            draw_text_ex(
                lesson.concept,
                rect.x + 14.0,
                rect.y + rect.h - 14.0,
                TextParams { font_size: 14, color: Color::from_rgba(130, 160, 140, 255), ..Default::default() },
            );
        }

        // Bottom info bar + PLAY button
        let sel = self.lessons[self.cursor];
        let info_y = SH - 200.0;
        draw_rectangle(0.0, info_y, SW, 200.0, Color::from_rgba(15, 35, 25, 235));
        draw_line(0.0, info_y, SW, info_y, 1.0, Color::from_rgba(60, 120, 80, 200));
        draw_text_ex(
            sel.id, 20.0, info_y + 30.0,
            TextParams { font_size: 18, color: Color::from_rgba(120, 220, 160, 255), ..Default::default() },
        );
        draw_text_ex(
            sel.title, 80.0, info_y + 30.0,
            TextParams { font_size: 20, color: Color::from_rgba(220, 255, 230, 255), ..Default::default() },
        );
        draw_text_ex(
            sel.instruction, 20.0, info_y + 58.0,
            TextParams { font_size: 16, color: Color::from_rgba(200, 220, 200, 255), ..Default::default() },
        );
        let goal_text = format!(
            "Goal: {} crossings   ·   {} hearts",
            sel.math.map(|m| m.goal_hops).unwrap_or(5),
            STARTING_LIVES,
        );
        draw_text_ex(
            &goal_text, 20.0, info_y + 82.0,
            TextParams { font_size: 14, color: Color::from_rgba(180, 160, 80, 220), ..Default::default() },
        );

        // PLAY button (pulsing)
        let pulse = ((self.time * 3.0).sin() * 0.15 + 0.85) as f32;
        let play_color = Color::new(0.20, 0.85 * pulse, 0.40, 1.0);
        let r = select_play_rect();
        filled_pill(r.x, r.y, r.w, r.h, r.h * 0.5, Color::from_rgba(20, 80, 40, 240));
        draw_pill_outline(r.x, r.y, r.w, r.h, r.h * 0.5, play_color, 3.0);
        let label = "PLAY ▶";
        let tm = measure_text(label, None, 32, 1.0);
        draw_text_ex(
            label,
            r.x + (r.w - tm.width) * 0.5,
            r.y + r.h * 0.66,
            TextParams { font_size: 32, color: WHITE, ..Default::default() },
        );
    }

    fn draw_playing(&self) {
        let lesson = self.lessons[self.lesson_idx];
        let math = self.current_math();

        // ── Lanes ──────────────────────────────────────────────────────────────
        for lane in 0..LANES {
            let cy = Self::lane_y(lane);
            let top = cy - LANE_H * 0.5;
            let bg = lane_bg_color(lane, &lesson);
            draw_rectangle(0.0, top, SW, LANE_H, bg);

            if lane == 0 || lane == LANES - 1 {
                self.draw_grass(top);
            } else if lane >= 2 && lane <= 4 {
                self.draw_water(top, lane);
            } else {
                self.draw_road(top, &lesson);
            }

            // Lane label
            let lname = lane_label(lane, &lesson);
            if !lname.is_empty() {
                draw_text_ex(
                    lname, 8.0, cy + 6.0,
                    TextParams { font_size: 14, color: Color::from_rgba(15, 25, 20, 220), ..Default::default() },
                );
                draw_text_ex(
                    lname, 6.0, cy + 4.0,
                    TextParams { font_size: 14, color: Color::from_rgba(240, 240, 240, 230), ..Default::default() },
                );
            }

            if lane == 0 {
                draw_goal_beacon(cy, &lesson, self.time);
            }
        }

        // ── Quantity overlay (PK-10) ───────────────────────────────────────────
        if math.concept == MathConcept::QuantComp {
            draw_quantity_comparison(&self.hazards);
        }
        if math.concept == MathConcept::Patterns {
            draw_pattern_legend();
        }

        // ── Hazards ───────────────────────────────────────────────────────────
        for h in &self.hazards {
            let col = hazard_color(h, &lesson);
            let lbl = hazard_label(h, &lesson);
            if h.kind == 1 {
                draw_log(h.x, h.y - 20.0, h.w, 40.0, col,
                    math.concept == MathConcept::Shapes && lesson.id == "PK-MATH-06");
            } else {
                draw_car(h.x, h.y - 20.0, h.w, 40.0, col, h.speed > 0.0);
            }
            if !lbl.is_empty() {
                let fsz = if h.w >= 120.0 { 18 } else { 14 };
                let tx = (h.x + 8.0).min(h.x + h.w - 38.0);
                draw_text_ex(
                    lbl, tx + 1.5, h.y + 7.0,
                    TextParams { font_size: fsz, color: Color::from_rgba(20, 20, 20, 220), ..Default::default() },
                );
                draw_text_ex(
                    lbl, tx, h.y + 5.5,
                    TextParams { font_size: fsz, color: Color::from_rgba(255, 255, 255, 240), ..Default::default() },
                );
            }
        }

        // ── Frog ──────────────────────────────────────────────────────────────
        draw_frog(self.px, self.py, self.frog_squash, self.frog_face_dir, self.time, self.invincible_for);
        if math.concept == MathConcept::Shapes {
            let shp = if lesson.id == "PK-MATH-05" { "CIRCLE" } else { "OVAL" };
            let tw = measure_text(shp, None, 16, 1.0).width;
            draw_text_ex(
                shp,
                self.px - tw * 0.5,
                self.py - 32.0,
                TextParams { font_size: 16, color: Color::from_rgba(80, 255, 160, 240), ..Default::default() },
            );
        }

        // ── Floating count labels ─────────────────────────────────────────────
        for lbl in &self.hop_labels {
            let alpha = (lbl.ttl / 1.6).clamp(0.0, 1.0);
            let scale = 1.0 + (1.0 - alpha) * 1.3;
            let fsz = (54.0 * scale) as u16;
            draw_text_ex(
                &format!("{}!", lbl.num),
                lbl.x - fsz as f32 * 0.30 + 2.0, lbl.y + 2.0,
                TextParams { font_size: fsz, color: Color::new(0.0, 0.0, 0.0, alpha * 0.6), ..Default::default() },
            );
            draw_text_ex(
                &format!("{}!", lbl.num),
                lbl.x - fsz as f32 * 0.30, lbl.y,
                TextParams { font_size: fsz, color: Color::new(1.0, 0.88, 0.15, alpha), ..Default::default() },
            );
        }

        // ── Header (drawn last) ────────────────────────────────────────────────
        self.draw_header(&lesson, &math);
    }

    fn draw_header(&self, lesson: &LessonPlan, math: &MathLessonData) {
        // Backdrop
        draw_rectangle(0.0, 0.0, SW, HEADER_H, Color::from_rgba(10, 18, 28, 240));
        draw_line(0.0, HEADER_H, SW, HEADER_H, 2.0, Color::from_rgba(60, 110, 80, 220));

        // Lesson title row
        let id_title = format!("{}: {}", lesson.id, lesson.title);
        draw_centered_text(&id_title, 100.0, 22, Color::from_rgba(120, 220, 160, 255));
        draw_centered_text(lesson.concept, 124.0, 15, Color::from_rgba(140, 175, 150, 255));
        draw_centered_text(lesson.instruction, 150.0, 15, Color::from_rgba(180, 210, 195, 255));

        // Hop counter centered
        let goal_total_dots = math.start_count + math.goal_hops;
        let dot_r = if goal_total_dots <= 5 { 12.0 }
            else if goal_total_dots <= 10 { 10.0 }
            else if goal_total_dots <= 15 { 8.5 }
            else { 7.5 };
        let dot_gap = (dot_r * 2.0_f32 + 6.0_f32).max(18.0_f32);
        let row_w = goal_total_dots as f32 * dot_gap;
        let dot_y = 200.0;
        let mut dot_x = SW * 0.5 - row_w * 0.5 + dot_r;
        for i in 0..goal_total_dots {
            let in_pre = i < math.start_count;
            let active = (i >= math.start_count) && (i < math.start_count + self.hops_done);
            let c = if active {
                Color::from_rgba(80, 255, 160, 255)
            } else if in_pre {
                Color::from_rgba(45, 130, 90, 220)
            } else {
                Color::from_rgba(60, 80, 70, 200)
            };
            draw_circle(dot_x, dot_y, dot_r, c);
            if active {
                draw_circle_lines(dot_x, dot_y, dot_r, 1.5, Color::from_rgba(180, 255, 200, 255));
            }
            if active || in_pre {
                let label = format!("{}", i + 1);
                let lm = measure_text(&label, None, 12, 1.0);
                draw_text_ex(
                    &label,
                    dot_x - lm.width * 0.5,
                    dot_y + 4.0,
                    TextParams { font_size: 12, color: Color::from_rgba(8, 20, 12, 240), ..Default::default() },
                );
            }
            dot_x += dot_gap;
        }

        // Hearts (top-right)
        for i in 0..self.max_lives {
            let hx = SW - 28.0 - i as f32 * 36.0;
            let active = i < self.lives;
            let col = if active {
                Color::from_rgba(255, 90, 110, 255)
            } else {
                Color::from_rgba(70, 36, 42, 220)
            };
            draw_heart(hx, 52.0, 14.0, col);
        }
    }

    fn draw_result_overlay(&self, won: bool) {
        draw_rectangle(0.0, 0.0, SW, SH, Color::from_rgba(0, 0, 0, 175));
        let cx = SW * 0.5;
        if won {
            // Pulsing star
            let pulse = (self.time * 4.0).sin() * 0.18 + 1.0;
            draw_star(cx, SH * 0.32, 100.0 * pulse, Color::from_rgba(255, 220, 80, 255));
            draw_centered_text("YOU DID IT!", SH * 0.45, 58, Color::from_rgba(80, 255, 160, 255));
            let lesson = self.lessons[self.lesson_idx];
            draw_centered_text(lesson.success, SH * 0.52, 22, Color::from_rgba(210, 245, 210, 255));
        } else {
            draw_centered_text("OUT OF HEARTS", SH * 0.42, 48, Color::from_rgba(255, 110, 110, 255));
            draw_centered_text(
                &format!("You made {} crossings", self.hops_done),
                SH * 0.48, 22, Color::from_rgba(230, 200, 200, 255),
            );
        }
        // Star earnings preview
        let stars = self.academy_stars();
        let star_y = SH * 0.60;
        let star_r = 28.0;
        let star_gap = 78.0;
        let row_w = 3.0 * star_gap;
        let mut sx = cx - row_w * 0.5 + star_gap * 0.5;
        for s in 0..3u8 {
            let col = if s < stars {
                Color::from_rgba(255, 220, 80, 255)
            } else {
                Color::from_rgba(60, 50, 30, 200)
            };
            draw_star(sx, star_y, star_r, col);
            sx += star_gap;
        }
        draw_centered_text(
            &format!("{} of 3 stars earned", stars),
            SH * 0.66, 20, Color::from_rgba(220, 220, 200, 255),
        );

        draw_centered_text(
            "Tap anywhere to return to the hub",
            SH * 0.75, 18, Color::from_rgba(160, 180, 160, 255),
        );
    }

    // ── Lane decorations ──────────────────────────────────────────────────────
    fn draw_grass(&self, top: f32) {
        for t in &self.grass_tufts {
            let gx = t.x;
            let gy = top + LANE_H * 0.5 + t.y;
            let g_col = Color::from_rgba(38, 110, 50, 220);
            draw_line(gx, gy + t.h, gx, gy - t.h, 1.8, g_col);
            draw_line(gx - 3.0, gy + t.h, gx - 3.0, gy - t.h * 0.7, 1.2, g_col);
            draw_line(gx + 3.0, gy + t.h, gx + 3.0, gy - t.h * 0.7, 1.2, g_col);
        }
    }

    fn draw_water(&self, top: f32, lane: i32) {
        let wave_col = Color::from_rgba(180, 220, 240, 100);
        let ofs = self.time * 28.0 + lane as f32 * 19.0;
        for i in 0..12 {
            let bx = ((i as f32 * 70.0 + ofs) % (SW + 70.0)) - 35.0;
            let by = top + LANE_H * 0.5 + ((self.time * 2.0 + i as f32 * 0.7).sin() * 8.0);
            draw_line(bx, by, bx + 38.0, by, 2.0, wave_col);
        }
        let sp_col = Color::from_rgba(255, 255, 255, 150);
        for i in 0..6 {
            let sx = (i as f32 * 137.0 + self.time * 13.0) % SW;
            let sy = top + 8.0 + ((self.time * 1.7 + i as f32 * 1.1).sin().abs() * (LANE_H - 16.0));
            draw_circle(sx, sy, 1.5, sp_col);
        }
    }

    fn draw_road(&self, top: f32, lesson: &LessonPlan) {
        let kind = lesson.math.map(|m| m.concept).unwrap_or(MathConcept::Counting);
        if !matches!(kind, MathConcept::Colors) {
            let dash_col = Color::from_rgba(230, 220, 130, 220);
            let dash_w = 30.0;
            let gap = 20.0;
            let cy = top + LANE_H * 0.5;
            let mut x = -((self.time * 22.0) % (dash_w + gap));
            while x < SW {
                draw_rectangle(x, cy - 2.0, dash_w, 4.0, dash_col);
                x += dash_w + gap;
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Layout helpers
// ─────────────────────────────────────────────────────────────────────────────

fn home_rect() -> Rect { Rect::new(HOME_X, HOME_Y, HOME_W, HOME_H) }

fn lesson_tile_rect(i: usize) -> Rect {
    let col = (i / 6) as f32;
    let row = (i % 6) as f32;
    let w = 332.0f32;
    let h = 116.0f32;
    let x = 20.0 + col * (w + 16.0);
    let y = 160.0 + row * (h + 12.0);
    Rect::new(x, y, w, h)
}

fn lesson_tile_at(p: Vec2, count: usize) -> Option<usize> {
    for i in 0..count {
        let r = lesson_tile_rect(i);
        if r.contains(p) {
            return Some(i);
        }
    }
    None
}

fn select_play_rect() -> Rect {
    Rect::new(SW * 0.5 - 130.0, SH - 110.0, 260.0, 80.0)
}

// ─────────────────────────────────────────────────────────────────────────────
//  Drawing primitives
// ─────────────────────────────────────────────────────────────────────────────

fn draw_centered_text(text: &str, y: f32, size: u16, color: Color) {
    if text.is_empty() { return; }
    let m = measure_text(text, None, size, 1.0);
    draw_text_ex(
        text,
        SW * 0.5 - m.width * 0.5,
        y,
        TextParams { font_size: size, color, ..Default::default() },
    );
}

fn filled_pill(x: f32, y: f32, w: f32, h: f32, radius: f32, color: Color) {
    let r = radius.min(w * 0.5).min(h * 0.5);
    draw_rectangle(x + r, y, w - r * 2.0, h, color);
    draw_rectangle(x, y + r, w, h - r * 2.0, color);
    draw_circle(x + r, y + r, r, color);
    draw_circle(x + w - r, y + r, r, color);
    draw_circle(x + r, y + h - r, r, color);
    draw_circle(x + w - r, y + h - r, r, color);
}

fn draw_pill_outline(x: f32, y: f32, w: f32, h: f32, _radius: f32, color: Color, thickness: f32) {
    draw_rectangle_lines(x - 1.0, y - 1.0, w + 2.0, h + 2.0, thickness, color);
}

fn draw_home_button() {
    filled_pill(HOME_X, HOME_Y, HOME_W, HOME_H, HOME_H * 0.5,
        Color::from_rgba(35, 60, 80, 230));
    draw_pill_outline(HOME_X, HOME_Y, HOME_W, HOME_H, HOME_H * 0.5,
        Color::from_rgba(110, 200, 235, 220), 2.0);
    let label = "‹ HOME";
    let m = measure_text(label, None, 24, 1.0);
    draw_text_ex(
        label,
        HOME_X + (HOME_W - m.width) * 0.5,
        HOME_Y + HOME_H * 0.66,
        TextParams { font_size: 24, color: WHITE, ..Default::default() },
    );
}

fn draw_lesson_icon(cx: f32, cy: f32, kind: MathConcept) {
    match kind {
        MathConcept::Counting => {
            for i in 0..3u32 {
                draw_circle(cx - 9.0 + i as f32 * 9.0, cy, 3.5, Color::from_rgba(120, 220, 160, 255));
            }
        }
        MathConcept::Shapes => {
            draw_triangle(
                Vec2::new(cx, cy - 11.0),
                Vec2::new(cx - 10.0, cy + 8.0),
                Vec2::new(cx + 10.0, cy + 8.0),
                Color::from_rgba(240, 200, 80, 255),
            );
        }
        MathConcept::Colors => {
            draw_circle(cx - 7.0, cy - 3.0, 5.5, Color::from_rgba(220, 60, 60, 255));
            draw_circle(cx + 7.0, cy - 3.0, 5.5, Color::from_rgba(60, 110, 230, 255));
            draw_circle(cx, cy + 5.5, 5.5, Color::from_rgba(80, 200, 90, 255));
        }
        MathConcept::SizeComp => {
            draw_rectangle(cx - 11.0, cy - 8.0, 12.0, 16.0, Color::from_rgba(240, 130, 30, 255));
            draw_rectangle(cx + 3.0, cy - 3.0, 6.0, 6.0, Color::from_rgba(60, 170, 170, 255));
        }
        MathConcept::QuantComp => {
            for i in 0..4 { draw_circle(cx - 12.0 + i as f32 * 4.5, cy, 2.0, Color::from_rgba(220, 220, 160, 255)); }
            for i in 0..2 { draw_circle(cx + 4.0 + i as f32 * 4.5, cy + 4.5, 2.0, Color::from_rgba(220, 220, 160, 255)); }
        }
        MathConcept::Patterns => {
            for i in 0..4 {
                let c = if i % 2 == 0 { Color::from_rgba(220, 60, 60, 255) } else { Color::from_rgba(60, 110, 230, 255) };
                draw_rectangle(cx - 11.0 + i as f32 * 6.0, cy - 4.0, 5.0, 8.0, c);
            }
        }
        MathConcept::Sorting => {
            draw_rectangle(cx - 11.0, cy - 6.0, 8.0, 12.0, Color::from_rgba(220, 60, 60, 255));
            draw_circle(cx + 6.0, cy, 5.5, Color::from_rgba(60, 110, 230, 255));
        }
    }
}

fn lane_bg_color(lane: i32, lesson: &LessonPlan) -> Color {
    let concept = lesson.math.map(|m| m.concept).unwrap_or(MathConcept::Counting);
    let is_river = lane >= 2 && lane <= 4;
    let is_grass = lane == 0 || lane == LANES - 1;
    if matches!(concept, MathConcept::Colors) {
        if lesson.id == "PK-MATH-07" {
            match lane {
                0 => Color::from_rgba(220, 200, 30, 255),
                1 => Color::from_rgba(150, 22, 22, 255),
                2 => Color::from_rgba(20, 60, 175, 255),
                3 => Color::from_rgba(12, 40, 100, 255),
                4 => Color::from_rgba(22, 130, 50, 255),
                5 => Color::from_rgba(150, 22, 22, 255),
                _ => Color::from_rgba(22, 100, 45, 255),
            }
        } else {
            match lane {
                0 => Color::from_rgba(220, 220, 220, 255),
                1 => Color::from_rgba(195, 95, 14, 255),
                2 => Color::from_rgba(100, 22, 140, 255),
                3 => Color::from_rgba(72, 46, 22, 255),
                4 => Color::from_rgba(100, 22, 140, 255),
                5 => Color::from_rgba(195, 95, 14, 255),
                _ => Color::from_rgba(85, 60, 35, 255),
            }
        }
    } else if is_river {
        Color::from_rgba(28, 75, 155, 255)
    } else if is_grass {
        Color::from_rgba(58, 130, 65, 255)
    } else {
        Color::from_rgba(58, 58, 70, 255)
    }
}

fn lane_label(lane: i32, lesson: &LessonPlan) -> &'static str {
    let concept = lesson.math.map(|m| m.concept).unwrap_or(MathConcept::Counting);
    if matches!(concept, MathConcept::Colors) {
        if lesson.id == "PK-MATH-07" {
            match lane { 0 => "YELLOW", 1 | 5 => "RED", 2 | 3 => "BLUE", 4 => "GREEN", _ => "GREEN" }
        } else {
            match lane { 0 => "WHITE", 1 | 5 => "ORANGE", 2 | 4 => "PURPLE", 3 => "BROWN", _ => "BROWN" }
        }
    } else {
        match lane { 0 => "GOAL", 6 => "START", 1 | 5 => "ROAD", 2 | 3 | 4 => "RIVER", _ => "" }
    }
}

fn draw_goal_beacon(cy: f32, lesson: &LessonPlan, time: f32) {
    let pulse = (time * 3.0).sin() * 0.25 + 0.85;
    let glow = Color::from_rgba(255, 230, 90, ((pulse * 0.5 + 0.2) * 110.0) as u8);
    let cx = SW * 0.5;
    draw_circle(cx, cy, 60.0 * pulse, glow);
    draw_circle(cx, cy, 40.0, Color::from_rgba(255, 240, 130, 230));
    if lesson.id == "PK-MATH-05" {
        draw_triangle(
            Vec2::new(cx, cy - 28.0),
            Vec2::new(cx - 28.0, cy + 22.0),
            Vec2::new(cx + 28.0, cy + 22.0),
            Color::from_rgba(255, 165, 30, 255),
        );
        draw_text_ex("TRIANGLE", cx + 38.0, cy + 6.0,
            TextParams { font_size: 16, color: Color::from_rgba(40, 30, 10, 255), ..Default::default() });
    } else if lesson.id == "PK-MATH-06" {
        draw_star(cx, cy, 36.0, Color::from_rgba(255, 165, 30, 255));
        draw_text_ex("STAR", cx + 42.0, cy + 6.0,
            TextParams { font_size: 16, color: Color::from_rgba(40, 30, 10, 255), ..Default::default() });
    } else {
        draw_circle(cx, cy, 22.0, Color::from_rgba(255, 165, 30, 255));
        let m = measure_text("GOAL", None, 16, 1.0);
        draw_text_ex(
            "GOAL",
            cx - m.width * 0.5,
            cy + 6.0,
            TextParams { font_size: 16, color: Color::from_rgba(40, 30, 10, 255), ..Default::default() },
        );
    }
}

fn hazard_color(h: &Hazard, lesson: &LessonPlan) -> Color {
    let concept = lesson.math.map(|m| m.concept).unwrap_or(MathConcept::Counting);
    match concept {
        MathConcept::Colors => {
            if lesson.id == "PK-MATH-07" {
                match h.lane {
                    1 | 5 => Color::from_rgba(225, 60, 60, 255),
                    2 | 3 => Color::from_rgba(60, 110, 230, 255),
                    4 => Color::from_rgba(60, 195, 80, 255),
                    _ => Color::from_rgba(220, 200, 60, 255),
                }
            } else {
                match h.lane {
                    1 | 5 => Color::from_rgba(240, 125, 25, 255),
                    2 | 4 => Color::from_rgba(160, 60, 200, 255),
                    3 => Color::from_rgba(140, 95, 55, 255),
                    _ => Color::from_rgba(180, 180, 180, 255),
                }
            }
        }
        MathConcept::Patterns => {
            if h.kind == 0 {
                if h.pat_idx % 2 == 0 { Color::from_rgba(220, 60, 60, 255) }
                else { Color::from_rgba(60, 110, 230, 255) }
            } else if h.pat_idx % 3 != 2 {
                Color::from_rgba(220, 60, 60, 255)
            } else {
                Color::from_rgba(60, 110, 230, 255)
            }
        }
        MathConcept::SizeComp => {
            if h.size_big { Color::from_rgba(240, 130, 25, 255) }
            else { Color::from_rgba(60, 175, 170, 255) }
        }
        MathConcept::Sorting => {
            if h.size_big { Color::from_rgba(220, 60, 60, 255) }
            else { Color::from_rgba(60, 110, 230, 255) }
        }
        _ => {
            if h.kind == 1 { Color::from_rgba(140, 90, 55, 255) }
            else { Color::from_rgba(210, 70, 70, 255) }
        }
    }
}

fn hazard_label(h: &Hazard, lesson: &LessonPlan) -> &'static str {
    let concept = lesson.math.map(|m| m.concept).unwrap_or(MathConcept::Counting);
    match concept {
        MathConcept::Shapes => {
            if lesson.id == "PK-MATH-05" {
                if h.kind == 1 { "LOG" } else { "RECT" }
            } else if h.kind == 1 { "OVAL" } else { "RECT" }
        }
        MathConcept::SizeComp => if h.size_big { "BIG" } else { "small" },
        MathConcept::Sorting => if h.size_big { "BIG-RED" } else { "sm-BLU" },
        MathConcept::Patterns => {
            if h.kind == 0 {
                if h.pat_idx % 2 == 0 { "A" } else { "B" }
            } else {
                match h.pat_idx % 3 { 0 | 1 => "A", _ => "B" }
            }
        }
        MathConcept::Colors => {
            if lesson.id == "PK-MATH-07" {
                match h.lane { 1 | 5 => "RED", 2 | 3 => "BLUE", 4 => "GREEN", _ => "" }
            } else {
                match h.lane { 1 | 5 => "ORG", 2 | 4 => "PRP", 3 => "BRN", _ => "" }
            }
        }
        _ => "",
    }
}

fn draw_quantity_comparison(hazards: &[Hazard]) {
    let half = SW * 0.5;
    for &lane in &[1i32, 5] {
        let ly = FrogLane::lane_y(lane);
        let left = hazards.iter().filter(|h| h.lane == lane && h.x < half).count();
        let right = hazards.iter().filter(|h| h.lane == lane && h.x >= half).count();
        let rel = if left > right { "L=MORE" } else if right > left { "R=MORE" } else { "EQUAL" };
        draw_rectangle(SW - 188.0, ly - 16.0, 180.0, 22.0, Color::from_rgba(0, 0, 0, 170));
        draw_text_ex(
            &format!("L:{} R:{} {}", left, right, rel),
            SW - 184.0, ly + 0.0,
            TextParams { font_size: 16, color: Color::from_rgba(220, 235, 220, 255), ..Default::default() },
        );
    }
}

fn draw_pattern_legend() {
    let bx = 20.0;
    let by = FOOTER_TOP + 40.0;
    draw_rectangle(bx - 6.0, by - 22.0, SW - 28.0, 100.0, Color::from_rgba(8, 16, 28, 220));
    draw_rectangle_lines(bx - 6.0, by - 22.0, SW - 28.0, 100.0, 1.5, Color::from_rgba(60, 100, 80, 200));
    draw_text_ex(
        "Road pattern (AB):", bx, by,
        TextParams { font_size: 17, color: Color::from_rgba(220, 235, 220, 255), ..Default::default() },
    );
    for i in 0..4u32 {
        let c = if i % 2 == 0 { Color::from_rgba(220, 60, 60, 255) } else { Color::from_rgba(60, 110, 230, 255) };
        draw_rectangle(bx + 200.0 + i as f32 * 50.0, by - 14.0, 40.0, 22.0, c);
        draw_text_ex(
            if i % 2 == 0 { "A" } else { "B" },
            bx + 215.0 + i as f32 * 50.0, by + 0.0,
            TextParams { font_size: 17, color: WHITE, ..Default::default() },
        );
    }
    draw_text_ex(
        "River pattern (AAB):", bx, by + 44.0,
        TextParams { font_size: 17, color: Color::from_rgba(220, 235, 220, 255), ..Default::default() },
    );
    for i in 0..3u32 {
        let c = if i < 2 { Color::from_rgba(220, 60, 60, 255) } else { Color::from_rgba(60, 110, 230, 255) };
        draw_rectangle(bx + 200.0 + i as f32 * 50.0, by + 30.0, 40.0, 22.0, c);
        draw_text_ex(
            if i < 2 { "A" } else { "B" },
            bx + 215.0 + i as f32 * 50.0, by + 44.0,
            TextParams { font_size: 17, color: WHITE, ..Default::default() },
        );
    }
}

// ── Frog sprite ──────────────────────────────────────────────────────────────
fn draw_frog(x: f32, y: f32, squash: f32, face_dir: f32, time: f32, invincible_for: f32) {
    let inv = invincible_for > 0.0;
    let alpha: f32 = if inv {
        if (time * 16.0).sin() > 0.0 { 0.90 } else { 0.35 }
    } else { 1.0 };
    let tint = |r: u8, g: u8, b: u8, a: u8| -> Color {
        let cyan_mix = if inv { 0.25 } else { 0.0 };
        Color::new(
            (r as f32 / 255.0) * (1.0 - cyan_mix) + 0.3 * cyan_mix,
            (g as f32 / 255.0) * (1.0 - cyan_mix) + 0.85 * cyan_mix,
            (b as f32 / 255.0) * (1.0 - cyan_mix) + 1.0 * cyan_mix,
            (a as f32 / 255.0) * alpha,
        )
    };

    let s = squash.clamp(0.0, 1.0);
    let rx = 24.0 * (1.0 + s * 0.45);
    let ry = 24.0 * (1.0 - s * 0.30);
    // Shadow
    draw_ellipse_filled(x, y + 24.0 + s * 5.0, 22.0 - s * 6.0, 6.0,
        Color::new(0.0, 0.0, 0.0, 0.35 * alpha));
    // Hind legs
    let leg_col = tint(50, 170, 90, 255);
    draw_circle(x - rx * 0.55, y + ry * 0.45, 8.5, leg_col);
    draw_circle(x + rx * 0.55, y + ry * 0.45, 8.5, leg_col);
    // Body
    let body_col = tint(85, 220, 120, 255);
    draw_ellipse_filled(x, y, rx, ry, body_col);
    // Belly
    draw_ellipse_filled(x, y + ry * 0.25, rx * 0.7, ry * 0.45, tint(190, 245, 200, 220));
    // Eyes
    let eye_dx = rx * 0.45;
    let eye_y = y - ry * 0.55;
    let eye_r = 9.0;
    draw_circle(x - eye_dx, eye_y, eye_r + 1.8, body_col);
    draw_circle(x + eye_dx, eye_y, eye_r + 1.8, body_col);
    draw_circle(x - eye_dx, eye_y, eye_r, tint(250, 250, 250, 255));
    draw_circle(x + eye_dx, eye_y, eye_r, tint(250, 250, 250, 255));
    let pupil_off = face_dir.clamp(-1.0, 1.0) * 2.5;
    draw_circle(x - eye_dx + pupil_off, eye_y + 1.5, 4.5, tint(20, 20, 30, 255));
    draw_circle(x + eye_dx + pupil_off, eye_y + 1.5, 4.5, tint(20, 20, 30, 255));
    draw_circle(x - eye_dx + pupil_off + 1.5, eye_y, 1.5, tint(255, 255, 255, 230));
    draw_circle(x + eye_dx + pupil_off + 1.5, eye_y, 1.5, tint(255, 255, 255, 230));
    // Mouth
    let smile_col = tint(30, 90, 50, 255);
    draw_arc(x, y + ry * 0.2, rx * 0.45, 0.15 * PI, 0.85 * PI, 2.5, smile_col);
}

fn draw_arc(cx: f32, cy: f32, r: f32, start: f32, end: f32, thickness: f32, color: Color) {
    let n = 14;
    let mut prev_x = cx + r * start.cos();
    let mut prev_y = cy + r * start.sin();
    for i in 1..=n {
        let t = start + (end - start) * i as f32 / n as f32;
        let xx = cx + r * t.cos();
        let yy = cy + r * t.sin();
        draw_line(prev_x, prev_y, xx, yy, thickness, color);
        prev_x = xx;
        prev_y = yy;
    }
}

fn draw_ellipse_filled(cx: f32, cy: f32, rx: f32, ry: f32, color: Color) {
    let n = 24;
    let center = Vec2::new(cx, cy);
    let mut prev = Vec2::new(cx + rx, cy);
    for i in 1..=n {
        let angle = TAU * i as f32 / n as f32;
        let p = Vec2::new(cx + rx * angle.cos(), cy + ry * angle.sin());
        draw_triangle(center, prev, p, color);
        prev = p;
    }
}

fn draw_ellipse_lines(cx: f32, cy: f32, rx: f32, ry: f32, thickness: f32, color: Color) {
    let n = 28;
    let mut prev = Vec2::new(cx + rx, cy);
    for i in 1..=n {
        let angle = TAU * i as f32 / n as f32;
        let p = Vec2::new(cx + rx * angle.cos(), cy + ry * angle.sin());
        draw_line(prev.x, prev.y, p.x, p.y, thickness, color);
        prev = p;
    }
}

fn draw_car(x: f32, y: f32, w: f32, h: f32, color: Color, going_right: bool) {
    draw_rectangle(x + 3.0, y + h - 1.0, w, 4.0, Color::from_rgba(0, 0, 0, 110));
    let r = h * 0.18;
    draw_rectangle(x + r, y, w - 2.0 * r, h, color);
    draw_circle(x + r, y + h * 0.5, r * 1.2, color);
    draw_circle(x + w - r, y + h * 0.5, r * 1.2, color);
    let win_col = Color::from_rgba(180, 230, 250, 220);
    let win_y = y + h * 0.18;
    let win_h = h * 0.4;
    let win_w = w * 0.25;
    draw_rectangle(x + w * 0.18, win_y, win_w, win_h, win_col);
    draw_rectangle(x + w * 0.55, win_y, win_w, win_h, win_col);
    let wheel_r = h * 0.30;
    draw_circle(x + w * 0.22, y + h - 3.0, wheel_r, Color::from_rgba(30, 30, 30, 255));
    draw_circle(x + w * 0.78, y + h - 3.0, wheel_r, Color::from_rgba(30, 30, 30, 255));
    draw_circle(x + w * 0.22, y + h - 3.0, wheel_r * 0.45, Color::from_rgba(110, 110, 120, 255));
    draw_circle(x + w * 0.78, y + h - 3.0, wheel_r * 0.45, Color::from_rgba(110, 110, 120, 255));
    let hx = if going_right { x + w - 6.0 } else { x + 6.0 };
    draw_circle(hx, y + h * 0.35, 4.0, Color::from_rgba(255, 230, 110, 255));
    draw_circle(hx, y + h * 0.35, 2.0, Color::from_rgba(255, 255, 220, 255));
}

fn draw_log(x: f32, y: f32, w: f32, h: f32, color: Color, oval_emphasis: bool) {
    draw_rectangle(x + 3.0, y + h - 1.0, w, 4.0, Color::from_rgba(0, 0, 0, 110));
    let r = h * 0.5;
    if w > 2.0 * r { draw_rectangle(x + r, y, w - 2.0 * r, h, color); }
    draw_circle(x + r, y + r, r, color);
    draw_circle(x + w - r, y + r, r, color);
    let dark = Color::from_rgba(45, 28, 14, 200);
    let grooves = ((w / 32.0) as i32).max(3);
    for i in 1..grooves {
        let gx = x + (w * i as f32 / grooves as f32);
        draw_line(gx, y + 4.0, gx, y + h - 4.0, 1.8, dark);
    }
    draw_circle_lines(x + r, y + r, r * 0.55, 1.5, dark);
    draw_circle_lines(x + r, y + r, r * 0.32, 1.2, dark);
    draw_circle_lines(x + w - r, y + r, r * 0.55, 1.5, dark);
    draw_circle_lines(x + w - r, y + r, r * 0.32, 1.2, dark);
    if oval_emphasis {
        draw_ellipse_lines(x + w * 0.5, y + h * 0.5, w * 0.5 - 2.0, h * 0.45, 2.5,
            Color::from_rgba(255, 240, 130, 220));
    }
}

fn draw_star(cx: f32, cy: f32, r_outer: f32, color: Color) {
    let r_inner = r_outer * 0.42;
    let n = 5usize;
    let mut verts: Vec<Vec2> = Vec::with_capacity(n * 2);
    for i in 0..(n * 2) {
        let angle = PI * i as f32 / n as f32 - PI / 2.0;
        let r = if i % 2 == 0 { r_outer } else { r_inner };
        verts.push(Vec2::new(cx + r * angle.cos(), cy + r * angle.sin()));
    }
    let center = Vec2::new(cx, cy);
    for i in 0..(n * 2) {
        draw_triangle(center, verts[i], verts[(i + 1) % (n * 2)], color);
    }
}

fn draw_heart(cx: f32, cy: f32, r: f32, color: Color) {
    let lobe_r = r * 0.55;
    let lobe_off_x = r * 0.42;
    let lobe_off_y = r * 0.15;
    draw_circle(cx - lobe_off_x, cy - lobe_off_y, lobe_r, color);
    draw_circle(cx + lobe_off_x, cy - lobe_off_y, lobe_r, color);
    draw_triangle(
        Vec2::new(cx - r * 0.95, cy + lobe_off_y * 0.5),
        Vec2::new(cx + r * 0.95, cy + lobe_off_y * 0.5),
        Vec2::new(cx, cy + r * 1.05),
        color,
    );
}


// ─────────────────────────────────────────────────────────────────────────────
//  Tests
// ─────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lane_y_is_centered_within_each_lane() {
        for lane in 0..LANES {
            let cy = FrogLane::lane_y(lane);
            let top = cy - LANE_H * 0.5;
            let bot = cy + LANE_H * 0.5;
            assert!(top >= LANE_TOP - 0.001, "lane {} top {} above LANE_TOP {}", lane, top, LANE_TOP);
            assert!(bot <= LANE_TOP + LANES as f32 * LANE_H + 0.001,
                "lane {} bot {} below LANE_TOP+lanes {}", lane, bot, LANE_TOP + LANES as f32 * LANE_H);
        }
    }

    #[test]
    fn lesson_tile_grid_holds_twelve_lessons_without_overlap() {
        let mut rects = Vec::new();
        for i in 0..12 {
            let r = lesson_tile_rect(i);
            assert!(r.x >= 0.0 && r.x + r.w <= SW, "tile {} overflows width: {:?}", i, r);
            assert!(r.y >= 0.0 && r.y + r.h <= SH, "tile {} overflows height: {:?}", i, r);
            for prev in &rects {
                let prev_r: Rect = *prev;
                let separated = r.x + r.w <= prev_r.x
                    || prev_r.x + prev_r.w <= r.x
                    || r.y + r.h <= prev_r.y
                    || prev_r.y + prev_r.h <= r.y;
                assert!(separated, "tile {} overlaps {:?}", i, prev_r);
            }
            rects.push(r);
        }
    }
}
