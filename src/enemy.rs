use crate::assets;
use crate::levels::{Grade, LevelConfig};
use crate::question::Question;
use crate::random;
use crate::screen;
use macroquad::prelude::*;

const ENEMY_MIN_Y: f32 = 132.0;
const ENEMY_MAX_Y: f32 = 410.0;
const ENEMY_PLAYER_ZONE_Y: f32 = 530.0;
const ENEMY_KILL_DROP: f32 = 14.0;
const MOBILE_ENEMY_MIN_Y: f32 = 412.0;
const MOBILE_ENEMY_MAX_Y: f32 = 680.0;
const MOBILE_ENEMY_PLAYER_ZONE_Y: f32 = 1062.0;
const MOBILE_ENEMY_KILL_DROP: f32 = 34.0;

/// Type of enemy: standard invader or puzzle type showing an answer.
#[derive(Debug, Clone)]
pub enum EnemyType {
    Standard,
    Puzzle(i64), // Shows this number as the answer option
}

/// A single enemy in the grid.
#[derive(Debug, Clone)]
pub struct Enemy {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub alive: bool,
    pub r#type: EnemyType,
}

impl Enemy {
    /// Creates a new enemy at the given grid position.
    fn new(x: f32, y: f32, velocity_x: f32, velocity_y: f32) -> Self {
        let w = 44.0;
        Enemy {
            x,
            y,
            width: w,
            height: 34.0,
            velocity_x,
            velocity_y,
            alive: true,
            r#type: EnemyType::Standard,
        }
    }

    fn draw_scale(&self) -> f32 {
        screen::portrait_gameplay_scale()
    }

    fn scaled_width(&self) -> f32 {
        self.width * self.draw_scale()
    }

    fn scaled_height(&self) -> f32 {
        self.height * self.draw_scale()
    }

    /// Draws this enemy at current position with the given color and scale.
    pub fn draw(&self, grade_color: Color) {
        if !self.alive {
            return;
        }
        let scale = self.draw_scale();
        match &self.r#type {
            EnemyType::Standard => {
                assets::draw_enemy_invader(self.x, self.y, grade_color, scale);
            }
            EnemyType::Puzzle(num) => {
                assets::draw_puzzle_enemy(self.x, self.y, grade_color, scale, *num);
            }
        }
    }

    /// Checks if a bullet hits this enemy's hitbox.
    pub fn is_hit(&self, bx: f32, by: f32) -> bool {
        if !self.alive {
            return false;
        }
        let scale = self.draw_scale();
        let margin = 4.0 * scale;
        bx >= self.x - margin
            && bx <= self.x + self.scaled_width() + margin
            && by >= self.y - margin
            && by <= self.y + self.scaled_height() + margin
    }

    /// Returns the center X of this enemy (for spawning enemy bullets).
    pub fn center_x(&self) -> f32 {
        self.x + self.width / 2.0
    }

    /// Returns the bottom Y of this enemy (bullet spawns here).
    pub fn bottom_y(&self) -> f32 {
        self.y + self.height
    }
}

/// Manages Math Invaders targets with movement, firing, and puzzle assignment.
pub struct EnemyGrid {
    pub enemies: Vec<Enemy>,
    screen_w: f32,
    /// Horizontal speed multiplier (increases as fewer enemies remain).
    pub speed_mult: f32,
    /// Base movement pixels per frame.
    pub base_speed: f32,
    /// Minimum interval between enemy fire attempts (ms).
    pub fire_interval_ms: u64,
}

impl EnemyGrid {
    /// Creates a new enemy grid for the given grade level and current question.
    pub fn new(
        _grade: Grade,
        config: &LevelConfig,
        screen_w: f32,
        active_question: Option<&Question>,
    ) -> Self {
        let target_count = config.rows * config.cols;
        let min_y = enemy_min_y();

        let mut enemies = Vec::new();
        for index in 0..target_count {
            let band = index % 4;
            let x = random::f32_range(40.0, screen_w - 84.0);
            let y = min_y + band as f32 * 70.0 + random::f32_range(0.0, 32.0);
            let direction = if random::bool(0.5) { 1.0 } else { -1.0 };
            let velocity_x = direction * random::f32_range(0.45, 1.15);
            let velocity_y = random::f32_range(-0.18, 0.18);

            enemies.push(Enemy::new(x, y, velocity_x, velocity_y));
        }

        let mut grid = EnemyGrid {
            enemies,
            screen_w,
            speed_mult: config.enemy_move_speed,
            base_speed: 0.5,
            fire_interval_ms: config.fire_interval_ms,
        };

        if let Some(question) = active_question {
            grid.assign_answers(question);
        }

        grid
    }

    /// Assigns answer numbers to every alive enemy, with exactly one correct target.
    pub fn assign_answers(&mut self, question: &Question) {
        let alive_indices: Vec<usize> = self
            .enemies
            .iter()
            .enumerate()
            .filter(|(_, enemy)| enemy.alive)
            .map(|(index, _)| index)
            .collect();

        if alive_indices.is_empty() {
            return;
        }

        let correct_slot = random::usize_exclusive(alive_indices.len());

        for (slot, enemy_index) in alive_indices.iter().enumerate() {
            let number = if slot == correct_slot {
                question.correct_answer
            } else if question.wrong_answers.is_empty() {
                question.correct_answer + slot as i64 + 1
            } else {
                let wrong_index = random::usize_exclusive(question.wrong_answers.len());
                question.wrong_answers[wrong_index]
            };

            self.enemies[*enemy_index].r#type = EnemyType::Puzzle(number);
        }
    }

    /// Updates enemy positions each frame. Returns true if any enemy reached the player zone.
    pub fn update(&mut self) -> bool {
        let effective_speed = self.base_speed * self.speed_mult.max(0.5) * screen::frame_step();
        let min_y = enemy_min_y();
        let max_y = enemy_max_y();
        let player_zone_y = enemy_player_zone_y();

        for e in &mut self.enemies {
            if !e.alive {
                continue;
            }

            e.x += e.velocity_x * effective_speed;
            e.y += e.velocity_y * effective_speed;

            if e.x > self.screen_w + e.width {
                e.x = -e.width;
            } else if e.x + e.width < 0.0 {
                e.x = self.screen_w;
            }

            if e.y < min_y || e.y > max_y {
                e.velocity_y *= -1.0;
                e.y = e.y.clamp(min_y, max_y);
            }

            if e.y + e.height > player_zone_y {
                return true;
            }
        }

        false
    }

    /// Draws all alive enemies with the given grade color.
    pub fn draw(&self, grade_color: Color) {
        for e in &self.enemies {
            if !e.alive {
                continue;
            }
            e.draw(grade_color);
        }
    }

    /// Returns true if all enemies are dead (wave cleared).
    pub fn is_cleared(&self) -> bool {
        self.enemies.iter().all(|e| !e.alive)
    }

    /// Marks an enemy as dead at the given index.
    pub fn kill_enemy(&mut self, idx: usize) {
        if idx < self.enemies.len() {
            self.enemies[idx].alive = false;
            self.drop_alive_after_kill();
        }
    }

    fn drop_alive_after_kill(&mut self) {
        let max_y = enemy_max_y();
        let drop = enemy_kill_drop();
        for enemy in &mut self.enemies {
            if enemy.alive {
                enemy.y = (enemy.y + drop).min(max_y);
            }
        }
    }

    /// Returns a random alive enemy for firing (or None).
    pub fn random_alive_enemy(&self) -> Option<(usize, &Enemy)> {
        let alive_indices: Vec<usize> = self
            .enemies
            .iter()
            .enumerate()
            .filter(|(_, e)| e.alive)
            .map(|(i, _)| i)
            .collect();

        if alive_indices.is_empty() {
            return None;
        }
        let idx = random::usize_exclusive(alive_indices.len());
        Some((alive_indices[idx], &self.enemies[alive_indices[idx]]))
    }

    /// Returns the count of alive enemies.
    #[allow(dead_code)]
    pub fn alive_count(&self) -> usize {
        self.enemies.iter().filter(|e| e.alive).count()
    }

    /// Checks if a bullet hits any enemy. Returns (index, is_correct_answer) for puzzle enemies.
    pub fn check_bullet_hit(
        &mut self,
        bx: f32,
        by: f32,
        correct_answer: Option<i64>,
    ) -> Option<(usize, bool)> {
        for (i, e) in self.enemies.iter().enumerate() {
            if !e.alive {
                continue;
            }
            if e.is_hit(bx, by) {
                let is_correct = match &e.r#type {
                    EnemyType::Puzzle(n) => correct_answer == Some(*n),
                    _ => true, // Standard enemies always count as a hit
                };
                return Some((i, is_correct));
            }
        }
        None
    }

    /// Returns the lowest enemy in each column for smart firing (bottom-row targeting).
    #[allow(dead_code)]
    pub fn bottom_enemies(&self) -> Vec<(usize, &Enemy)> {
        let mut result = Vec::new();
        if self.enemies.is_empty() {
            return result;
        }

        // Group by approximate column based on starting X positions
        let _cols = (self.fire_interval_ms as usize).max(1); // Just use a heuristic
        for e in &self.enemies {
            if !e.alive {
                continue;
            }
            result.push((0, e)); // Simplified: just return all alive enemies
        }

        result.sort_by(|a, b| b.1.y.partial_cmp(&a.1.y).unwrap());
        result.truncate(5); // Limit to 5 potential shooters
        result
    }
}

fn enemy_min_y() -> f32 {
    if screen::portrait_layout() {
        MOBILE_ENEMY_MIN_Y
    } else {
        ENEMY_MIN_Y
    }
}

fn enemy_max_y() -> f32 {
    if screen::portrait_layout() {
        MOBILE_ENEMY_MAX_Y
    } else {
        ENEMY_MAX_Y
    }
}

fn enemy_player_zone_y() -> f32 {
    if screen::portrait_layout() {
        MOBILE_ENEMY_PLAYER_ZONE_Y
    } else {
        ENEMY_PLAYER_ZONE_Y
    }
}

fn enemy_kill_drop() -> f32 {
    if screen::portrait_layout() {
        MOBILE_ENEMY_KILL_DROP
    } else {
        ENEMY_KILL_DROP
    }
}

/// An explosion effect for destroyed enemies or hit bullets.
#[derive(Debug)]
pub struct Explosion {
    pub x: f32,
    pub y: f32,
    /// Progress from 0.0 (just started) to 1.0 (fully faded).
    pub progress: f32,
}

impl Explosion {
    pub fn new(x: f32, y: f32) -> Self {
        Explosion {
            x,
            y,
            progress: 0.0,
        }
    }

    /// Updates explosion animation each frame. Returns true when done (fully faded).
    pub fn update(&mut self) -> bool {
        self.progress += 0.04 * screen::frame_step();
        self.progress >= 1.0
    }

    /// Draws the explosion at current progress.
    pub fn draw(&self) {
        assets::draw_explosion(self.x, self.y, self.progress);
    }
}
