mod assets;
mod enemy;
mod levels;
mod player;
mod question;
mod ui;

use enemy::{EnemyGrid, Explosion};
use levels::Grade;
use macroquad::prelude::*;
use player::{Bullet, EnemyBullet, Player};
use question::{generate_question, Question};

const SCREEN_W: f32 = 800.0;
const SCREEN_H: f32 = 600.0;

fn window_conf() -> Conf {
    Conf {
        window_title: "Star Crusher".to_string(),
        window_width: SCREEN_W as i32,
        window_height: SCREEN_H as i32,
        window_resizable: false,
        ..Default::default()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GameMode {
    Title,
    Playing,
    GateIntro,
    GateQuestion,
    GameOver,
    Victory,
}

struct Game {
    mode: GameMode,
    grade: Grade,
    wave: usize,
    score: u32,
    lives: u8,
    player: Player,
    enemies: EnemyGrid,
    player_bullets: Vec<Bullet>,
    enemy_bullets: Vec<EnemyBullet>,
    explosions: Vec<Explosion>,
    active_question: Question,
    gate_question: Question,
    gate_answer: String,
    gate_feedback: Option<(bool, f64)>,
    gates_remaining: usize,
    last_enemy_fire: f64,
}

impl Game {
    fn new() -> Self {
        let grade = Grade::Preschool;
        let active_question = generate_question(grade);
        let enemies = EnemyGrid::new(grade, &grade.config(), SCREEN_W, Some(&active_question));

        Self {
            mode: GameMode::Title,
            grade,
            wave: 1,
            score: 0,
            lives: 3,
            player: Player::new(SCREEN_W),
            enemies,
            player_bullets: Vec::new(),
            enemy_bullets: Vec::new(),
            explosions: Vec::new(),
            active_question,
            gate_question: generate_question(grade),
            gate_answer: String::new(),
            gate_feedback: None,
            gates_remaining: grade.config().question_gate_count,
            last_enemy_fire: 0.0,
        }
    }

    fn reset(&mut self) {
        *self = Self::new();
        self.start_game();
    }

    fn start_game(&mut self) {
        self.mode = GameMode::Playing;
        self.spawn_wave();
    }

    fn spawn_wave(&mut self) {
        self.active_question = generate_question(self.grade);
        self.enemies = EnemyGrid::new(
            self.grade,
            &self.grade.config(),
            SCREEN_W,
            Some(&self.active_question),
        );
        self.player = Player::new(SCREEN_W);
        self.player_bullets.clear();
        self.enemy_bullets.clear();
        self.explosions.clear();
        self.last_enemy_fire = get_time();
    }

    fn begin_gate(&mut self) {
        self.gates_remaining = self.grade.config().question_gate_count;
        self.next_gate_question();
        self.mode = GameMode::GateIntro;
    }

    fn next_gate_question(&mut self) {
        self.gate_question = generate_question(self.grade);
        self.gate_answer.clear();
        self.gate_feedback = None;
    }

    fn update(&mut self) {
        match self.mode {
            GameMode::Title => {
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    self.reset();
                }
            }
            GameMode::Playing => self.update_playing(),
            GameMode::GateIntro => {
                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    self.mode = GameMode::GateQuestion;
                }
            }
            GameMode::GateQuestion => self.update_gate_question(),
            GameMode::GameOver | GameMode::Victory => {
                if is_key_pressed(KeyCode::Enter) {
                    self.reset();
                }
            }
        }
    }

    fn update_playing(&mut self) {
        self.player.update(SCREEN_W);

        if is_key_pressed(KeyCode::Space) && self.player_bullets.len() < 3 {
            self.player_bullets
                .push(Bullet::new(self.player.center_x(), self.player.top_y()));
        }

        if self.enemies.update() {
            self.mode = GameMode::GameOver;
            return;
        }

        self.update_enemy_fire();
        self.update_player_bullets();
        self.update_enemy_bullets();
        self.explosions.retain_mut(|explosion| !explosion.update());

        if self.enemies.is_cleared() {
            self.score += 100 * (self.grade.index() as u32 + 1);
            self.begin_gate();
        }
    }

    fn update_enemy_fire(&mut self) {
        let interval = self.enemies.fire_interval_ms as f64 / 1000.0;
        if get_time() - self.last_enemy_fire >= interval {
            if let Some((_, enemy)) = self.enemies.random_alive_enemy() {
                self.enemy_bullets
                    .push(EnemyBullet::new(enemy.center_x(), enemy.bottom_y()));
            }
            self.last_enemy_fire = get_time();
        }
    }

    fn update_player_bullets(&mut self) {
        let mut next_bullets = Vec::new();
        for mut bullet in self.player_bullets.drain(..) {
            if bullet.update() {
                continue;
            }

            if let Some((enemy_idx, is_correct)) = self.enemies.check_bullet_hit(
                bullet.x,
                bullet.y,
                Some(self.active_question.correct_answer),
            ) {
                let enemy = &self.enemies.enemies[enemy_idx];
                self.explosions
                    .push(Explosion::new(enemy.center_x(), enemy.bottom_y()));

                if is_correct {
                    self.enemies.kill_enemy(enemy_idx);
                    self.score += 10 + self.grade.index() as u32 * 5;
                } else {
                    self.lives = self.lives.saturating_sub(1);
                }
            } else {
                next_bullets.push(bullet);
            }
        }
        self.player_bullets = next_bullets;
    }

    fn update_enemy_bullets(&mut self) {
        let mut next_bullets = Vec::new();
        for mut bullet in self.enemy_bullets.drain(..) {
            if bullet.update() {
                continue;
            }

            if self.player.contains_point(bullet.x, bullet.y) {
                self.explosions.push(Explosion::new(bullet.x, bullet.y));
                self.lives = self.lives.saturating_sub(1);
            } else {
                next_bullets.push(bullet);
            }
        }
        self.enemy_bullets = next_bullets;

        if self.lives == 0 {
            self.mode = GameMode::GameOver;
        }
    }

    fn update_gate_question(&mut self) {
        while let Some(ch) = get_char_pressed() {
            if ch.is_ascii_digit() || (ch == '-' && self.gate_answer.is_empty()) {
                self.gate_answer.push(ch);
            }
        }

        if is_key_pressed(KeyCode::Backspace) {
            self.gate_answer.pop();
        }

        if is_key_pressed(KeyCode::Enter) && !self.gate_answer.is_empty() {
            let answer = self.gate_answer.parse::<i64>().ok();
            let is_correct = answer == Some(self.gate_question.correct_answer);
            self.gate_feedback = Some((is_correct, get_time()));

            if is_correct {
                self.score += 50 * (self.grade.index() as u32 + 1);
                self.gates_remaining = self.gates_remaining.saturating_sub(1);

                if self.gates_remaining == 0 {
                    self.advance_grade_or_finish();
                } else {
                    self.next_gate_question();
                }
            } else {
                self.gate_answer.clear();
            }
        }
    }

    fn advance_grade_or_finish(&mut self) {
        if let Some(next_grade) = self.grade.next() {
            self.grade = next_grade;
            self.wave += 1;
            self.spawn_wave();
            self.mode = GameMode::Playing;
        } else {
            self.mode = GameMode::Victory;
        }
    }

    fn draw(&self) {
        clear_background(BLACK);

        match self.mode {
            GameMode::Title => ui::draw_title_screen(),
            GameMode::Playing => self.draw_playing(),
            GameMode::GateIntro => {
                self.draw_playing();
                ui::draw_question_gate(&self.grade, self.grade.math_topics());
            }
            GameMode::GateQuestion => self.draw_gate_question(),
            GameMode::GameOver => {
                self.draw_playing();
                ui::draw_game_over(self.score, &self.grade);
            }
            GameMode::Victory => ui::draw_victory_screen(self.score),
        }
    }

    fn draw_playing(&self) {
        draw_starfield();
        assets::draw_border(SCREEN_W, SCREEN_H);
        self.enemies.draw(self.grade.enemy_color());
        self.player.draw();

        for bullet in &self.player_bullets {
            bullet.draw();
        }
        for bullet in &self.enemy_bullets {
            bullet.draw();
        }
        for explosion in &self.explosions {
            explosion.draw();
        }

        ui::draw_hud(
            &self.grade,
            self.score,
            self.lives,
            self.wave,
            Some(&self.active_question.text),
        );
    }

    fn draw_gate_question(&self) {
        draw_starfield();
        assets::draw_border(SCREEN_W, SCREEN_H);
        ui::draw_question_gate(&self.grade, self.grade.math_topics());

        let question_lines: Vec<&str> = self.gate_question.text.lines().collect();
        for (i, line) in question_lines.iter().enumerate() {
            let text_size = 26;
            let metrics = measure_text(line, None, text_size, 1.0);
            draw_text(
                line,
                SCREEN_W / 2.0 - metrics.width / 2.0,
                430.0 + i as f32 * 28.0,
                text_size as f32,
                YELLOW,
            );
        }

        ui::draw_answer_input(&self.gate_answer);
        if let Some((is_correct, _)) = self.gate_feedback {
            ui::draw_answer_feedback(is_correct);
        }
    }
}

fn draw_starfield() {
    for i in 0..90 {
        let x = ((i * 73 + 19) % SCREEN_W as i32) as f32;
        let y = ((i * 41 + 37) % 560) as f32;
        assets::draw_star(x, y, 0.6 + (i % 3) as f32 * 0.4);
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new();

    loop {
        game.update();
        game.draw();
        next_frame().await;
    }
}
