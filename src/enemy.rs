use crate::assets;
use crate::levels::{Grade, LevelConfig};
use crate::question::Question;
use ::rand::Rng;
use macroquad::prelude::*;

const ENEMY_MIN_Y: f32 = 145.0;
const ENEMY_MAX_Y: f32 = 430.0;

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

    /// Draws this enemy at current position with the given color and scale.
    pub fn draw(&self, grade_color: Color) {
        if !self.alive {
            return;
        }
        match &self.r#type {
            EnemyType::Standard => {
                assets::draw_enemy_invader(self.x, self.y, grade_color, 1.0);
            }
            EnemyType::Puzzle(num) => {
                assets::draw_puzzle_enemy(self.x, self.y, grade_color, 1.0, *num);
            }
        }
    }

    /// Checks if a bullet hits this enemy's hitbox.
    pub fn is_hit(&self, bx: f32, by: f32) -> bool {
        !self.alive
            || (bx >= self.x - 4.0
                && bx <= self.x + self.width + 4.0
                && by >= self.y - 4.0
                && by <= self.y + self.height + 4.0)
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
        let mut rng = ::rand::thread_rng();
        let target_count = config.rows * config.cols;

        let mut enemies = Vec::new();
        for index in 0..target_count {
            let band = index % 4;
            let x = rng.gen_range(40.0..screen_w - 84.0);
            let y = ENEMY_MIN_Y + band as f32 * 70.0 + rng.gen_range(0.0..32.0);
            let direction = if rng.gen_bool(0.5) { 1.0 } else { -1.0 };
            let velocity_x = direction * rng.gen_range(0.45..1.15);
            let velocity_y = rng.gen_range(-0.18..0.18);

            enemies.push(Enemy::new(x, y, velocity_x, velocity_y));
        }

        let mut grid = EnemyGrid {
            enemies,
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

        let mut rng = ::rand::thread_rng();
        let correct_slot = rng.gen_range(0..alive_indices.len());

        for (slot, enemy_index) in alive_indices.iter().enumerate() {
            let number = if slot == correct_slot {
                question.correct_answer
            } else if question.wrong_answers.is_empty() {
                question.correct_answer + slot as i64 + 1
            } else {
                let wrong_index = rng.gen_range(0..question.wrong_answers.len());
                question.wrong_answers[wrong_index]
            };

            self.enemies[*enemy_index].r#type = EnemyType::Puzzle(number);
        }
    }

    /// Updates enemy positions each frame. Returns true if any enemy reached the player zone.
    pub fn update(&mut self) -> bool {
        let effective_speed = self.base_speed * self.speed_mult.max(0.5);

        for e in &mut self.enemies {
            if !e.alive {
                continue;
            }

            e.x += e.velocity_x * effective_speed;
            e.y += e.velocity_y * effective_speed;

            if e.x > 980.0 {
                e.x = -e.width;
            } else if e.x + e.width < 0.0 {
                e.x = 980.0;
            }

            if e.y < ENEMY_MIN_Y || e.y > ENEMY_MAX_Y {
                e.velocity_y *= -1.0;
                e.y = e.y.clamp(ENEMY_MIN_Y, ENEMY_MAX_Y);
            }

            if e.y + e.height > 480.0 {
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
        }
    }

    /// Returns a random alive enemy for firing (or None).
    pub fn random_alive_enemy(&self) -> Option<(usize, &Enemy)> {
        let mut rng = ::rand::thread_rng();
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
        let idx = rng.gen_range(0..alive_indices.len());
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
                    EnemyType::Puzzle(n) => correct_answer.map_or(false, |ca| *n == ca),
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
        self.progress += 0.04; // Animation speed
        self.progress >= 1.0
    }

    /// Draws the explosion at current progress.
    pub fn draw(&self) {
        assets::draw_explosion(self.x, self.y, self.progress);
    }
}
