use crate::assets;
use crate::levels::{Grade, LevelConfig};
use crate::question::Question;
use ::rand::Rng;
use macroquad::prelude::*;

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
    pub alive: bool,
    pub r#type: EnemyType,
}

impl Enemy {
    /// Creates a new enemy at the given grid position.
    fn new(x: f32, y: f32) -> Self {
        let w = 24.0;
        Enemy {
            x,
            y,
            width: w,
            height: 18.0,
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
                assets::draw_puzzle_enemy(self.x, self.y, grade_color, 1.2, *num);
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

/// Manages a grid of enemies with movement, firing, and puzzle assignment.
pub struct EnemyGrid {
    pub enemies: Vec<Enemy>,
    /// Current horizontal direction: 1 = right, -1 = left.
    pub move_dir: i32,
    /// Horizontal speed multiplier (increases as fewer enemies remain).
    pub speed_mult: f32,
    /// Base movement pixels per frame.
    pub base_speed: f32,
    /// How far to drop when hitting screen edge.
    pub drop_amount: f32,
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

        // Calculate starting positions to center the grid on screen
        let total_width = (config.cols as f32) * 40.0;
        let start_x = (screen_w - total_width) / 2.0 + 8.0;
        let start_y = 60.0;

        let mut enemies = Vec::new();
        for row in 0..config.rows {
            for col in 0..config.cols {
                let x = start_x + (col as f32) * 40.0;
                let y = start_y + (row as f32) * 35.0;

                let mut enemy = Enemy::new(x, y);

                // Assign puzzle type if there's an active question and this enemy is chosen randomly
                if let Some(q) = active_question {
                    if rng.gen_bool(config.puzzle_enemy_chance as f64) {
                        // Pick either the correct answer or a wrong one
                        if rng.gen_bool(0.25) && !q.wrong_answers.is_empty() {
                            enemy.r#type = EnemyType::Puzzle(q.correct_answer);
                        } else if !q.wrong_answers.is_empty() {
                            let idx = rng.gen_range(0..q.wrong_answers.len());
                            enemy.r#type = EnemyType::Puzzle(q.wrong_answers[idx]);
                        }
                    }
                }

                enemies.push(enemy);
            }
        }

        // Ensure at least one puzzle enemy shows the correct answer if we have a question
        if active_question.is_some() && config.puzzle_enemy_chance > 0.0 {
            let has_correct = enemies.iter().any(|e| matches!(&e.r#type, EnemyType::Puzzle(n) if Some(*n as i64) == active_question.map(|q| q.correct_answer)));
            if !has_correct && config.puzzle_enemy_chance >= 0.15 {
                // Force one enemy to show the correct answer
                let idx = rng.gen_range(0..enemies.len());
                enemies[idx].r#type = EnemyType::Puzzle(active_question.unwrap().correct_answer);
            }
        }

        EnemyGrid {
            enemies,
            move_dir: 1,
            speed_mult: config.enemy_move_speed,
            base_speed: 0.5,
            drop_amount: config.enemy_drop_amount,
            fire_interval_ms: config.fire_interval_ms,
        }
    }

    /// Updates enemy positions each frame. Returns true if any enemy reached the player zone (game over condition).
    pub fn update(&mut self) -> bool {
        let alive_count = self.enemies.iter().filter(|e| e.alive).count();
        // Speed up as fewer enemies remain
        let total = self.enemies.len() as f32;
        let _survival_ratio = if total > 0.0 {
            (alive_count as f32) / total
        } else {
            1.0
        };

        // Calculate speed: faster when more are alive, slightly slower as they thin out but with a minimum
        let effective_speed = self.base_speed * self.speed_mult.max(0.5);

        // Check if any enemy needs to trigger edge bounce or drop
        let mut need_drop = false;
        for e in &self.enemies {
            if !e.alive {
                continue;
            }
            if (self.move_dir == 1 && e.x + e.width > 780.0) || (self.move_dir == -1 && e.x < 20.0)
            {
                need_drop = true;
                break;
            }
        }

        // Check if any enemy reached the player zone (y > 450)
        for e in &self.enemies {
            if !e.alive {
                continue;
            }
            if e.y + e.height > 480.0 {
                return true; // Enemies breached the defense line
            }
        }

        if need_drop {
            self.move_dir *= -1;
            for e in &mut self.enemies {
                if !e.alive {
                    continue;
                }
                e.y += self.drop_amount;
                e.x -= (self.move_dir as f32) * effective_speed; // Undo the horizontal move that caused edge hit
            }
        } else {
            for e in &mut self.enemies {
                if !e.alive {
                    continue;
                }
                e.x += (self.move_dir as f32) * effective_speed;
            }
        }

        false // No breach yet
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
