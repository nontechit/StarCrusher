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
const MOBILE_ENEMY_MIN_Y: f32 = 382.0;
const MOBILE_ENEMY_MAX_Y: f32 = 650.0;
const MOBILE_ENEMY_PLAYER_ZONE_Y: f32 = 1006.0;
const MOBILE_ENEMY_KILL_DROP: f32 = 34.0;

/// Type of enemy: standard invader or puzzle type showing an answer.
#[derive(Debug, Clone)]
pub enum EnemyType {
    Standard,
    Puzzle(i64), // Shows this number as the answer option
    ShapePuzzle { shape: CountShape, number: i64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CountShape {
    Circle,
    Heart,
    Rectangle,
    Square,
    Star,
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
        Enemy {
            x,
            y,
            width: 58.0,
            height: 50.0,
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
            EnemyType::ShapePuzzle { shape, number } => {
                assets::draw_shape_puzzle_enemy(
                    self.x,
                    self.y,
                    grade_color,
                    scale,
                    *shape,
                    *number,
                );
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
        let target_count = target_count_for_question(config, active_question);
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
        if let Some((shape, count)) = count_shape_from_question(question) {
            self.assign_shape_answers(question, shape, count);
            return;
        }

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

            self.enemies[*enemy_index].r#type =
                if let Some((shape, _)) = count_shape_from_question(question) {
                    EnemyType::ShapePuzzle { shape, number }
                } else {
                    EnemyType::Puzzle(number)
                };
        }
    }

    fn assign_shape_answers(&mut self, question: &Question, shape: CountShape, count: i64) {
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
        let mut numbers = vec![question.correct_answer];
        for wrong in &question.wrong_answers {
            if numbers.len() >= alive_indices.len() {
                break;
            }
            if *wrong != question.correct_answer && !numbers.contains(wrong) {
                numbers.push(*wrong);
            }
        }

        let mut candidate = 1;
        while numbers.len() < alive_indices.len() {
            if candidate != question.correct_answer && !numbers.contains(&candidate) {
                numbers.push(candidate);
            }
            candidate += 1;
        }

        let mut wrong_cursor = 1;
        for (slot, enemy_index) in alive_indices.iter().enumerate() {
            let number = if slot == correct_slot {
                question.correct_answer
            } else {
                while wrong_cursor < numbers.len()
                    && numbers[wrong_cursor] == question.correct_answer
                {
                    wrong_cursor += 1;
                }
                let value = numbers
                    .get(wrong_cursor)
                    .copied()
                    .unwrap_or((count + slot as i64 + 1).max(1));
                wrong_cursor += 1;
                value
            };

            self.enemies[*enemy_index].r#type = EnemyType::ShapePuzzle { shape, number };
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

    pub fn clear_all(&mut self) {
        for enemy in &mut self.enemies {
            enemy.alive = false;
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
                    EnemyType::ShapePuzzle { number, .. } => correct_answer == Some(*number),
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

fn target_count_for_question(config: &LevelConfig, active_question: Option<&Question>) -> usize {
    active_question
        .and_then(count_shape_from_question)
        .map(|(_, count)| count.max(1) as usize)
        .unwrap_or(config.rows * config.cols)
}

pub fn count_shape_from_question(question: &Question) -> Option<(CountShape, i64)> {
    let text = question.text.to_ascii_lowercase();
    if !text.starts_with("how many ") {
        return None;
    }

    let shape = if text.contains("heart") {
        CountShape::Heart
    } else if text.contains("circle") {
        CountShape::Circle
    } else if text.contains("rectangle") {
        CountShape::Rectangle
    } else if text.contains("square") {
        CountShape::Square
    } else if text.contains("star") {
        CountShape::Star
    } else {
        return None;
    };

    Some((shape, question.correct_answer.clamp(1, 9)))
}

pub fn question_uses_visual_count(question: &Question) -> bool {
    count_shape_from_question(question).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_question(text: &str, correct_answer: i64) -> Question {
        Question {
            text: text.to_string(),
            correct_answer,
            wrong_answers: vec![1, 2, 3, 6],
        }
    }

    #[test]
    fn shape_count_questions_spawn_matching_shape_targets() {
        let question = sample_question("How many stars?", 4);

        assert_eq!(
            target_count_for_question(&Grade::Preschool.config(), Some(&question)),
            4
        );
        assert!(question_uses_visual_count(&question));
        assert_eq!(
            count_shape_from_question(&question),
            Some((CountShape::Star, 4))
        );

        let mut grid = EnemyGrid {
            enemies: vec![Enemy::new(0.0, 0.0, 0.0, 0.0)],
            screen_w: 800.0,
            speed_mult: 1.0,
            base_speed: 0.5,
            fire_interval_ms: 1000,
        };
        grid.assign_answers(&question);

        let mut correct_targets = 0;
        for enemy in &grid.enemies {
            assert_eq!(enemy.width, 58.0);
            assert_eq!(enemy.height, 50.0);
            match enemy.r#type {
                EnemyType::ShapePuzzle { shape, number } => {
                    assert_eq!(shape, CountShape::Star);
                    if number == 4 {
                        correct_targets += 1;
                    }
                }
                _ => panic!("expected shaped puzzle target"),
            }
        }

        assert_eq!(correct_targets, 1);
    }

    #[test]
    fn every_grade_uses_large_numbered_puzzle_targets() {
        let grades = [
            Grade::Preschool,
            Grade::Kindergarten,
            Grade::FirstGrade,
            Grade::SecondGrade,
            Grade::ThirdGrade,
            Grade::FourthGrade,
            Grade::FifthGrade,
        ];
        let question = sample_question("3 + 2 = ?", 5);

        for grade in grades {
            let config = grade.config();
            let mut grid = EnemyGrid {
                enemies: vec![Enemy::new(0.0, 0.0, 0.0, 0.0)],
                screen_w: 1200.0,
                speed_mult: config.enemy_move_speed,
                base_speed: 0.5,
                fire_interval_ms: config.fire_interval_ms,
            };
            grid.assign_answers(&question);

            assert_eq!(
                target_count_for_question(&config, Some(&question)),
                config.rows * config.cols
            );
            assert!(grid.enemies.iter().all(|enemy| enemy.width == 58.0));
            assert!(grid.enemies.iter().all(|enemy| enemy.height == 50.0));
            assert!(grid
                .enemies
                .iter()
                .all(|enemy| matches!(enemy.r#type, EnemyType::Puzzle(_))));
            assert_eq!(
                grid.enemies
                    .iter()
                    .filter(|enemy| matches!(enemy.r#type, EnemyType::Puzzle(5)))
                    .count(),
                1,
                "{} should have exactly one correct target",
                grade.display_name()
            );
        }
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
