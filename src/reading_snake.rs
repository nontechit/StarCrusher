use crate::random;
use crate::screen::{self, primary_tap_position, to_virtual_position, SCREEN_H, SCREEN_W};
use crate::ui;
use macroquad::prelude::*;

const MOBILE_GRID_W: i32 = 14;
const MOBILE_GRID_H: i32 = 14;
const DESKTOP_GRID_W: i32 = 26;
const DESKTOP_GRID_H: i32 = 12;
const DESKTOP_PANEL_W: f32 = 1240.0;
const DESKTOP_PANEL_X: f32 = (SCREEN_W - DESKTOP_PANEL_W) / 2.0;
const DESKTOP_PANEL_TOP: f32 = 6.0;
const DESKTOP_HEADER_H: f32 = 76.0;
const DESKTOP_FOOTER_ZONE: f32 = 78.0;
const DESKTOP_BOARD_FRAME_BOTTOM: f32 = 10.0;
const DESKTOP_FOOTER_GAP: f32 = 28.0;
const DESKTOP_BOTTOM_CHROME: f32 = 0.0;
const DESKTOP_PANEL_BOTTOM_PAD: f32 = 8.0;
const MOBILE_CELL_W: f32 = 62.0;
const MOBILE_CELL_H: f32 = 23.0;
const MOBILE_BOARD_X: f32 = (SCREEN_W - MOBILE_GRID_W as f32 * MOBILE_CELL_W) / 2.0;
const MOBILE_GOAL_CARD_Y: f32 = 116.0;
const MOBILE_GOAL_TEXT_Y: f32 = 150.0;
const MOBILE_GOAL_TEXT_W: f32 = 940.0;
const MOBILE_GOAL_TEXT_SIZE: u16 = 28;
const MOBILE_GOAL_MAX_LINES: usize = 3;
const MOBILE_BOARD_GAP: f32 = 18.0;
const MOBILE_FOOTER_GAP: f32 = 24.0;
const MOBILE_FOOTER_H: f32 = 84.0;
const MOBILE_PLAYFIELD_BOTTOM: f32 = ui::MOBILE_ACTION_Y - 10.0;
const MOBILE_SWIPE_THRESHOLD: f32 = 34.0;

struct MobilePlayfieldLayout {
    header_bottom: f32,
    goal_card_h: f32,
    goal_text: String,
    board_y: f32,
    cell_h: f32,
}
const STEP_SECONDS: f64 = 0.25;
const SNAKE_HEAD_SAFE_RADIUS: i32 = 3;
const MAX_LIVES: u8 = 9;

const MAX_CUSTOM_WORDS: usize = 64;
const WORDS: &[(&str, &str, &str)] = &[
    ("KEY", "noun", "A small tool used to open a lock."),
    ("BUMPY", "adjective", "Not smooth; full of bumps."),
    ("PUPPY", "noun", "A young dog."),
    ("FUNNY", "adjective", "Causing laughter or amusement."),
    ("PENNY", "noun", "A coin worth one cent."),
    ("SANDY", "adjective", "Covered with or made of sand."),
    ("MY", "adjective", "Belonging to me."),
    ("NIGHT", "noun", "The dark time between sunset and morning."),
    ("WASH", "verb", "To clean with water."),
    (
        "WOULD",
        "helping verb",
        "Used to express a conditional action or hypothetical situation.",
    ),
    ("FOUND", "verb", "Discovered or located something."),
    ("HARD", "adjective", "Solid and firm; difficult to do or deal with."),
    ("NEAR", "preposition", "At a short distance from; not far away."),
    ("WOMAN", "noun", "An adult female person."),
    ("WRITE", "verb", "To make words with letters."),
];

const MAX_CUSTOM_WORD_LEN: usize = 12;
const MAX_CUSTOM_DEFINITION_LEN: usize = 180;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CellPos {
    x: i32,
    y: i32,
}

impl CellPos {
    const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadingSnakeAction {
    None,
    ExitToTitle,
    Completed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WordEntry {
    word: String,
    part_of_speech: String,
    definition: String,
}

impl WordEntry {
    fn from_parts(word: String, part_of_speech: String, definition: String) -> Self {
        Self {
            word,
            part_of_speech,
            definition,
        }
    }

    fn default_definition(word: &str) -> String {
        format!("Practice spelling the word {}.", word)
    }
}

pub struct ReadingSnake {
    snake: Vec<CellPos>,
    dir: CellPos,
    next_dir: CellPos,
    target: LetterTile,
    decoys: Vec<LetterTile>,
    word: String,
    part_of_speech: String,
    definition: String,
    custom_words: Vec<WordEntry>,
    word_order: Vec<usize>,
    word_index: usize,
    letter_index: usize,
    score: u32,
    lives: u8,
    nightmare_mode: bool,
    bonus_round: bool,
    start_bonus_on_complete: bool,
    completion_returns_action: bool,
    last_step: f64,
    game_over: bool,
    completed: bool,
    showing_definition_card: bool,
    definition_card_title: &'static str,
    message: &'static str,
    touch_start: Option<Vec2>,
}

#[derive(Clone, Debug)]
struct LetterTile {
    pos: CellPos,
    letter: char,
}

impl ReadingSnake {
    pub fn new() -> Self {
        Self::new_with_mode(Vec::new(), false)
    }

    pub fn new_nightmare() -> Self {
        Self::new_with_mode(Vec::new(), true)
    }

    pub fn new_with_words(custom_words: Vec<WordEntry>) -> Self {
        Self::new_with_mode(custom_words, false)
    }

    pub fn new_nightmare_with_words(custom_words: Vec<WordEntry>) -> Self {
        Self::new_with_mode(custom_words, true)
    }

    pub fn new_adventure() -> Self {
        let mut game = Self::new_with_mode(Vec::new(), false);
        game.start_bonus_on_complete = false;
        game.completion_returns_action = true;
        game
    }

    pub fn new_adventure_nightmare() -> Self {
        let mut game = Self::new_with_mode(Vec::new(), true);
        game.completion_returns_action = true;
        game
    }

    fn new_with_mode(custom_words: Vec<WordEntry>, nightmare_mode: bool) -> Self {
        let mut game = Self {
            snake: Vec::new(),
            dir: CellPos::new(1, 0),
            next_dir: CellPos::new(1, 0),
            target: LetterTile {
                pos: CellPos::new(0, 0),
                letter: 'A',
            },
            decoys: Vec::new(),
            word: String::new(),
            part_of_speech: String::new(),
            definition: String::new(),
            custom_words,
            word_order: Vec::new(),
            word_index: 0,
            letter_index: 0,
            score: 0,
            lives: 5,
            nightmare_mode,
            bonus_round: false,
            start_bonus_on_complete: true,
            completion_returns_action: false,
            last_step: get_time(),
            game_over: false,
            completed: false,
            showing_definition_card: false,
            definition_card_title: if nightmare_mode {
                "Nightmare word!"
            } else {
                "New word!"
            },
            message: "Collect the next letter.",
            touch_start: None,
        };
        game.reset_run();
        game
    }

    pub fn update(&mut self) -> ReadingSnakeAction {
        if is_key_pressed(KeyCode::Escape) {
            return ReadingSnakeAction::ExitToTitle;
        }

        if self.game_over {
            if self.completed && self.completion_returns_action {
                return ReadingSnakeAction::Completed;
            }
            let mobile_start =
                primary_tap_position().is_some_and(ui::mobile_action_button_contains);
            let desktop_start = !screen::portrait_layout() && primary_tap_position().is_some();
            if is_key_pressed(KeyCode::Enter)
                || is_key_pressed(KeyCode::Space)
                || mobile_start
                || desktop_start
            {
                *self = Self::new_with_mode(self.custom_words.clone(), self.nightmare_mode);
            }
            return ReadingSnakeAction::None;
        }

        if self.showing_definition_card {
            let mobile_start =
                primary_tap_position().is_some_and(ui::mobile_action_button_contains);
            let desktop_start = !screen::portrait_layout() && primary_tap_position().is_some();
            if is_key_pressed(KeyCode::Enter)
                || is_key_pressed(KeyCode::Space)
                || mobile_start
                || desktop_start
            {
                self.showing_definition_card = false;
                self.place_letters();
                self.last_step = get_time();
            }
            return ReadingSnakeAction::None;
        }

        self.handle_input();
        if get_time() - self.last_step >= STEP_SECONDS {
            self.step();
            self.last_step = get_time();
            if self.completed && self.completion_returns_action {
                return ReadingSnakeAction::Completed;
            }
        }

        ReadingSnakeAction::None
    }

    pub fn draw(&self) {
        let palette = reading_palette(self.nightmare_mode);
        clear_background(if screen::portrait_layout() {
            Color::new(0.018, 0.02, 0.028, 1.0)
        } else {
            palette.desktop_clear
        });
        let portrait = screen::portrait_layout();
        if portrait {
            draw_mobile_space_background();
        } else {
            draw_desktop_playfield_backdrop(self.nightmare_mode);
        }

        if !(portrait && self.showing_definition_card && !self.game_over) {
            self.draw_header();
            self.draw_board();
            self.draw_tiles();
            self.draw_snake();
            self.draw_footer();
        }

        if self.game_over {
            draw_rectangle(
                0.0,
                0.0,
                SCREEN_W,
                SCREEN_H,
                Color::new(0.0, 0.0, 0.0, 0.75),
            );
            let title = if self.completed {
                if self.nightmare_mode {
                    "NIGHTMARE SNAKE COMPLETE"
                } else {
                    "READING SNAKE COMPLETE"
                }
            } else if self.nightmare_mode {
                "NIGHTMARE SNAKE OVER"
            } else {
                "READING SNAKE OVER"
            };
            let title_color = if self.completed { YELLOW } else { RED };
            if screen::portrait_layout() {
                draw_surface_card(150.0, 194.0, 980.0, 278.0, 34.0, surface());
                centered_text(title, 286.0, 44, title_color);
                centered_text(
                    &format!("Final Score: {}", self.score),
                    360.0,
                    28,
                    soft_white(),
                );
                ui::draw_mobile_action_button("START");
            } else {
                let overlay_x = DESKTOP_PANEL_X + 36.0;
                let overlay_w = DESKTOP_PANEL_W - 72.0;
                draw_surface_card(overlay_x, 228.0, overlay_w, 228.0, 26.0, elevated_surface());
                centered_text(title, 300.0, 42, title_color);
                centered_text(&format!("Final Score: {}", self.score), 364.0, 28, YELLOW);
                centered_text("Press ENTER to play again", 418.0, 22, WHITE);
                centered_text("Press ESC for title", 448.0, 18, GRAY);
            }
        } else if self.showing_definition_card {
            self.draw_definition_card();
        }
    }

    fn reset_run(&mut self) {
        self.regenerate_word_order();
        self.reset_snake_to_spawn();
        self.pick_word();
    }

    fn pick_word(&mut self) {
        self.ensure_word_order();
        let word_entry = self.word_entry(self.word_index);
        self.word = word_entry.word;
        self.part_of_speech = word_entry.part_of_speech;
        self.definition = word_entry.definition;
        self.letter_index = 0;
        self.showing_definition_card = true;
        self.message = if self.nightmare_mode {
            "All letters look the same. Spell in order."
        } else {
            "Spell the word in order."
        };
    }

    fn handle_input(&mut self) {
        if (is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W)) && self.dir.y != 1 {
            self.next_dir = CellPos::new(0, -1);
        }
        if (is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S)) && self.dir.y != -1 {
            self.next_dir = CellPos::new(0, 1);
        }
        if (is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A)) && self.dir.x != 1 {
            self.next_dir = CellPos::new(-1, 0);
        }
        if (is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D)) && self.dir.x != -1 {
            self.next_dir = CellPos::new(1, 0);
        }

        let handled_swipe = self.handle_touch_swipe();

        if !handled_swipe {
            if let Some(tap) = primary_tap_position() {
                if screen::portrait_layout() && self.tap_in_ui_chrome(tap) {
                    return;
                }
                self.steer_from_tap(tap);
            }
        }
    }

    fn handle_touch_swipe(&mut self) -> bool {
        if !screen::portrait_layout() {
            return false;
        }

        let mut steered = false;
        for touch in touches() {
            let point = to_virtual_position(touch.position);
            match touch.phase {
                TouchPhase::Started => {
                    if !self.tap_in_ui_chrome(point) && self.point_in_board(point) {
                        self.touch_start = Some(point);
                    }
                }
                TouchPhase::Moved | TouchPhase::Ended => {
                    if let Some(start) = self.touch_start {
                        let delta = point - start;
                        if delta.length() >= MOBILE_SWIPE_THRESHOLD {
                            let next = if delta.x.abs() > delta.y.abs() {
                                CellPos::new(delta.x.signum() as i32, 0)
                            } else {
                                CellPos::new(0, delta.y.signum() as i32)
                            };
                            self.set_next_dir(next);
                            self.touch_start = Some(point);
                            steered = true;
                        }
                    }

                    if touch.phase == TouchPhase::Ended {
                        self.touch_start = None;
                    }
                }
                TouchPhase::Stationary => {}
                _ => {
                    self.touch_start = None;
                }
            }
        }

        steered
    }

    fn tap_in_ui_chrome(&self, tap: Vec2) -> bool {
        if !screen::portrait_layout() {
            return false;
        }

        ui::mobile_back_button_contains(tap)
            || ui::mobile_action_button_contains(tap)
            || tap.y < self.mobile_layout().header_bottom
            || tap.y > self.mobile_footer_top()
    }

    fn playfield_metrics(&self) -> (f32, f32, f32, f32) {
        if screen::portrait_layout() {
            let layout = self.mobile_layout();
            (MOBILE_BOARD_X, layout.board_y, MOBILE_CELL_W, layout.cell_h)
        } else {
            let (board_x, board_y, cell_w, cell_h) = board_metrics();
            (board_x, board_y, cell_w, cell_h)
        }
    }

    fn mobile_layout(&self) -> MobilePlayfieldLayout {
        mobile_playfield_layout(&self.definition)
    }

    fn mobile_footer_top(&self) -> f32 {
        let (_, board_y, _, cell_h) = self.playfield_metrics();
        board_y + MOBILE_GRID_H as f32 * cell_h + MOBILE_FOOTER_GAP
    }

    fn steer_from_tap(&mut self, tap: Vec2) {
        let (board_x, board_y, cell_w, cell_h) = self.playfield_metrics();

        if screen::portrait_layout() && !self.point_in_board(tap) {
            return;
        }

        let head = self.snake[0];
        let head_center = vec2(
            board_x + head.x as f32 * cell_w + cell_w / 2.0,
            board_y + head.y as f32 * cell_h + cell_h / 2.0,
        );
        let delta = tap - head_center;
        if delta.length_squared() < 4.0 {
            return;
        }

        let next = if delta.x.abs() > delta.y.abs() {
            CellPos::new(delta.x.signum() as i32, 0)
        } else {
            CellPos::new(0, delta.y.signum() as i32)
        };

        self.set_next_dir(next);
    }

    fn point_in_board(&self, point: Vec2) -> bool {
        let (board_x, board_y, cell_w, cell_h) = self.playfield_metrics();
        let grid_w = grid_width();
        let grid_h = grid_height();
        point.x >= board_x
            && point.x <= board_x + grid_w as f32 * cell_w
            && point.y >= board_y
            && point.y <= board_y + grid_h as f32 * cell_h
    }

    fn set_next_dir(&mut self, next: CellPos) {
        if next.x != -self.dir.x || next.y != -self.dir.y {
            self.next_dir = next;
        }
    }

    fn step(&mut self) {
        self.dir = self.next_dir;
        let head = self.snake[0];
        let next = CellPos::new(head.x + self.dir.x, head.y + self.dir.y);

        if next.x < 0
            || next.x >= grid_width()
            || next.y < 0
            || next.y >= grid_height()
            || self.snake.contains(&next)
        {
            self.miss("Watch the walls and your tail.");
            return;
        }

        self.snake.insert(0, next);

        if next == self.target.pos {
            self.score += 10;
            self.letter_index += 1;
            if self.letter_index >= self.word.len() {
                self.score += 50;
                if self.nightmare_mode {
                    self.lives = (self.lives + 1).min(MAX_LIVES);
                    self.definition_card_title = "Nightmare complete! Bonus life!";
                } else {
                    self.definition_card_title = "Great job! Next word:";
                }
                self.advance_word();
            } else {
                self.message = "Good letter. Keep spelling.";
                self.place_letters();
            }
            return;
        }

        if self.decoys.iter().any(|tile| tile.pos == next) {
            self.miss("Wrong letter. Follow the word order.");
        } else {
            self.snake.pop();
        }
    }

    fn miss(&mut self, message: &'static str) {
        self.lives = self.lives.saturating_sub(1);
        self.message = message;
        if self.lives == 0 {
            self.game_over = true;
        } else {
            self.reset_snake_to_spawn();
            self.place_letters();
        }
    }

    fn advance_word(&mut self) {
        if self.word_index + 1 < self.word_count() {
            self.word_index += 1;
            self.pick_word();
        } else if self.nightmare_mode || !self.start_bonus_on_complete {
            self.completed = true;
            self.game_over = true;
            self.message = "You completed every word.";
        } else {
            self.start_bonus_round();
        }
    }

    fn start_bonus_round(&mut self) {
        self.nightmare_mode = true;
        self.bonus_round = true;
        self.word_index = 0;
        self.letter_index = 0;
        self.lives = (self.lives + 1).min(MAX_LIVES);
        self.definition_card_title = "Bonus Nightmare round!";
        self.message = "Bonus round: all letters look alike.";
        self.reset_snake_to_spawn();
        self.pick_word();
    }

    fn word_count(&self) -> usize {
        if self.custom_words.is_empty() {
            WORDS.len()
        } else {
            self.custom_words.len()
        }
    }

    fn regenerate_word_order(&mut self) {
        self.word_order = shuffled_word_order(self.word_count());
    }

    fn ensure_word_order(&mut self) {
        if self.word_order.len() != self.word_count() {
            self.regenerate_word_order();
            self.word_index = self.word_index.min(self.word_count().saturating_sub(1));
        }
    }

    fn word_entry(&self, index: usize) -> WordEntry {
        let ordered_index = self.word_order.get(index).copied().unwrap_or(index);
        if self.custom_words.is_empty() {
            let (word, part_of_speech, definition) = WORDS[ordered_index % WORDS.len()];
            WordEntry::from_parts(
                word.to_string(),
                part_of_speech.to_string(),
                definition.to_string(),
            )
        } else {
            self.custom_words[ordered_index % self.custom_words.len()].clone()
        }
    }

    fn reset_snake_to_spawn(&mut self) {
        let mid_x = grid_width() / 2;
        let mid_y = grid_height() / 2;
        self.snake = vec![
            CellPos::new(mid_x, mid_y),
            CellPos::new(mid_x - 1, mid_y),
            CellPos::new(mid_x - 2, mid_y),
        ];
        self.dir = CellPos::new(1, 0);
        self.next_dir = self.dir;
    }

    fn place_letters(&mut self) {
        let expected = self.word.chars().nth(self.letter_index).unwrap_or('A');
        self.target = LetterTile {
            pos: self.random_empty_cell(&[]),
            letter: expected,
        };

        let alphabet: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
            .chars()
            .filter(|letter| *letter != expected)
            .collect();

        self.decoys.clear();
        for _ in 0..4 {
            let occupied: Vec<CellPos> = std::iter::once(self.target.pos)
                .chain(self.decoys.iter().map(|tile| tile.pos))
                .collect();
            let letter_index = random::usize_exclusive(alphabet.len());
            self.decoys.push(LetterTile {
                pos: self.random_empty_cell(&occupied),
                letter: alphabet.get(letter_index).copied().unwrap_or('Z'),
            });
        }
    }

    fn random_empty_cell(&self, reserved: &[CellPos]) -> CellPos {
        loop {
            let pos = CellPos::new(
                random::i32_inclusive(0, grid_width() - 1),
                random::i32_inclusive(0, grid_height() - 1),
            );
            if !self.snake.contains(&pos)
                && !reserved.contains(&pos)
                && !self.is_in_head_safe_area(pos)
            {
                return pos;
            }
        }
    }

    fn is_in_head_safe_area(&self, pos: CellPos) -> bool {
        let head = self.snake[0];
        (pos.x - head.x).abs() <= SNAKE_HEAD_SAFE_RADIUS
            && (pos.y - head.y).abs() <= SNAKE_HEAD_SAFE_RADIUS
    }

    fn draw_header(&self) {
        if screen::portrait_layout() {
            self.draw_mobile_header();
            return;
        }

        self.draw_desktop_header();
    }

    fn draw_desktop_header(&self) {
        let def_size = screen::mobile_text_size(20);
        let stat_size = screen::mobile_text_size(20);
        let header_baseline = {
            let metrics = measure_text("Ag", None, stat_size, 1.0);
            DESKTOP_PANEL_TOP + DESKTOP_HEADER_H / 2.0 + metrics.height / 2.5
        };

        centered_text(
            &format!("Definition: {}", self.definition),
            header_baseline,
            def_size,
            WHITE,
        );

        let score_txt = format!("Score: {}", self.score);
        draw_text(
            &score_txt,
            DESKTOP_PANEL_X + 28.0,
            header_baseline,
            stat_size as f32,
            YELLOW,
        );

        let lives_txt = format!("Lives: {}", self.lives);
        let lives_w = measure_text(&lives_txt, None, stat_size, 1.0).width;
        draw_text(
            &lives_txt,
            DESKTOP_PANEL_X + DESKTOP_PANEL_W - lives_w - 28.0,
            header_baseline,
            stat_size as f32,
            WHITE,
        );
    }

    fn draw_mobile_header(&self) {
        let title = if self.nightmare_mode {
            if self.bonus_round {
                "Night Reading"
            } else {
                "Night Planet"
            }
        } else {
            "Reading Planet"
        };
        let layout = self.mobile_layout();

        draw_mobile_header_band();
        draw_text(
            title,
            154.0,
            48.0,
            screen::mobile_text_size(32) as f32,
            soft_white(),
        );

        draw_stat_chip(
            82.0,
            62.0,
            232.0,
            "Score",
            &self.score.to_string(),
            star_yellow(),
        );
        draw_stat_chip(342.0, 62.0, 208.0, "Lives", &self.lives.to_string(), mint());
        draw_stat_chip(
            578.0,
            62.0,
            270.0,
            "Next",
            &self.next_letter_label(),
            if self.nightmare_mode {
                planet_pink()
            } else {
                soft_cyan()
            },
        );

        draw_surface_card(
            82.0,
            MOBILE_GOAL_CARD_Y,
            1116.0,
            layout.goal_card_h,
            20.0,
            elevated_surface(),
        );
        centered_text(
            "Definition",
            MOBILE_GOAL_TEXT_Y - 10.0,
            screen::mobile_text_size(22),
            muted_text(),
        );
        draw_wrapped_centered_text(
            &layout.goal_text,
            MOBILE_GOAL_TEXT_Y + 16.0,
            MOBILE_GOAL_TEXT_W,
            screen::mobile_text_size(MOBILE_GOAL_TEXT_SIZE),
            soft_white(),
        );
    }

    fn draw_board(&self) {
        let palette = reading_palette(self.nightmare_mode);
        let (board_x, board_y, cell_w, cell_h) = self.playfield_metrics();
        let grid_w = grid_width();
        let grid_h = grid_height();
        if screen::portrait_layout() {
            draw_surface_card(
                board_x - 26.0,
                board_y - 24.0,
                grid_w as f32 * cell_w + 52.0,
                grid_h as f32 * cell_h + 48.0,
                28.0,
                Color::new(0.095, 0.105, 0.15, 0.98),
            );
            draw_round_rect(
                board_x - 26.0,
                board_y - 24.0,
                grid_w as f32 * cell_w + 52.0,
                7.0,
                4.0,
                palette.desktop_accent,
            );
        } else {
            draw_surface_card(
                board_x - 12.0,
                board_y - 10.0,
                grid_w as f32 * cell_w + 24.0,
                grid_h as f32 * cell_h + 20.0,
                16.0,
                palette.desktop_board_shell,
            );
        }

        for x in 0..grid_w {
            for y in 0..grid_h {
                let color = if screen::portrait_layout() {
                    if (x + y) % 2 == 0 {
                        Color::new(0.12, 0.15, 0.2, 0.98)
                    } else {
                        Color::new(0.09, 0.12, 0.17, 0.98)
                    }
                } else if (x + y) % 2 == 0 {
                    palette.desktop_cell_a
                } else {
                    palette.desktop_cell_b
                };
                draw_rectangle(
                    board_x + x as f32 * cell_w,
                    board_y + y as f32 * cell_h,
                    cell_w,
                    cell_h,
                    color,
                );
            }
        }
    }

    fn draw_tiles(&self) {
        let palette = reading_palette(self.nightmare_mode);
        let target_color = if self.nightmare_mode {
            palette.letter_tile
        } else {
            Color::new(1.0, 0.86, 0.2, 1.0)
        };
        let decoy_color = if self.nightmare_mode {
            palette.letter_tile
        } else {
            Color::new(0.8, 0.25, 0.25, 1.0)
        };

        self.draw_tile(&self.target, target_color);
        for tile in &self.decoys {
            self.draw_tile(tile, decoy_color);
        }
    }

    fn draw_tile(&self, tile: &LetterTile, color: Color) {
        let (board_x, board_y, cell_w, cell_h) = self.playfield_metrics();
        let mobile = screen::portrait_layout();
        let letter_size = if mobile {
            28
        } else {
            (cell_w * 0.58).clamp(18.0, 32.0) as u16
        };
        let x = board_x + tile.pos.x as f32 * cell_w;
        let y = board_y + tile.pos.y as f32 * cell_h;
        let inset_x = if mobile { 6.0 } else { 2.0 };
        let inset_y = if mobile { 1.5 } else { 2.0 };
        draw_round_rect(
            x + inset_x,
            y + inset_y,
            cell_w - inset_x * 2.0,
            cell_h - inset_y * 2.0,
            if mobile { 8.0 } else { 0.0 },
            color,
        );
        if mobile {
            draw_round_rect(
                x + inset_x + 4.0,
                y + inset_y + 4.0,
                cell_w - inset_x * 2.0 - 8.0,
                4.0,
                2.0,
                Color::new(1.0, 1.0, 1.0, 0.22),
            );
        }
        let letter = tile.letter.to_string();
        let metrics = measure_text(&letter, None, letter_size, 1.0);
        draw_text(
            &letter,
            x + cell_w / 2.0 - metrics.width / 2.0,
            y + cell_h / 2.0 + metrics.height / 2.5,
            letter_size as f32,
            BLACK,
        );
    }

    fn draw_snake(&self) {
        let palette = reading_palette(self.nightmare_mode);
        let (board_x, board_y, cell_w, cell_h) = self.playfield_metrics();
        let mobile = screen::portrait_layout();
        for (idx, part) in self.snake.iter().enumerate() {
            let x = board_x + part.x as f32 * cell_w;
            let y = board_y + part.y as f32 * cell_h;
            let color = if idx == 0 {
                if mobile {
                    soft_cyan()
                } else {
                    palette.desktop_snake_head
                }
            } else if mobile {
                Color::new(0.38, 0.86, 0.58, 1.0)
            } else {
                palette.desktop_snake_body
            };
            let inset_x = if mobile { 6.0 } else { 3.0 };
            let inset_y = if mobile { 2.5 } else { 3.0 };
            draw_round_rect(
                x + inset_x,
                y + inset_y,
                cell_w - inset_x * 2.0,
                cell_h - inset_y * 2.0,
                if mobile { 6.0 } else { 0.0 },
                color,
            );
        }
    }

    fn draw_footer(&self) {
        if screen::portrait_layout() {
            self.draw_mobile_footer();
            return;
        }

        let (_, board_y, _, cell_h) = board_metrics();
        let grid_h = grid_height();
        let board_bottom = board_y + grid_h as f32 * cell_h + DESKTOP_BOARD_FRAME_BOTTOM;
        let footer_y = board_bottom + DESKTOP_FOOTER_GAP;
        let progress_size = screen::mobile_text_size(26);
        let message_size = screen::mobile_text_size(18);
        let progress = format_word_progress(&self.word, self.letter_index);
        let label = "Word";
        let label_size = 26_u16;
        let progress_size_u16 = progress_size;
        let label_w = measure_text(label, None, label_size, 1.0).width;
        let progress_w = measure_text(&progress, None, progress_size_u16, 1.0).width;
        let row_gap = 14.0;
        let row_w = label_w + row_gap + progress_w;
        let row_x = SCREEN_W / 2.0 - row_w / 2.0;

        draw_text(label, row_x, footer_y + 2.0, label_size as f32, muted_text());
        draw_text(
            &progress,
            row_x + label_w + row_gap,
            footer_y,
            progress_size as f32,
            star_yellow(),
        );
        centered_text(self.message, footer_y + 34.0, message_size, soft_white());
    }

    fn draw_mobile_footer(&self) {
        let footer_y = self.mobile_footer_top();
        let progress = format_word_progress(&self.word, self.letter_index);
        let label_size = screen::mobile_text_size(22);
        let progress_size = screen::mobile_text_size(34);
        let message_size = screen::mobile_text_size(20);

        draw_surface_card(82.0, footer_y, 1116.0, MOBILE_FOOTER_H, 20.0, elevated_surface());
        centered_text_in_rect(
            "Word",
            82.0,
            footer_y + 2.0,
            1116.0,
            22.0,
            label_size,
            muted_text(),
        );
        centered_text_in_rect(
            &progress,
            82.0,
            footer_y + 20.0,
            1116.0,
            34.0,
            progress_size,
            star_yellow(),
        );
        centered_text(
            self.message,
            footer_y + 50.0,
            message_size,
            soft_white(),
        );
        centered_text(
            "Tap or swipe on the board to steer",
            footer_y + 74.0,
            screen::mobile_text_size(18),
            muted_text(),
        );
    }

    fn draw_definition_card(&self) {
        if screen::portrait_layout() {
            self.draw_mobile_definition_card();
            return;
        }

        let title_size = screen::mobile_text_size(42);
        let pos_size = screen::mobile_text_size(24);
        let def_size = screen::mobile_text_size(30);
        let palette = reading_palette(self.nightmare_mode);

        draw_rectangle(
            0.0,
            0.0,
            SCREEN_W,
            SCREEN_H,
            Color::new(0.0, 0.0, 0.0, 0.65),
        );

        let (_, board_y, _, cell_h) = board_metrics();
        let board_h = grid_height() as f32 * cell_h;
        let card_w = DESKTOP_PANEL_W - 96.0;
        let card_x = DESKTOP_PANEL_X + 48.0;
        let card_h = 304.0;
        let card_y = board_y + (board_h - card_h) / 2.0;
        let accent_inset = 22.0;
        let accent_color = palette.desktop_accent;
        let content_w = card_w - 112.0;

        draw_surface_card(
            card_x,
            card_y,
            card_w,
            card_h,
            22.0,
            palette.desktop_card_fill,
        );
        draw_round_rect(
            card_x + accent_inset,
            card_y + accent_inset,
            card_w - accent_inset * 2.0,
            3.0,
            1.5,
            accent_color,
        );
        draw_round_rect(
            card_x + accent_inset,
            card_y + card_h - accent_inset - 3.0,
            card_w - accent_inset * 2.0,
            3.0,
            1.5,
            accent_color,
        );

        let content_top = card_y + accent_inset + 28.0;
        centered_text(
            self.definition_card_title,
            content_top + 18.0,
            title_size,
            palette.desktop_card_title,
        );
        centered_text(
            &format!("Part of speech: {}", self.part_of_speech),
            content_top + 82.0,
            pos_size,
            accent_color,
        );
        draw_wrapped_centered_text(
            &self.definition,
            content_top + 126.0,
            content_w,
            def_size,
            WHITE,
        );
        centered_text(
            "Press ENTER to start",
            card_y + card_h - accent_inset - 18.0,
            20,
            GRAY,
        );
    }

    fn draw_mobile_definition_card(&self) {
        const CARD_X: f32 = 82.0;
        const CARD_W: f32 = 1116.0;
        const CARD_H: f32 = 520.0;
        const CONTENT_W: f32 = CARD_W - 96.0;
        const HEADER_H: f32 = 86.0;
        let card_y = 108.0;

        draw_rectangle(
            0.0,
            0.0,
            SCREEN_W,
            SCREEN_H,
            Color::new(0.01, 0.012, 0.02, 0.93),
        );

        let screen_title = if self.nightmare_mode {
            if self.bonus_round {
                "Night Reading"
            } else {
                "Night Planet"
            }
        } else {
            "Reading Planet"
        };
        centered_text(
            screen_title,
            54.0,
            screen::mobile_text_size(36),
            soft_white(),
        );

        let accent = if self.nightmare_mode {
            Color::new(0.48, 0.29, 1.0, 0.92)
        } else {
            Color::new(0.4, 1.0, 0.65, 0.88)
        };
        draw_mobile_reading_card(CARD_X, card_y, CARD_W, CARD_H, accent);

        let title_size = screen::mobile_text_size(46);
        let word_size = screen::mobile_text_size(if self.nightmare_mode { 92 } else { 84 });
        let pos_size = screen::mobile_text_size(36);
        let def_size = screen::mobile_text_size(44);
        let hint_size = screen::mobile_text_size(26);
        let word_color = if self.nightmare_mode {
            planet_pink()
        } else {
            soft_cyan()
        };

        centered_text_in_card(
            self.definition_card_title,
            CARD_X,
            CARD_W,
            card_y + HEADER_H / 2.0 + title_size as f32 / 3.0,
            title_size,
            soft_white(),
        );

        let mut content_y = card_y + HEADER_H + 42.0;
        centered_text_fit_in_card(
            &self.word,
            CARD_X,
            CARD_W,
            content_y,
            CONTENT_W,
            word_size,
            word_color,
        );
        content_y += word_size as f32 + 22.0;
        centered_text_in_card(
            &format!("{} word", self.part_of_speech),
            CARD_X,
            CARD_W,
            content_y,
            pos_size,
            if self.nightmare_mode {
                soft_cyan()
            } else {
                mint()
            },
        );
        content_y += pos_size as f32 + 30.0;
        draw_wrapped_centered_in_card(
            &self.definition,
            CARD_X,
            CARD_W,
            content_y,
            CONTENT_W,
            def_size,
            soft_white(),
        );
        centered_text(
            "Tap START when you know the word",
            ui::MOBILE_ACTION_Y - 40.0,
            hint_size,
            Color::new(0.78, 0.8, 0.88, 1.0),
        );
        ui::draw_mobile_action_button("START");
    }

    fn next_letter_label(&self) -> String {
        self.word
            .chars()
            .nth(self.letter_index)
            .map(|letter| letter.to_string())
            .unwrap_or_else(|| "-".to_string())
    }
}

pub fn custom_words_from_input(input: &str) -> Vec<WordEntry> {
    if input.contains(':') {
        input
            .split([';', '\n', '\r'])
            .filter_map(|entry| {
                let (word, definition) = entry.split_once(':')?;
                let word = normalize_word(word)?;
                let definition = definition.trim();
                Some(WordEntry::from_parts(
                    word.clone(),
                    "custom word".to_string(),
                    if definition.is_empty() {
                        WordEntry::default_definition(&word)
                    } else {
                        clamp_definition(definition)
                    },
                ))
            })
            .take(MAX_CUSTOM_WORDS)
            .collect()
    } else {
        input
            .split(|ch: char| !ch.is_ascii_alphabetic())
            .filter_map(|word| {
                let word = normalize_word(word)?;
                let definition = WordEntry::default_definition(&word);
                Some(WordEntry::from_parts(
                    word,
                    "custom word".to_string(),
                    definition,
                ))
            })
            .take(MAX_CUSTOM_WORDS)
            .collect()
    }
}

fn clamp_definition(definition: &str) -> String {
    definition
        .chars()
        .take(MAX_CUSTOM_DEFINITION_LEN)
        .collect::<String>()
}

fn normalize_word(word: &str) -> Option<String> {
    let word: String = word
        .chars()
        .filter(|ch| ch.is_ascii_alphabetic())
        .take(MAX_CUSTOM_WORD_LEN)
        .flat_map(char::to_uppercase)
        .collect();
    if word.is_empty() {
        None
    } else {
        Some(word)
    }
}

fn format_word_progress(word: &str, letter_index: usize) -> String {
    word.chars()
        .enumerate()
        .map(|(idx, ch)| if idx < letter_index { ch } else { '_' })
        .collect::<Vec<char>>()
        .into_iter()
        .flat_map(|ch| [ch, ' '])
        .collect()
}

fn shuffled_word_order(word_count: usize) -> Vec<usize> {
    let mut order: Vec<usize> = (0..word_count).collect();
    random::shuffle(&mut order);
    order
}

fn mobile_playfield_layout(definition: &str) -> MobilePlayfieldLayout {
    let goal_text =
        truncate_wrapped_text(definition, MOBILE_GOAL_TEXT_W, MOBILE_GOAL_TEXT_SIZE, MOBILE_GOAL_MAX_LINES);
    let goal_lines = wrapped_line_count(&goal_text, MOBILE_GOAL_TEXT_W, MOBILE_GOAL_TEXT_SIZE).max(1);
    let goal_text_h = wrapped_block_height(goal_lines, MOBILE_GOAL_TEXT_SIZE);
    let goal_card_h = (MOBILE_GOAL_TEXT_Y - MOBILE_GOAL_CARD_Y) + goal_text_h + 14.0;
    let header_bottom = MOBILE_GOAL_CARD_Y + goal_card_h;
    let mut board_y = header_bottom + MOBILE_BOARD_GAP;
    let mut cell_h = MOBILE_CELL_H;

    let mut board_h = MOBILE_GRID_H as f32 * cell_h;
    let mut footer_top = board_y + board_h + MOBILE_FOOTER_GAP;
    let mut stack_bottom = footer_top + MOBILE_FOOTER_H;

    if stack_bottom > MOBILE_PLAYFIELD_BOTTOM {
        let max_board_h =
            MOBILE_PLAYFIELD_BOTTOM - MOBILE_FOOTER_GAP - MOBILE_FOOTER_H - board_y;
        if max_board_h >= MOBILE_GRID_H as f32 * 18.0 {
            cell_h = (max_board_h / MOBILE_GRID_H as f32).floor().max(18.0);
            board_h = MOBILE_GRID_H as f32 * cell_h;
            footer_top = board_y + board_h + MOBILE_FOOTER_GAP;
            stack_bottom = footer_top + MOBILE_FOOTER_H;
        }
    }

    if stack_bottom > MOBILE_PLAYFIELD_BOTTOM {
        board_y = MOBILE_PLAYFIELD_BOTTOM - MOBILE_FOOTER_H - MOBILE_FOOTER_GAP - board_h;
        board_y = board_y.max(header_bottom + MOBILE_BOARD_GAP);
    }

    MobilePlayfieldLayout {
        header_bottom,
        goal_card_h,
        goal_text,
        board_y,
        cell_h,
    }
}

fn grid_width() -> i32 {
    if screen::portrait_layout() {
        MOBILE_GRID_W
    } else {
        DESKTOP_GRID_W
    }
}

fn grid_height() -> i32 {
    if screen::portrait_layout() {
        MOBILE_GRID_H
    } else {
        DESKTOP_GRID_H
    }
}

fn desktop_cell_size() -> f32 {
    let inner_w = DESKTOP_PANEL_W - 40.0;
    let inner_h = SCREEN_H
        - DESKTOP_PANEL_TOP
        - DESKTOP_HEADER_H
        - DESKTOP_FOOTER_ZONE
        - DESKTOP_BOTTOM_CHROME
        - 8.0;
    let from_w = inner_w / DESKTOP_GRID_W as f32;
    let from_h = inner_h / DESKTOP_GRID_H as f32;
    from_w.min(from_h)
}

fn board_metrics() -> (f32, f32, f32, f32) {
    let cell = desktop_cell_size();
    let board_w = DESKTOP_GRID_W as f32 * cell;
    let board_x = DESKTOP_PANEL_X + (DESKTOP_PANEL_W - board_w) / 2.0;
    let board_y = DESKTOP_PANEL_TOP + DESKTOP_HEADER_H;
    (board_x, board_y, cell, cell)
}

fn desktop_board_bottom() -> f32 {
    let (_, board_y, _, cell_h) = board_metrics();
    board_y + grid_height() as f32 * cell_h + DESKTOP_BOARD_FRAME_BOTTOM
}

fn desktop_panel_bottom() -> f32 {
    desktop_board_bottom() + DESKTOP_FOOTER_GAP + DESKTOP_FOOTER_ZONE + DESKTOP_PANEL_BOTTOM_PAD
}

fn draw_desktop_playfield_backdrop(nightmare_mode: bool) {
    let panel_h = desktop_panel_bottom() - DESKTOP_PANEL_TOP + 8.0;
    let panel_fill = if nightmare_mode {
        Color::new(0.08, 0.07, 0.12, 0.97)
    } else {
        surface()
    };
    draw_surface_card(
        DESKTOP_PANEL_X,
        DESKTOP_PANEL_TOP,
        DESKTOP_PANEL_W,
        panel_h,
        28.0,
        panel_fill,
    );
}

struct ReadingPalette {
    desktop_clear: Color,
    desktop_board_shell: Color,
    desktop_cell_a: Color,
    desktop_cell_b: Color,
    desktop_accent: Color,
    desktop_card_fill: Color,
    desktop_card_title: Color,
    desktop_snake_head: Color,
    desktop_snake_body: Color,
    letter_tile: Color,
}

fn reading_palette(nightmare_mode: bool) -> ReadingPalette {
    if nightmare_mode {
        ReadingPalette {
            desktop_clear: Color::new(0.018, 0.02, 0.035, 1.0),
            desktop_board_shell: Color::new(0.11, 0.09, 0.16, 0.96),
            desktop_cell_a: Color::new(0.12, 0.15, 0.2, 0.98),
            desktop_cell_b: Color::new(0.09, 0.12, 0.17, 0.98),
            desktop_accent: Color::new(0.48, 0.29, 1.0, 0.88),
            desktop_card_fill: Color::new(0.09, 0.08, 0.13, 0.98),
            desktop_card_title: planet_pink(),
            desktop_snake_head: soft_cyan(),
            desktop_snake_body: Color::new(0.35, 0.55, 0.95, 1.0),
            letter_tile: Color::new(0.55, 0.75, 1.0, 1.0),
        }
    } else {
        ReadingPalette {
            desktop_clear: Color::new(0.02, 0.04, 0.03, 1.0),
            desktop_board_shell: Color::new(0.08, 0.17, 0.13, 0.96),
            desktop_cell_a: Color::new(0.08, 0.17, 0.13, 0.92),
            desktop_cell_b: Color::new(0.055, 0.125, 0.12, 0.92),
            desktop_accent: Color::new(0.4, 1.0, 0.65, 0.88),
            desktop_card_fill: Color::new(0.06, 0.14, 0.1, 0.98),
            desktop_card_title: YELLOW,
            desktop_snake_head: mint(),
            desktop_snake_body: Color::new(0.38, 0.86, 0.58, 1.0),
            letter_tile: Color::new(1.0, 0.86, 0.2, 1.0),
        }
    }
}

fn draw_mobile_space_background() {
    draw_rectangle(
        0.0,
        0.0,
        SCREEN_W,
        SCREEN_H,
        Color::new(0.035, 0.035, 0.07, 1.0),
    );
    draw_rectangle(
        0.0,
        0.0,
        SCREEN_W,
        188.0,
        Color::new(0.38, 0.22, 0.95, 0.72),
    );

    for i in 0..42 {
        let x = ((i * 83 + 29) % SCREEN_W as i32) as f32;
        let y = ((i * 47 + 61) % SCREEN_H as i32) as f32;
        let radius = if i % 6 == 0 { 1.8 } else { 0.9 };
        let color = if i % 8 == 0 {
            Color::new(0.46, 0.9, 1.0, 0.38)
        } else {
            Color::new(0.94, 0.96, 0.82, 0.32)
        };
        draw_circle(x, y, radius, color);
    }

    draw_circle(1088.0, 282.0, 84.0, Color::new(0.55, 0.36, 1.0, 0.1));
    draw_circle(150.0, 602.0, 96.0, Color::new(0.3, 0.62, 1.0, 0.08));
}

fn draw_mobile_header_band() {
    draw_circle(1000.0, 60.0, 280.0, Color::new(0.54, 0.35, 1.0, 0.12));
    draw_circle(256.0, 116.0, 180.0, Color::new(0.25, 0.52, 1.0, 0.1));
}

fn draw_mobile_reading_card(x: f32, y: f32, w: f32, h: f32, accent: Color) {
    draw_round_rect(x, y, w, h, 36.0, Color::new(0.115, 0.12, 0.16, 0.98));
    draw_round_rect(x, y, w, 86.0, 36.0, Color::new(0.16, 0.17, 0.23, 0.96));
    draw_round_rect(
        x + 34.0,
        y + h - 8.0,
        w - 68.0,
        8.0,
        4.0,
        accent,
    );
}

fn draw_surface_card(x: f32, y: f32, w: f32, h: f32, radius: f32, fill: Color) {
    draw_round_rect(x, y, w, h, radius, fill);
    draw_round_rect(
        x,
        y,
        w,
        (h * 0.34).max(18.0),
        radius,
        Color::new(1.0, 1.0, 1.0, 0.025),
    );
}

fn draw_round_rect(x: f32, y: f32, w: f32, h: f32, radius: f32, color: Color) {
    let r = radius.min(w / 2.0).min(h / 2.0);
    draw_rectangle(x + r, y, w - r * 2.0, h, color);
    draw_rectangle(x, y + r, w, h - r * 2.0, color);
    draw_circle(x + r, y + r, r, color);
    draw_circle(x + w - r, y + r, r, color);
    draw_circle(x + r, y + h - r, r, color);
    draw_circle(x + w - r, y + h - r, r, color);
}

fn centered_text_in_rect(text: &str, x: f32, y: f32, w: f32, h: f32, font_size: u16, color: Color) {
    let metrics = measure_text(text, None, font_size, 1.0);
    draw_text(
        text,
        x + w / 2.0 - metrics.width / 2.0,
        y + h / 2.0 + metrics.height / 2.5,
        font_size as f32,
        color,
    );
}

fn draw_stat_chip(x: f32, y: f32, w: f32, label: &str, value: &str, accent: Color) {
    let label_size = screen::mobile_text_size(24);
    let value_size = screen::mobile_text_size(30);
    draw_round_rect(x, y, w, 46.0, 23.0, Color::new(0.09, 0.1, 0.13, 0.96));
    draw_circle(x + 28.0, y + 23.0, 8.0, accent);
    draw_text(label, x + 50.0, y + 30.0, label_size as f32, muted_text());
    let metrics = measure_text(value, None, value_size, 1.0);
    draw_text(
        value,
        x + w - metrics.width - 28.0,
        y + 33.0,
        value_size as f32,
        soft_white(),
    );
}

fn surface() -> Color {
    Color::new(0.095, 0.1, 0.14, 0.97)
}

fn elevated_surface() -> Color {
    Color::new(0.12, 0.125, 0.17, 0.97)
}

fn soft_white() -> Color {
    Color::new(0.96, 0.97, 1.0, 1.0)
}

fn muted_text() -> Color {
    Color::new(0.66, 0.67, 0.74, 1.0)
}

fn soft_cyan() -> Color {
    Color::new(0.5, 0.72, 1.0, 1.0)
}

fn star_yellow() -> Color {
    Color::new(1.0, 0.82, 0.34, 1.0)
}

fn planet_pink() -> Color {
    Color::new(1.0, 0.42, 0.68, 1.0)
}

fn mint() -> Color {
    Color::new(0.48, 0.95, 0.68, 1.0)
}

fn centered_text(text: &str, y: f32, font_size: u16, color: Color) {
    centered_text_in_card(text, 0.0, SCREEN_W, y, font_size, color);
}

fn centered_text_in_card(
    text: &str,
    card_x: f32,
    card_w: f32,
    y: f32,
    font_size: u16,
    color: Color,
) {
    let metrics = measure_text(text, None, font_size, 1.0);
    draw_text(
        text,
        card_x + card_w / 2.0 - metrics.width / 2.0,
        y,
        font_size as f32,
        color,
    );
}

fn centered_text_fit(text: &str, y: f32, max_width: f32, base_size: u16, color: Color) {
    centered_text_fit_in_card(text, 0.0, SCREEN_W, y, max_width, base_size, color);
}

fn centered_text_fit_in_card(
    text: &str,
    card_x: f32,
    card_w: f32,
    y: f32,
    max_width: f32,
    base_size: u16,
    color: Color,
) {
    let mut size = base_size;
    while size > 28 && measure_text(text, None, size, 1.0).width > max_width {
        size -= 2;
    }
    centered_text_in_card(text, card_x, card_w, y, size, color);
}

fn draw_wrapped_centered_text(text: &str, y: f32, max_width: f32, font_size: u16, color: Color) {
    draw_wrapped_centered_in_card(text, 0.0, SCREEN_W, y, max_width, font_size, color);
}

fn draw_wrapped_centered_in_card(
    text: &str,
    card_x: f32,
    card_w: f32,
    y: f32,
    max_width: f32,
    font_size: u16,
    color: Color,
) {
    let mut line = String::new();
    let mut line_y = y;

    for word in text.split_whitespace() {
        let next = if line.is_empty() {
            word.to_string()
        } else {
            format!("{} {}", line, word)
        };

        if measure_text(&next, None, font_size, 1.0).width > max_width && !line.is_empty() {
            centered_text_in_card(&line, card_x, card_w, line_y, font_size, color);
            line = word.to_string();
            line_y += font_size as f32 + 10.0;
        } else {
            line = next;
        }
    }

    if !line.is_empty() {
        centered_text_in_card(&line, card_x, card_w, line_y, font_size, color);
    }
}

fn wrapped_line_count(text: &str, max_width: f32, font_size: u16) -> usize {
    let mut line = String::new();
    let mut lines = 0usize;

    for word in text.split_whitespace() {
        let next = if line.is_empty() {
            word.to_string()
        } else {
            format!("{} {}", line, word)
        };

        if measure_text(&next, None, font_size, 1.0).width > max_width && !line.is_empty() {
            lines += 1;
            line = word.to_string();
        } else {
            line = next;
        }
    }

    if !line.is_empty() {
        lines += 1;
    }

    lines
}

fn wrapped_block_height(lines: usize, font_size: u16) -> f32 {
    if lines == 0 {
        0.0
    } else {
        lines as f32 * font_size as f32 + (lines.saturating_sub(1) as f32 * 8.0)
    }
}

fn truncate_wrapped_text(text: &str, max_width: f32, font_size: u16, max_lines: usize) -> String {
    if max_lines == 0 {
        return String::new();
    }

    let words: Vec<&str> = text.split_whitespace().collect();
    let mut line = String::new();
    let mut lines: Vec<String> = Vec::new();
    let mut word_idx = 0usize;
    let mut truncated = false;

    while word_idx < words.len() {
        let word = words[word_idx];
        let next = if line.is_empty() {
            word.to_string()
        } else {
            format!("{} {}", line, word)
        };

        if measure_text(&next, None, font_size, 1.0).width > max_width && !line.is_empty() {
            lines.push(line);
            line = word.to_string();
            if lines.len() >= max_lines {
                truncated = word_idx + 1 < words.len() || !line.is_empty();
                break;
            }
        } else {
            line = next;
        }
        word_idx += 1;
    }

    if lines.len() < max_lines {
        if !line.is_empty() {
            lines.push(line);
        }
    } else {
        truncated = true;
    }

    if truncated {
        if let Some(last) = lines.last_mut() {
            let trimmed = last.trim_end_matches('.').to_string();
            *last = format!("{trimmed}...");
        }
    }

    lines.join(" ")
}

fn draw_wrapped_text(text: &str, x: f32, y: f32, max_width: f32, font_size: u16, color: Color) {
    let mut line = String::new();
    let mut line_y = y;

    for word in text.split_whitespace() {
        let next = if line.is_empty() {
            word.to_string()
        } else {
            format!("{} {}", line, word)
        };

        if measure_text(&next, None, font_size, 1.0).width > max_width && !line.is_empty() {
            draw_text(&line, x, line_y, font_size as f32, color);
            line = word.to_string();
            line_y += font_size as f32 + 8.0;
        } else {
            line = next;
        }
    }

    if !line.is_empty() {
        draw_text(&line, x, line_y, font_size as f32, color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sorted_order(mut order: Vec<usize>) -> Vec<usize> {
        order.sort_unstable();
        order
    }

    fn test_game(custom_words: Vec<WordEntry>, word_order: Vec<usize>) -> ReadingSnake {
        ReadingSnake {
            snake: vec![CellPos::new(0, 0)],
            dir: CellPos::new(1, 0),
            next_dir: CellPos::new(1, 0),
            target: LetterTile {
                pos: CellPos::new(0, 0),
                letter: 'A',
            },
            decoys: Vec::new(),
            word: String::new(),
            part_of_speech: String::new(),
            definition: String::new(),
            custom_words,
            word_order,
            word_index: 0,
            letter_index: 0,
            score: 0,
            lives: 5,
            nightmare_mode: false,
            bonus_round: false,
            start_bonus_on_complete: true,
            completion_returns_action: false,
            last_step: 0.0,
            game_over: false,
            completed: false,
            showing_definition_card: false,
            definition_card_title: "New word!",
            message: "Collect the next letter.",
            touch_start: None,
        }
    }

    #[test]
    fn parses_custom_words_from_spaces_commas_and_lines() {
        assert_eq!(
            custom_words_from_input("cat, dog\nmoon sun"),
            vec![
                WordEntry::from_parts(
                    "CAT".to_string(),
                    "custom word".to_string(),
                    "Practice spelling the word CAT.".to_string()
                ),
                WordEntry::from_parts(
                    "DOG".to_string(),
                    "custom word".to_string(),
                    "Practice spelling the word DOG.".to_string()
                ),
                WordEntry::from_parts(
                    "MOON".to_string(),
                    "custom word".to_string(),
                    "Practice spelling the word MOON.".to_string()
                ),
                WordEntry::from_parts(
                    "SUN".to_string(),
                    "custom word".to_string(),
                    "Practice spelling the word SUN.".to_string()
                ),
            ]
        );
    }

    #[test]
    fn skips_punctuation_and_limits_word_length() {
        assert_eq!(
            custom_words_from_input("  rocket!!! supercalifragilistic  "),
            vec![
                WordEntry::from_parts(
                    "ROCKET".to_string(),
                    "custom word".to_string(),
                    "Practice spelling the word ROCKET.".to_string()
                ),
                WordEntry::from_parts(
                    "SUPERCALIFRA".to_string(),
                    "custom word".to_string(),
                    "Practice spelling the word SUPERCALIFRA.".to_string(),
                ),
            ]
        );
    }

    #[test]
    fn parses_custom_words_with_definitions() {
        assert_eq!(
            custom_words_from_input("apple: a fruit; moon: shines at night"),
            vec![
                WordEntry::from_parts(
                    "APPLE".to_string(),
                    "custom word".to_string(),
                    "a fruit".to_string()
                ),
                WordEntry::from_parts(
                    "MOON".to_string(),
                    "custom word".to_string(),
                    "shines at night".to_string()
                ),
            ]
        );
    }

    #[test]
    fn caps_custom_word_count_and_definition_length() {
        let input = (0..80)
            .map(|idx| format!("word{}: {}", idx, "long ".repeat(80)))
            .collect::<Vec<String>>()
            .join(";");
        let words = custom_words_from_input(&input);

        assert_eq!(words.len(), MAX_CUSTOM_WORDS);
        assert!(words
            .iter()
            .all(|word| word.definition.chars().count() <= MAX_CUSTOM_DEFINITION_LEN));
    }

    #[test]
    fn shuffled_word_order_contains_each_default_index_once() {
        let order = shuffled_word_order(WORDS.len());

        assert_eq!(order.len(), WORDS.len());
        assert_eq!(
            sorted_order(order),
            (0..WORDS.len()).collect::<Vec<usize>>()
        );
    }

    #[test]
    fn shuffled_word_order_contains_each_custom_index_once() {
        let custom_word_count = 4;
        let order = shuffled_word_order(custom_word_count);

        assert_eq!(order.len(), custom_word_count);
        assert_eq!(
            sorted_order(order),
            (0..custom_word_count).collect::<Vec<usize>>()
        );
    }

    #[test]
    fn word_entry_maps_through_custom_word_order() {
        let custom_words = custom_words_from_input("cat dog moon");
        let game = test_game(custom_words, vec![2, 0, 1]);

        assert_eq!(game.word_entry(0).word, "MOON");
        assert_eq!(game.word_entry(1).word, "CAT");
        assert_eq!(game.word_entry(2).word, "DOG");
    }

    #[test]
    fn wrapped_block_height_grows_with_line_count() {
        assert!(wrapped_block_height(3, 24) > wrapped_block_height(1, 24));
    }

    #[test]
    fn mobile_goal_card_bottom_accounts_for_text_block() {
        let goal_lines = 3usize;
        let goal_text_h = wrapped_block_height(goal_lines, MOBILE_GOAL_TEXT_SIZE);
        let goal_card_h = (MOBILE_GOAL_TEXT_Y - MOBILE_GOAL_CARD_Y) + goal_text_h + 14.0;

        assert!(goal_card_h > 54.0);
        assert!(MOBILE_GOAL_CARD_Y + goal_card_h + MOBILE_BOARD_GAP > 190.0);
    }
}
