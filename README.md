# Star Crusher

Star Crusher is an educational arcade collection for K-5 practice. The main game is a Space Invaders-style math game where enemies display possible answers to grade-level math questions. It also includes Reading Snake, a Snake-inspired mini game where players collect letters in order to spell words.

## Features

- Seven-grade progression from Preschool through 5th Grade.
- Grade-appropriate math questions covering counting, arithmetic, multiplication, division, fractions, percentages, pre-algebra, area, volume, and ratios.
- Space Invaders-inspired enemy waves with increasing grid size, speed, and fire rate.
- Question gates between waves that require typed answers to advance.
- Reading Snake mini game for letter order and word recognition practice.
- Procedural graphics only; no external assets or fonts required.
- Fixed 800x600 macroquad viewport.

## Controls

- Move: `Left` / `Right` arrow keys or `A` / `D`
- Shoot: `Space`
- Start / continue: `Enter` or `Space`
- Start Reading Snake from title: `R`
- Return from Reading Snake to title: `Esc`
- Type gate answers with number keys, then press `Enter`
- Delete typed answer characters with `Backspace`

Reading Snake controls:

- Move: arrow keys or `W` / `A` / `S` / `D`
- Restart after game over: `Enter` or `Space`
- Return to title: `Esc`

## Requirements

- Rust 2021 toolchain
- Cargo

Install Rust from <https://www.rust-lang.org/tools/install> if needed.

## Run The Game

```bash
cargo run
```

## Check Compilation

```bash
cargo check
```

## Project Structure

```text
src/main.rs      Game entry point, state machine, update/draw loop
src/levels.rs    Grade progression and difficulty configuration
src/question.rs  Grade-specific math question generation
src/reading_snake.rs Reading Snake mini game
src/enemy.rs     Enemy grid, puzzle enemies, enemy movement, explosions
src/player.rs    Player ship, player bullets, enemy bullets
src/ui.rs        HUD, title, game over, victory, and question gate UI
src/assets.rs    Procedural drawing helpers for ships, enemies, stars, effects
```

## Gameplay Loop

Math Invaders:

1. Start from the title screen.
2. Clear the current enemy wave by shooting enemies.
3. Use the active math question to identify correct puzzle enemies.
4. Answer typed math questions at the wave-complete gate.
5. Advance through each grade until the 5th Grade wave is completed.

Reading Snake:

1. Press `R` on the title screen.
2. Follow the word prompt at the bottom of the screen.
3. Steer the snake into the next correct letter.
4. Avoid wrong letters, walls, and the snake's own tail.
5. Complete words to earn bonus points and receive a new word.

## Development Notes

- This is a binary Rust project, so `Cargo.lock` is intentionally committed.
- Build output in `target/` is ignored.
- The game uses `macroquad` for windowing, input, and drawing, and `rand` for question/enemy randomization.
