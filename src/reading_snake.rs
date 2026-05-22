use crate::random;
use crate::screen::{self, SCREEN_H, SCREEN_W};
use macroquad::prelude::*;

const GRID_W: i32 = 14;
const GRID_H: i32 = 14;
const CELL: f32 = 27.0;
const BOARD_X: f32 = 323.0;
const BOARD_Y: f32 = 160.0;
const STEP_SECONDS: f64 = 0.25;
const SNAKE_HEAD_SAFE_RADIUS: i32 = 3;
const MAX_LIVES: u8 = 9;

const WORDS: &[(&str, &str, &str)] = &[
    ("KEY", "noun", "A small tool used to open a lock."),
    ("BUMPY", "adjective", "Not smooth; full of bumps."),
    ("PUPPY", "noun", "A young dog."),
    ("FUNNY", "adjective", "Something that makes you laugh."),
    ("PENNY", "noun", "A coin worth one cent."),
    ("SANDY", "adjective", "Covered with or made of sand."),
    ("MY", "adjective", "Belonging to me."),
    ("NIGHT", "noun", "The dark time between sunset and morning."),
    ("WASH", "verb", "To clean with water."),
    (
        "WOULD",
        "helping verb",
        "A helping word used to tell what might happen.",
    ),
    ("FOUND", "verb", "Discovered or located something."),
    ("HARD", "adjective", "Firm, difficult, or not easy."),
    ("NEAR", "preposition", "Close by."),
    ("WOMAN", "noun", "An adult female person."),
    (
        "WOULD",
        "helping verb",
        "A helping word used to tell what might happen.",
    ),
    ("WRITE", "verb", "To make words with letters."),
];

const MAX_CUSTOM_WORD_LEN: usize = 12;

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
            if is_key_pressed(KeyCode::Enter)
                || is_key_pressed(KeyCode::Space)
                || primary_tap_position().is_some()
            {
                *self = Self::new_with_mode(self.custom_words.clone(), self.nightmare_mode);
            }
            return ReadingSnakeAction::None;
        }

        if self.showing_definition_card {
            if is_key_pressed(KeyCode::Enter)
                || is_key_pressed(KeyCode::Space)
                || primary_tap_position().is_some()
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
        clear_background(Color::new(0.02, 0.04, 0.03, 1.0));
        self.draw_header();
        self.draw_board();
        self.draw_tiles();
        self.draw_snake();
        self.draw_footer();

        if self.game_over {
            draw_rectangle(
                0.0,
                0.0,
                SCREEN_W,
                SCREEN_H,
                Color::new(0.0, 0.0, 0.0, 0.75),
            );
            let title = if self.completed {
                "READING SNAKE COMPLETE"
            } else {
                "READING SNAKE OVER"
            };
            let title_color = if self.completed { YELLOW } else { RED };
            centered_text(title, 300.0, 42, title_color);
            centered_text(&format!("Final Score: {}", self.score), 360.0, 28, YELLOW);
            centered_text("Press ENTER to play again", 420.0, 22, WHITE);
            centered_text("Press ESC for title", 455.0, 18, GRAY);
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
        self.message = "Spell the word in order.";
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

        if let Some(tap) = primary_tap_position() {
            let head = self.snake[0];
            let head_center = vec2(
                BOARD_X + head.x as f32 * CELL + CELL / 2.0,
                BOARD_Y + head.y as f32 * CELL + CELL / 2.0,
            );
            let delta = tap - head_center;
            let next = if delta.x.abs() > delta.y.abs() {
                CellPos::new(delta.x.signum() as i32, 0)
            } else {
                CellPos::new(0, delta.y.signum() as i32)
            };

            if next.x != -self.dir.x || next.y != -self.dir.y {
                self.next_dir = next;
            }
        }
    }

    fn step(&mut self) {
        self.dir = self.next_dir;
        let head = self.snake[0];
        let next = CellPos::new(head.x + self.dir.x, head.y + self.dir.y);

        if next.x < 0
            || next.x >= GRID_W
            || next.y < 0
            || next.y >= GRID_H
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
        self.snake = vec![
            CellPos::new(GRID_W / 2, GRID_H / 2),
            CellPos::new(GRID_W / 2 - 1, GRID_H / 2),
            CellPos::new(GRID_W / 2 - 2, GRID_H / 2),
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
                random::i32_inclusive(0, GRID_W - 1),
                random::i32_inclusive(0, GRID_H - 1),
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
        let title_size = screen::mobile_text_size(36);
        let def_size = screen::mobile_text_size(20);
        let stat_size = screen::mobile_text_size(20);

        let title = if self.nightmare_mode {
            if self.bonus_round {
                "READING SNAKE BONUS NIGHTMARE"
            } else {
                "READING SNAKE NIGHTMARE"
            }
        } else {
            "READING SNAKE"
        };
        centered_text(title, 40.0, title_size, Color::new(0.4, 1.0, 0.65, 1.0));

        draw_text(&format!("Score: {}", self.score), 24.0, 72.0, stat_size as f32, YELLOW);
        draw_text(&format!("Lives: {}", self.lives), 680.0, 72.0, stat_size as f32, WHITE);

        centered_text(&format!("Definition: {}", self.definition), 100.0, def_size, WHITE);
    }

    fn draw_board(&self) {
        for x in 0..GRID_W {
            for y in 0..GRID_H {
                let color = if (x + y) % 2 == 0 {
                    Color::new(0.08, 0.16, 0.1, 0.9)
                } else {
                    Color::new(0.06, 0.13, 0.09, 0.9)
                };
                draw_rectangle(
                    BOARD_X + x as f32 * CELL,
                    BOARD_Y + y as f32 * CELL,
                    CELL,
                    CELL,
                    color,
                );
            }
        }
    }

    fn draw_tiles(&self) {
        let target_color = if self.nightmare_mode {
            Color::new(0.55, 0.75, 1.0, 1.0)
        } else {
            Color::new(1.0, 0.86, 0.2, 1.0)
        };
        let decoy_color = if self.nightmare_mode {
            target_color
        } else {
            Color::new(0.8, 0.25, 0.25, 1.0)
        };

        self.draw_tile(&self.target, target_color);
        for tile in &self.decoys {
            self.draw_tile(tile, decoy_color);
        }
    }

    fn draw_tile(&self, tile: &LetterTile, color: Color) {
        let letter_size = screen::mobile_text_size(36);
        let x = BOARD_X + tile.pos.x as f32 * CELL;
        let y = BOARD_Y + tile.pos.y as f32 * CELL;
        draw_rectangle(x + 2.0, y + 2.0, CELL - 4.0, CELL - 4.0, color);
        let letter = tile.letter.to_string();
        let metrics = measure_text(&letter, None, letter_size, 1.0);
        draw_text(
            &letter,
            x + CELL / 2.0 - metrics.width / 2.0,
            y + CELL / 2.0 + metrics.height / 2.5,
            letter_size as f32,
            BLACK,
        );
    }

    fn draw_snake(&self) {
        for (idx, part) in self.snake.iter().enumerate() {
            let x = BOARD_X + part.x as f32 * CELL;
            let y = BOARD_Y + part.y as f32 * CELL;
            let color = if idx == 0 {
                Color::new(0.45, 1.0, 0.55, 1.0)
            } else {
                Color::new(0.15, 0.75, 0.35, 1.0)
            };
            draw_rectangle(x + 3.0, y + 3.0, CELL - 6.0, CELL - 6.0, color);
        }
    }

    fn draw_footer(&self) {
        let progress_size = screen::mobile_text_size(26);
        let message_size = screen::mobile_text_size(22);
        let hint_size = screen::mobile_text_size(18);
        let controls_size = screen::mobile_text_size(16);

        let progress = format_word_progress(&self.word, self.letter_index);
        centered_text(&format!("Word: {}", progress), 580.0, progress_size, YELLOW);
        centered_text(self.message, 618.0, message_size, WHITE);
        centered_text(
            "Meaning: Read the card, then spell the word.",
            652.0,
            hint_size,
            WHITE,
        );
        let controls = if self.nightmare_mode {
            "Nightmare: all letters look alike   ESC returns to title"
        } else {
            "Arrow Keys / WASD to move   ESC returns to title"
        };
        centered_text(controls, 684.0, controls_size, GRAY);
    }

    fn draw_definition_card(&self) {
        let title_size = screen::mobile_text_size(60);
        let pos_size = screen::mobile_text_size(40);
        let def_size = screen::mobile_text_size(60);

        draw_rectangle(
            0.0,
            0.0,
            SCREEN_W,
            SCREEN_H,
            Color::new(0.0, 0.0, 0.0, 0.65),
        );
        draw_rectangle(
            157.0,
            180.0,
            710.0,
            380.0,
            Color::new(0.06, 0.14, 0.1, 0.98),
        );
        draw_rectangle_lines(
            157.0,
            180.0,
            710.0,
            380.0,
            4.0,
            Color::new(0.4, 1.0, 0.65, 1.0),
        );

        centered_text(self.definition_card_title, 245.0, title_size, YELLOW);
        centered_text(
            &format!("Part of speech: {}", self.part_of_speech),
            310.0,
            pos_size,
            Color::new(0.4, 1.0, 0.65, 1.0),
        );
        draw_wrapped_centered_text(&self.definition, 382.0, 610.0, def_size, WHITE);
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
                        definition.to_string()
                    },
                ))
            })
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
            .collect()
    }
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

fn draw_wrapped_centered_text(text: &str, y: f32, max_width: f32, font_size: u16, color: Color) {
    let mut line = String::new();
    let mut line_y = y;

    for word in text.split_whitespace() {
        let next = if line.is_empty() {
            word.to_string()
        } else {
            format!("{} {}", line, word)
        };

        if measure_text(&next, None, font_size, 1.0).width > max_width && !line.is_empty() {
            centered_text(&line, line_y, font_size, color);
            line = word.to_string();
            line_y += font_size as f32 + 8.0;
        } else {
            line = next;
        }
    }

    if !line.is_empty() {
        centered_text(&line, line_y, font_size, color);
    }
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

fn to_virtual_position(position: Vec2) -> Vec2 {
    vec2(
        position.x * SCREEN_W / screen_width().max(1.0),
        position.y * SCREEN_H / screen_height().max(1.0),
    )
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
}
