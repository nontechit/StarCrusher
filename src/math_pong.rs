use ::rand::seq::SliceRandom;
use macroquad::prelude::*;

use crate::levels::Grade;
use crate::question::{generate_question, Question};
use crate::screen::{SCREEN_H, SCREEN_W};

const PADDLE_Y: f32 = 708.0;
const TARGET_Y: f32 = 120.0;
const TARGET_W: f32 = 76.0;
const TARGET_H: f32 = 42.0;
const BALL_RADIUS: f32 = 8.0;
const QUESTIONS_PER_GRADE: u8 = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MathPongAction {
    None,
    ExitToTitle,
}

#[derive(Clone, Debug)]
struct Target {
    rect: Rect,
    value: i64,
    correct: bool,
    flash_until: f64,
}

pub struct MathPong {
    grade: Grade,
    question: Question,
    targets: Vec<Target>,
    paddle_x: f32,
    paddle_w: f32,
    ball_pos: Vec2,
    ball_vel: Vec2,
    ball_launched: bool,
    score: u32,
    lives: u8,
    questions_cleared: u8,
    message: &'static str,
    game_over: bool,
    victory: bool,
}

impl MathPong {
    pub fn new() -> Self {
        let grade = Grade::Preschool;
        let question = generate_question(grade);
        let mut game = Self {
            grade,
            question,
            targets: Vec::new(),
            paddle_x: SCREEN_W / 2.0 - 55.0,
            paddle_w: 110.0,
            ball_pos: vec2(SCREEN_W / 2.0, PADDLE_Y - 14.0),
            ball_vel: Vec2::ZERO,
            ball_launched: false,
            score: 0,
            lives: 3,
            questions_cleared: 0,
            message: "Aim the ball at the correct number.",
            game_over: false,
            victory: false,
        };
        game.spawn_targets();
        game
    }

    pub fn update(&mut self) -> MathPongAction {
        if is_key_pressed(KeyCode::Escape) {
            return MathPongAction::ExitToTitle;
        }

        if self.game_over || self.victory {
            if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                *self = Self::new();
            }
            return MathPongAction::None;
        }

        self.update_paddle();
        self.update_ball();
        MathPongAction::None
    }

    pub fn draw(&self) {
        clear_background(Color::new(0.03, 0.03, 0.09, 1.0));
        draw_starfield();
        self.draw_header();
        self.draw_targets();
        self.draw_paddle();
        self.draw_ball();
        self.draw_footer();

        if self.game_over || self.victory {
            draw_rectangle(
                0.0,
                0.0,
                SCREEN_W,
                SCREEN_H,
                Color::new(0.0, 0.0, 0.0, 0.76),
            );
            let title = if self.victory {
                "MATH PONG MASTERED"
            } else {
                "MATH PONG OVER"
            };
            let color = if self.victory { GREEN } else { RED };
            centered_text(title, 220.0, 42, color);
            centered_text(&format!("Final Score: {}", self.score), 280.0, 28, YELLOW);
            centered_text("Press ENTER to play again", 340.0, 22, WHITE);
            centered_text("Press ESC for title", 375.0, 18, GRAY);
        }
    }

    fn update_paddle(&mut self) {
        let speed = 7.0;
        if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
            self.paddle_x -= speed;
        }
        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
            self.paddle_x += speed;
        }

        self.paddle_x = self.paddle_x.clamp(12.0, SCREEN_W - self.paddle_w - 12.0);

        if !self.ball_launched {
            self.ball_pos = vec2(self.paddle_x + self.paddle_w / 2.0, PADDLE_Y - 14.0);
            if is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Enter) {
                let grade_speed = 4.8 + self.grade.index() as f32 * 0.35;
                self.ball_vel = vec2(0.0, -grade_speed);
                self.ball_launched = true;
                self.message = "Bounce into the correct number.";
            }
        }
    }

    fn update_ball(&mut self) {
        if !self.ball_launched {
            return;
        }

        self.ball_pos += self.ball_vel;

        if self.ball_pos.x - BALL_RADIUS <= 0.0 || self.ball_pos.x + BALL_RADIUS >= SCREEN_W {
            self.ball_vel.x *= -1.0;
            self.ball_pos.x = self.ball_pos.x.clamp(BALL_RADIUS, SCREEN_W - BALL_RADIUS);
        }
        if self.ball_pos.y - BALL_RADIUS <= 0.0 {
            self.ball_vel.y = self.ball_vel.y.abs();
            self.ball_pos.y = BALL_RADIUS;
        }

        let paddle = Rect::new(self.paddle_x, PADDLE_Y, self.paddle_w, 16.0);
        if self.ball_vel.y > 0.0 && circle_hits_rect(self.ball_pos, BALL_RADIUS, paddle) {
            let hit_ratio =
                ((self.ball_pos.x - self.paddle_x) / self.paddle_w - 0.5).clamp(-0.5, 0.5);
            let speed = self.ball_vel.length().max(5.0);
            self.ball_vel = vec2(hit_ratio * speed * 1.5, -speed.abs());
            self.ball_pos.y = PADDLE_Y - BALL_RADIUS;
        }

        if self.ball_pos.y - BALL_RADIUS > SCREEN_H {
            self.lose_life("Ball missed. Try a cleaner angle.");
            return;
        }

        if let Some(idx) = self
            .targets
            .iter()
            .position(|target| circle_hits_rect(self.ball_pos, BALL_RADIUS, target.rect))
        {
            if self.targets[idx].correct {
                self.correct_hit();
            } else {
                self.targets[idx].flash_until = get_time() + 0.35;
                self.ball_vel.y *= -1.0;
                self.lose_life("Wrong number. Read the question carefully.");
            }
        }
    }

    fn correct_hit(&mut self) {
        self.score += 100 + self.grade.index() as u32 * 25;
        self.questions_cleared += 1;
        if self.questions_cleared >= QUESTIONS_PER_GRADE {
            self.questions_cleared = 0;
            if let Some(next_grade) = self.grade.next() {
                self.grade = next_grade;
                self.paddle_w = (self.paddle_w - 7.0).max(72.0);
                self.message = "Grade up! The ball is getting faster.";
            } else {
                self.victory = true;
                return;
            }
        } else {
            self.message = "Correct! New target ready.";
        }

        self.question = generate_question(self.grade);
        self.spawn_targets();
        self.reset_ball();
    }

    fn lose_life(&mut self, message: &'static str) {
        self.lives = self.lives.saturating_sub(1);
        self.message = message;
        if self.lives == 0 {
            self.game_over = true;
        } else {
            self.reset_ball();
        }
    }

    fn reset_ball(&mut self) {
        self.ball_launched = false;
        self.ball_vel = Vec2::ZERO;
        self.ball_pos = vec2(self.paddle_x + self.paddle_w / 2.0, PADDLE_Y - 14.0);
    }

    fn spawn_targets(&mut self) {
        let mut answers = build_answer_choices(&self.question, 5);
        let mut rng = ::rand::thread_rng();
        answers.shuffle(&mut rng);
        let count = answers.len();
        let spacing = 28.0;
        let total_w = count as f32 * TARGET_W + count.saturating_sub(1) as f32 * spacing;
        let start_x = SCREEN_W / 2.0 - total_w / 2.0;

        self.targets = answers
            .into_iter()
            .take(count)
            .enumerate()
            .map(|(idx, value)| Target {
                rect: Rect::new(
                    start_x + idx as f32 * (TARGET_W + spacing),
                    TARGET_Y,
                    TARGET_W,
                    TARGET_H,
                ),
                value,
                correct: value == self.question.correct_answer,
                flash_until: 0.0,
            })
            .collect();
    }

    fn draw_header(&self) {
        centered_text("MATH PONG", 42.0, 38, Color::new(0.55, 0.85, 1.0, 1.0));
        centered_text(
            &format!(
                "{} | Question {}/{}",
                self.grade.display_name(),
                self.questions_cleared + 1,
                QUESTIONS_PER_GRADE
            ),
            72.0,
            18,
            WHITE,
        );
        draw_text(&format!("Score: {}", self.score), 24.0, 36.0, 22.0, YELLOW);
        draw_text(&format!("Lives: {}", self.lives), 680.0, 36.0, 22.0, WHITE);
    }

    fn draw_targets(&self) {
        for target in &self.targets {
            let color = if target.flash_until > get_time() {
                RED
            } else {
                Color::new(0.2, 0.5, 0.95, 1.0)
            };
            draw_rectangle(
                target.rect.x,
                target.rect.y,
                target.rect.w,
                target.rect.h,
                color,
            );
            draw_rectangle_lines(
                target.rect.x,
                target.rect.y,
                target.rect.w,
                target.rect.h,
                2.0,
                WHITE,
            );

            let text = target.value.to_string();
            let metrics = measure_text(&text, None, 28, 1.0);
            draw_text(
                &text,
                target.rect.x + target.rect.w / 2.0 - metrics.width / 2.0,
                target.rect.y + target.rect.h / 2.0 + metrics.height / 2.5,
                28.0,
                WHITE,
            );
        }
    }

    fn draw_paddle(&self) {
        draw_rectangle(
            self.paddle_x,
            PADDLE_Y,
            self.paddle_w,
            16.0,
            Color::new(0.3, 1.0, 0.75, 1.0),
        );
        draw_rectangle_lines(self.paddle_x, PADDLE_Y, self.paddle_w, 16.0, 2.0, WHITE);
    }

    fn draw_ball(&self) {
        draw_circle(
            self.ball_pos.x,
            self.ball_pos.y,
            BALL_RADIUS + 4.0,
            Color::new(1.0, 0.8, 0.2, 0.22),
        );
        draw_circle(self.ball_pos.x, self.ball_pos.y, BALL_RADIUS, YELLOW);
    }

    fn draw_footer(&self) {
        let lines: Vec<&str> = self.question.text.lines().collect();
        let box_h = 62.0 + (lines.len().saturating_sub(1) as f32 * 22.0);
        draw_rectangle(
            100.0,
            424.0,
            SCREEN_H,
            box_h,
            Color::new(0.05, 0.08, 0.18, 0.88),
        );
        draw_rectangle_lines(
            100.0,
            424.0,
            SCREEN_H,
            box_h,
            2.0,
            Color::new(0.4, 0.7, 1.0, 1.0),
        );

        for (idx, line) in lines.iter().enumerate() {
            centered_text(line, 452.0 + idx as f32 * 22.0, 22, YELLOW);
        }
        centered_text(self.message, 506.0, 18, WHITE);
        centered_text(
            "Move: Arrow Keys / A,D   Launch: Space/Enter   ESC: Title",
            586.0,
            16,
            GRAY,
        );
    }
}

fn build_answer_choices(question: &Question, count: usize) -> Vec<i64> {
    let count = count.max(1);
    let correct = question.correct_answer;
    let mut decoys = Vec::new();

    for wrong in &question.wrong_answers {
        if *wrong != correct && !decoys.contains(wrong) {
            decoys.push(*wrong);
        }
        if decoys.len() + 1 >= count {
            break;
        }
    }

    let mut offset = 1;
    while decoys.len() + 1 < count {
        for candidate in [correct + offset, correct - offset] {
            if candidate != correct && !decoys.contains(&candidate) {
                decoys.push(candidate);
            }
            if decoys.len() + 1 >= count {
                break;
            }
        }
        offset += 1;
    }

    let mut answers = Vec::with_capacity(count);
    answers.push(correct);
    answers.extend(decoys.into_iter().take(count - 1));
    answers
}

fn circle_hits_rect(center: Vec2, radius: f32, rect: Rect) -> bool {
    let closest_x = center.x.clamp(rect.x, rect.x + rect.w);
    let closest_y = center.y.clamp(rect.y, rect.y + rect.h);
    center.distance_squared(vec2(closest_x, closest_y)) <= radius * radius
}

fn draw_starfield() {
    for i in 0..70 {
        let x = ((i * 67 + 31) % SCREEN_W as i32) as f32;
        let y = ((i * 43 + 17) % SCREEN_H as i32) as f32;
        let brightness = (get_time() as f32 + x * 0.01).sin() * 0.25 + 0.65;
        draw_circle(
            x,
            y,
            1.0 + (i % 3) as f32 * 0.45,
            Color::new(brightness, brightness, 1.0, 0.75),
        );
    }
}

fn centered_text(text: &str, y: f32, font_size: u16, color: Color) {
    let metrics = measure_text(text, None, font_size, 1.0);
    draw_text(
        text,
        SCREEN_W / 2.0 - metrics.width / 2.0,
        y,
        font_size as f32,
        color,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kindergarten_number_question_has_one_correct_answer() {
        let question = Question {
            text: "What number is this?   3".to_string(),
            correct_answer: 3,
            wrong_answers: vec![1, 2, 4],
        };

        let answers = build_answer_choices(&question, 5);

        assert_eq!(answers.len(), 5);
        assert_eq!(answers.iter().filter(|answer| **answer == 3).count(), 1);
    }
}
