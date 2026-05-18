use ::rand::seq::SliceRandom;
use ::rand::Rng;
use macroquad::prelude::*;

const SCREEN_W: f32 = 1024.0;
const SCREEN_H: f32 = 768.0;
const GRID_W: i32 = 34;
const GRID_H: i32 = 21;
const CELL: f32 = 24.0;
const BOARD_X: f32 = 104.0;
const BOARD_Y: f32 = 110.0;
const STEP_SECONDS: f64 = 0.18;
const SNAKE_HEAD_SAFE_RADIUS: i32 = 3;
const MAX_LIVES: u8 = 9;

const WORDS: &[(&str, &str)] = &[
    ("CAT", "A small furry pet that says meow."),
    ("DOG", "A friendly pet that often barks."),
    ("SUN", "The bright star that gives Earth light."),
    ("MAP", "A picture that helps you find places."),
    ("BOOK", "Pages with words or pictures to read."),
    ("STAR", "A bright light seen in the night sky."),
    ("MOON", "The round object that shines at night."),
    ("PLANT", "A living thing that grows from soil."),
    ("SPACE", "The huge area beyond Earth."),
    ("ROCKET", "A ship that can fly into space."),
    ("PLANET", "A large round world that moves around a star."),
    ("GALAXY", "A giant group of stars."),
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
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WordEntry {
    word: String,
    definition: String,
}

impl WordEntry {
    fn from_parts(word: String, definition: String) -> Self {
        Self { word, definition }
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
    definition: String,
    custom_words: Vec<WordEntry>,
    letter_index: usize,
    score: u32,
    lives: u8,
    nightmare_mode: bool,
    last_step: f64,
    game_over: bool,
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
            definition: String::new(),
            custom_words,
            letter_index: 0,
            score: 0,
            lives: 3,
            nightmare_mode,
            last_step: get_time(),
            game_over: false,
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
            if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                *self = Self::new_with_mode(self.custom_words.clone(), self.nightmare_mode);
            }
            return ReadingSnakeAction::None;
        }

        if self.showing_definition_card {
            if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
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
            centered_text("READING SNAKE OVER", 300.0, 42, RED);
            centered_text(&format!("Final Score: {}", self.score), 360.0, 28, YELLOW);
            centered_text("Press ENTER to play again", 420.0, 22, WHITE);
            centered_text("Press ESC for title", 455.0, 18, GRAY);
        } else if self.showing_definition_card {
            self.draw_definition_card();
        }
    }

    fn reset_run(&mut self) {
        self.reset_snake_to_spawn();
        self.pick_word();
    }

    fn pick_word(&mut self) {
        let mut rng = ::rand::thread_rng();
        let word_entry = self
            .custom_words
            .choose(&mut rng)
            .cloned()
            .unwrap_or_else(|| default_word_entry(&mut rng));
        self.word = word_entry.word;
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
                self.pick_word();
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
            let mut rng = ::rand::thread_rng();
            self.decoys.push(LetterTile {
                pos: self.random_empty_cell(&occupied),
                letter: *alphabet.choose(&mut rng).unwrap_or(&'Z'),
            });
        }
    }

    fn random_empty_cell(&self, reserved: &[CellPos]) -> CellPos {
        let mut rng = ::rand::thread_rng();
        loop {
            let pos = CellPos::new(rng.gen_range(0..GRID_W), rng.gen_range(0..GRID_H));
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
        let title = if self.nightmare_mode {
            "READING SNAKE NIGHTMARE"
        } else {
            "READING SNAKE"
        };
        centered_text(title, 48.0, 40, Color::new(0.4, 1.0, 0.65, 1.0));
        centered_text(&format!("Definition: {}", self.definition), 82.0, 18, WHITE);

        draw_text(&format!("Score: {}", self.score), 28.0, 40.0, 22.0, YELLOW);
        draw_text(&format!("Lives: {}", self.lives), 870.0, 40.0, 22.0, WHITE);
    }

    fn draw_board(&self) {
        draw_rectangle(
            BOARD_X - 8.0,
            BOARD_Y - 8.0,
            GRID_W as f32 * CELL + 16.0,
            GRID_H as f32 * CELL + 16.0,
            Color::new(0.05, 0.12, 0.08, 1.0),
        );
        draw_rectangle_lines(
            BOARD_X - 8.0,
            BOARD_Y - 8.0,
            GRID_W as f32 * CELL + 16.0,
            GRID_H as f32 * CELL + 16.0,
            3.0,
            Color::new(0.25, 0.75, 0.4, 1.0),
        );

        for x in 0..GRID_W {
            for y in 0..GRID_H {
                let color = if (x + y) % 2 == 0 {
                    Color::new(0.08, 0.16, 0.1, 1.0)
                } else {
                    Color::new(0.06, 0.13, 0.09, 1.0)
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
        let x = BOARD_X + tile.pos.x as f32 * CELL;
        let y = BOARD_Y + tile.pos.y as f32 * CELL;
        draw_rectangle(x + 2.0, y + 2.0, CELL - 4.0, CELL - 4.0, color);
        let letter = tile.letter.to_string();
        let metrics = measure_text(&letter, None, 22, 1.0);
        draw_text(
            &letter,
            x + CELL / 2.0 - metrics.width / 2.0,
            y + CELL / 2.0 + metrics.height / 2.5,
            22.0,
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
        let progress = format_word_progress(&self.word, self.letter_index);
        centered_text(&format!("Word: {}", progress), 675.0, 30, YELLOW);
        centered_text(
            "Meaning: Read the card, then spell the word.",
            710.0,
            18,
            WHITE,
        );
        centered_text(self.message, 735.0, 18, WHITE);
        let controls = if self.nightmare_mode {
            "Nightmare: all letters look alike   ESC returns to title"
        } else {
            "Arrow Keys / WASD to move   ESC returns to title"
        };
        centered_text(controls, 760.0, 16, GRAY);
    }

    fn draw_definition_card(&self) {
        draw_rectangle(
            0.0,
            0.0,
            SCREEN_W,
            SCREEN_H,
            Color::new(0.0, 0.0, 0.0, 0.65),
        );
        draw_rectangle(
            237.0,
            220.0,
            550.0,
            280.0,
            Color::new(0.06, 0.14, 0.1, 0.98),
        );
        draw_rectangle_lines(
            237.0,
            220.0,
            550.0,
            280.0,
            4.0,
            Color::new(0.4, 1.0, 0.65, 1.0),
        );

        centered_text(self.definition_card_title, 275.0, 30, YELLOW);
        centered_text("Definition", 320.0, 24, Color::new(0.4, 1.0, 0.65, 1.0));
        draw_wrapped_centered_text(&self.definition, 362.0, 460.0, 24, WHITE);
        centered_text("Press SPACE or ENTER when ready", 460.0, 20, GRAY);
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
                Some(WordEntry::from_parts(word, definition))
            })
            .collect()
    }
}

fn default_word_entry<R: Rng + ?Sized>(rng: &mut R) -> WordEntry {
    let (word, definition) = WORDS
        .choose(rng)
        .copied()
        .unwrap_or(("STAR", "A bright light seen in the night sky."));
    WordEntry::from_parts(word.to_string(), definition.to_string())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_custom_words_from_spaces_commas_and_lines() {
        assert_eq!(
            custom_words_from_input("cat, dog\nmoon sun"),
            vec![
                WordEntry::from_parts(
                    "CAT".to_string(),
                    "Practice spelling the word CAT.".to_string()
                ),
                WordEntry::from_parts(
                    "DOG".to_string(),
                    "Practice spelling the word DOG.".to_string()
                ),
                WordEntry::from_parts(
                    "MOON".to_string(),
                    "Practice spelling the word MOON.".to_string()
                ),
                WordEntry::from_parts(
                    "SUN".to_string(),
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
                    "Practice spelling the word ROCKET.".to_string()
                ),
                WordEntry::from_parts(
                    "SUPERCALIFRA".to_string(),
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
                WordEntry::from_parts("APPLE".to_string(), "a fruit".to_string()),
                WordEntry::from_parts("MOON".to_string(), "shines at night".to_string()),
            ]
        );
    }
}
