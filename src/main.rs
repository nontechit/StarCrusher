mod assets;
mod enemy;
mod levels;
mod math_pong;
mod player;
mod question;
mod random;
mod reading_snake;
mod screen;
mod ui;

use enemy::{EnemyGrid, Explosion};
use levels::Grade;
use macroquad::prelude::*;
use math_pong::{MathPong, MathPongAction};
use player::{Bullet, EnemyBullet, Player};
use question::{generate_question, Question};
use reading_snake::{custom_words_from_input, ReadingSnake, ReadingSnakeAction};
use screen::{enter_fullscreen, use_virtual_screen, window_conf, SCREEN_H, SCREEN_W};

const TITLE_MENU_ROW_LEFT: f32 = ui::TITLE_MENU_X + 22.0;
const TITLE_MENU_ROW_RIGHT: f32 = ui::TITLE_MENU_X + ui::TITLE_MENU_W - 22.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GameMode {
    Title,
    Playing,
    GateIntro,
    GateQuestion,
    GameOver,
    Victory,
    ReadingSnake,
    SpellingList,
    MathPong,
    AdventureIntro,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AdventureStep {
    MathInvaders,
    ReadingSnake,
    MathPong,
    NightmareSnake,
    MathInvadersProgression,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TitleMenuPage {
    Main,
    MiniGames,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TitleMenuOption {
    StartAdventure,
    PlayMiniGames,
    MathInvaders,
    MathPong,
    ReadingSnake,
    NightmareSnake,
    SpellingList,
}

impl TitleMenuOption {
    const MAIN: [Self; 3] = [
        Self::StartAdventure,
        Self::PlayMiniGames,
        Self::SpellingList,
    ];
    const MINI_GAMES: [Self; 3] = [Self::ReadingSnake, Self::MathPong, Self::NightmareSnake];

    fn menu_len(page: TitleMenuPage) -> usize {
        match page {
            TitleMenuPage::Main => Self::MAIN.len(),
            TitleMenuPage::MiniGames => Self::MINI_GAMES.len(),
        }
    }

    fn from_index(page: TitleMenuPage, index: usize) -> Self {
        match page {
            TitleMenuPage::Main => Self::MAIN[index % Self::MAIN.len()],
            TitleMenuPage::MiniGames => Self::MINI_GAMES[index % Self::MINI_GAMES.len()],
        }
    }
}

struct Game {
    mode: GameMode,
    title_menu_page: TitleMenuPage,
    title_selection: usize,
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
    spelling_input: String,
    gates_remaining: usize,
    last_enemy_fire: f64,
    reading_snake: ReadingSnake,
    math_pong: MathPong,
    intro_page: usize,
    adventure_active: bool,
    adventure_step: AdventureStep,
}

impl Game {
    fn new() -> Self {
        let grade = Grade::Preschool;
        let active_question = generate_question(grade);
        let enemies = EnemyGrid::new(grade, &grade.config(), SCREEN_W, Some(&active_question));

        Self {
            mode: GameMode::Title,
            title_menu_page: TitleMenuPage::Main,
            title_selection: 0,
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
            spelling_input: String::new(),
            gates_remaining: grade.config().question_gate_count,
            last_enemy_fire: 0.0,
            reading_snake: ReadingSnake::new(),
            math_pong: MathPong::new(),
            intro_page: 0,
            adventure_active: false,
            adventure_step: AdventureStep::MathInvaders,
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

    fn launch_title_option(&mut self, option: TitleMenuOption) {
        match option {
            TitleMenuOption::StartAdventure => {
                self.intro_page = 0;
                self.adventure_active = true;
                self.adventure_step = AdventureStep::MathInvaders;
                self.mode = GameMode::AdventureIntro;
            }
            TitleMenuOption::PlayMiniGames => {
                self.title_menu_page = TitleMenuPage::MiniGames;
                self.title_selection = 0;
            }
            TitleMenuOption::MathInvaders => self.reset(),
            TitleMenuOption::MathPong => {
                self.math_pong = MathPong::new();
                self.mode = GameMode::MathPong;
            }
            TitleMenuOption::ReadingSnake => {
                self.reading_snake = ReadingSnake::new();
                self.mode = GameMode::ReadingSnake;
            }
            TitleMenuOption::NightmareSnake => {
                self.reading_snake = ReadingSnake::new_nightmare();
                self.mode = GameMode::ReadingSnake;
            }
            TitleMenuOption::SpellingList => {
                self.spelling_input.clear();
                self.mode = GameMode::SpellingList;
            }
        }
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
                let menu_len = TitleMenuOption::menu_len(self.title_menu_page);
                if let Some(tap) = primary_tap_position() {
                    if let Some(index) = title_menu_index_at(tap, menu_len) {
                        self.title_selection = index;
                        self.launch_title_option(TitleMenuOption::from_index(
                            self.title_menu_page,
                            self.title_selection,
                        ));
                        return;
                    }
                }

                if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
                    self.title_selection = (self.title_selection + menu_len - 1) % menu_len;
                } else if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
                    self.title_selection = (self.title_selection + 1) % menu_len;
                }

                if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                    self.launch_title_option(TitleMenuOption::from_index(
                        self.title_menu_page,
                        self.title_selection,
                    ));
                } else if is_key_pressed(KeyCode::Escape)
                    && self.title_menu_page == TitleMenuPage::MiniGames
                {
                    self.title_menu_page = TitleMenuPage::Main;
                    self.title_selection = 1;
                } else if is_key_pressed(KeyCode::M) {
                    self.launch_title_option(TitleMenuOption::MathInvaders);
                } else if is_key_pressed(KeyCode::R) {
                    self.launch_title_option(TitleMenuOption::ReadingSnake);
                } else if is_key_pressed(KeyCode::N) {
                    self.launch_title_option(TitleMenuOption::NightmareSnake);
                } else if is_key_pressed(KeyCode::L) {
                    self.launch_title_option(TitleMenuOption::SpellingList);
                } else if is_key_pressed(KeyCode::P) {
                    if self.title_menu_page == TitleMenuPage::Main {
                        self.launch_title_option(TitleMenuOption::PlayMiniGames);
                    } else {
                        self.launch_title_option(TitleMenuOption::MathPong);
                    }
                }
            }
            GameMode::Playing => self.update_playing(),
            GameMode::GateIntro => {
                if is_key_pressed(KeyCode::Enter)
                    || is_key_pressed(KeyCode::Space)
                    || primary_tap_position().is_some()
                {
                    self.mode = GameMode::GateQuestion;
                }
            }
            GameMode::GateQuestion => self.update_gate_question(),
            GameMode::GameOver | GameMode::Victory => {
                if is_key_pressed(KeyCode::Enter) || primary_tap_position().is_some() {
                    self.reset();
                }
            }
            GameMode::ReadingSnake => match self.reading_snake.update() {
                ReadingSnakeAction::None => {}
                ReadingSnakeAction::ExitToTitle => self.exit_to_title(),
                ReadingSnakeAction::Completed => self.complete_reading_snake(),
            },
            GameMode::SpellingList => self.update_spelling_list(),
            GameMode::MathPong => match self.math_pong.update() {
                MathPongAction::None => {}
                MathPongAction::ExitToTitle => self.exit_to_title(),
                MathPongAction::Completed => self.complete_math_pong(),
            },
            GameMode::AdventureIntro => self.update_adventure_intro(),
        }
    }

    fn update_playing(&mut self) {
        self.player.update(SCREEN_W);

        let touch_fire = self.update_touch_player();
        if (is_key_pressed(KeyCode::Space) || touch_fire) && self.player_bullets.len() < 3 {
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
            if self.adventure_active && self.adventure_step == AdventureStep::MathInvaders {
                self.adventure_step = AdventureStep::ReadingSnake;
                self.reading_snake = ReadingSnake::new_adventure();
                self.mode = GameMode::ReadingSnake;
            } else {
                self.begin_gate();
            }
        }
    }

    fn exit_to_title(&mut self) {
        self.adventure_active = false;
        self.adventure_step = AdventureStep::MathInvaders;
        self.title_menu_page = TitleMenuPage::Main;
        self.mode = GameMode::Title;
    }

    fn complete_reading_snake(&mut self) {
        if !self.adventure_active {
            return;
        }

        match self.adventure_step {
            AdventureStep::ReadingSnake => {
                self.adventure_step = AdventureStep::MathPong;
                self.math_pong = MathPong::new();
                self.mode = GameMode::MathPong;
            }
            AdventureStep::NightmareSnake => {
                self.adventure_step = AdventureStep::MathInvadersProgression;
                self.begin_gate();
            }
            _ => {}
        }
    }

    fn complete_math_pong(&mut self) {
        if !self.adventure_active || self.adventure_step != AdventureStep::MathPong {
            return;
        }

        self.adventure_step = AdventureStep::NightmareSnake;
        self.reading_snake = ReadingSnake::new_adventure_nightmare();
        self.mode = GameMode::ReadingSnake;
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
                    if !self.enemies.is_cleared() {
                        self.active_question = generate_question(self.grade);
                        self.enemies.assign_answers(&self.active_question);
                    }
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
        let mut submit_answer = false;

        while let Some(ch) = get_char_pressed() {
            if ch.is_ascii_digit() || (ch == '-' && self.gate_answer.is_empty()) {
                self.gate_answer.push(ch);
            }
        }

        if is_key_pressed(KeyCode::Backspace) {
            self.gate_answer.pop();
        }

        if let Some(tap) = primary_tap_position() {
            match gate_key_at(tap) {
                Some(GateKey::Digit(digit)) => self.gate_answer.push(digit),
                Some(GateKey::Delete) => {
                    self.gate_answer.pop();
                }
                Some(GateKey::Submit) => submit_answer = true,
                None => {}
            }
        }

        if is_key_pressed(KeyCode::Enter) {
            submit_answer = true;
        }

        if submit_answer && !self.gate_answer.is_empty() {
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

    fn update_spelling_list(&mut self) {
        if is_key_pressed(KeyCode::Escape) {
            self.mode = GameMode::Title;
            return;
        }

        while let Some(ch) = get_char_pressed() {
            if ch.is_ascii_alphabetic()
                || ch == ' '
                || ch == ','
                || ch == ':'
                || ch == ';'
                || ch == '.'
                || ch == '\''
                || ch == '-'
                || ch == '\n'
                || ch == '\r'
            {
                self.spelling_input.push(ch);
            }
        }

        if is_key_pressed(KeyCode::Backspace) {
            self.spelling_input.pop();
        }

        if is_key_pressed(KeyCode::Enter)
            || is_key_pressed(KeyCode::N)
            || primary_tap_position().is_some()
        {
            let custom_words = custom_words_from_input(&self.spelling_input);
            self.reading_snake = if is_key_pressed(KeyCode::N) {
                ReadingSnake::new_nightmare_with_words(custom_words)
            } else {
                ReadingSnake::new_with_words(custom_words)
            };
            self.mode = GameMode::ReadingSnake;
        }
    }

    fn update_adventure_intro(&mut self) {
        let total_pages = ui::adventure_intro_page_count();

        if is_key_pressed(KeyCode::Enter)
            || is_key_pressed(KeyCode::Space)
            || primary_tap_position().is_some()
        {
            if self.intro_page + 1 >= total_pages {
                self.start_adventure_math_invaders();
            } else {
                self.intro_page += 1;
            }
        } else if is_key_pressed(KeyCode::Escape) {
            self.exit_to_title();
        }
    }

    fn update_touch_player(&mut self) -> bool {
        if let Some(pointer) = primary_pointer_position() {
            if pointer.y > 520.0 {
                self.player.x = (pointer.x - self.player.width / 2.0)
                    .clamp(8.0, SCREEN_W - self.player.width - 8.0);
            }
        }

        primary_tap_position().is_some_and(|tap| tap.y > 470.0)
    }

    fn start_adventure_math_invaders(&mut self) {
        let mut next = Self::new();
        next.adventure_active = true;
        next.adventure_step = AdventureStep::MathInvaders;
        *self = next;
        self.start_game();
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
        use_virtual_screen();
        clear_background(BLACK);

        match self.mode {
            GameMode::Title => ui::draw_title_screen(
                self.title_menu_page == TitleMenuPage::MiniGames,
                self.title_selection,
            ),
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
            GameMode::ReadingSnake => self.reading_snake.draw(),
            GameMode::SpellingList => ui::draw_spelling_list_screen(&self.spelling_input),
            GameMode::MathPong => self.math_pong.draw(),
            GameMode::AdventureIntro => ui::draw_adventure_intro(self.intro_page),
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
                455.0 + i as f32 * 28.0,
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

enum GateKey {
    Digit(char),
    Delete,
    Submit,
}

fn title_menu_index_at(point: Vec2, menu_len: usize) -> Option<usize> {
    if point.x < TITLE_MENU_ROW_LEFT || point.x > TITLE_MENU_ROW_RIGHT {
        return None;
    }

    (0..menu_len).find(|index| {
        let row_top = ui::TITLE_MENU_ROW_TOP
            + *index as f32 * (ui::TITLE_MENU_ROW_H + ui::TITLE_MENU_ROW_GAP);
        point.y >= row_top && point.y <= row_top + ui::TITLE_MENU_ROW_H
    })
}

fn gate_key_at(point: Vec2) -> Option<GateKey> {
    let labels = [
        GateKey::Digit('1'),
        GateKey::Digit('2'),
        GateKey::Digit('3'),
        GateKey::Digit('4'),
        GateKey::Digit('5'),
        GateKey::Digit('6'),
        GateKey::Digit('7'),
        GateKey::Digit('8'),
        GateKey::Digit('9'),
        GateKey::Delete,
        GateKey::Digit('0'),
        GateKey::Submit,
    ];

    for (index, key) in labels.into_iter().enumerate() {
        let col = index % 3;
        let row = index / 3;
        let x = ui::KEYPAD_X + col as f32 * (ui::KEYPAD_KEY + ui::KEYPAD_GAP);
        let y = ui::KEYPAD_Y + row as f32 * (ui::KEYPAD_KEY + ui::KEYPAD_GAP);
        if point.x >= x
            && point.x <= x + ui::KEYPAD_KEY
            && point.y >= y
            && point.y <= y + ui::KEYPAD_KEY
        {
            return Some(key);
        }
    }

    None
}

fn primary_tap_position() -> Option<Vec2> {
    for touch in touches() {
        if touch.phase == TouchPhase::Started {
            return Some(to_virtual_position(touch.position));
        }
    }

    if is_mouse_button_pressed(MouseButton::Left) {
        let (x, y) = mouse_position();
        return Some(to_virtual_position(vec2(x, y)));
    }

    None
}

fn primary_pointer_position() -> Option<Vec2> {
    for touch in touches() {
        if matches!(
            touch.phase,
            TouchPhase::Started | TouchPhase::Stationary | TouchPhase::Moved
        ) {
            return Some(to_virtual_position(touch.position));
        }
    }

    if is_mouse_button_down(MouseButton::Left) {
        let (x, y) = mouse_position();
        return Some(to_virtual_position(vec2(x, y)));
    }

    None
}

fn to_virtual_position(position: Vec2) -> Vec2 {
    vec2(
        position.x * SCREEN_W / screen_width().max(1.0),
        position.y * SCREEN_H / screen_height().max(1.0),
    )
}

fn draw_starfield() {
    for i in 0..90 {
        let x = ((i * 73 + 19) % SCREEN_W as i32) as f32;
        let y = ((i * 41 + 37) % (SCREEN_H as i32 - 40)) as f32;
        assets::draw_star(x, y, 0.6 + (i % 3) as f32 * 0.4);
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    enter_fullscreen();
    next_frame().await;
    enter_fullscreen();

    let mut game = Game::new();

    loop {
        game.update();
        game.draw();
        next_frame().await;
    }
}
