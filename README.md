# Star Crusher

Star Crusher is an educational arcade collection for K-5 practice. The main game is a Space Invaders-style math game where enemies display possible answers to grade-level math questions. It also includes Math Pong, a paddle-and-ball number target mode, and Reading Snake, a Snake-inspired mini game where players collect letters in order to spell words.

## Features

- Seven-grade progression from Preschool through 5th Grade.
- Grade-appropriate math questions covering counting, arithmetic, multiplication, division, fractions, percentages, pre-algebra, area, volume, and ratios.
- Space Invaders-inspired enemy waves with increasing grid size, speed, and fire rate.
- Question gates between waves that require typed answers to advance.
- Math Pong mode for launching a straight ball into randomly placed numbered targets.
- Reading Snake mini game for letter order, word recognition, and definition practice, with optional custom weekly spelling lists.
- Procedural graphics only; no external assets or fonts required.
- Fixed 800x600 macroquad viewport.

## Controls

- Move: `Left` / `Right` arrow keys or `A` / `D`
- Shoot: `Space`
- Start / continue: `Enter` or `Space`
- Start Math Pong from title: `P`
- Start Reading Snake from title: `R`
- Type a Reading Snake spelling list from title: `L`
- Return from mini games to title: `Esc`
- Type gate answers with number keys, then press `Enter`
- Delete typed answer characters with `Backspace`

Reading Snake controls:

- Move: arrow keys or `W` / `A` / `S` / `D`
- Start spelling after a definition card: `Enter` or `Space`
- Restart after game over: `Enter` or `Space`
- Return to title: `Esc`

Spelling-list entry controls:

- Start list entry from title: `L`
- Type `word: definition` pairs separated by semicolons, then press `Enter`
- Plain word lists separated by spaces or commas still work
- Delete typed characters with `Backspace`
- Leave the list blank and press `Enter` to use the default words
- Return to title without starting: `Esc`

Math Pong controls:

- Move paddle: `Left` / `Right` arrow keys or `A` / `D`
- Launch ball: `Space` or `Enter`
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
src/math_pong.rs Math Pong number target mini game
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

Math Pong:

1. Press `P` on the title screen.
2. Read the math question and identify the correct numbered target.
3. Move the paddle under the correct number before launching the ball.
4. Launch straight upward into the correct number to clear the question.
5. Clear five questions to advance to the next grade.

Reading Snake:

1. Press `R` on the title screen to play with the default word list.
2. Or press `L`, type weekly spelling words with definitions, then press `Enter`.
3. Use the format `apple: a fruit; moon: shines at night` for custom definitions.
4. Read the definition card, then press `Enter` or `Space` to start spelling.
5. Follow the blank word prompt at the bottom of the screen.
6. Steer the snake into the next correct letter.
7. Avoid wrong letters, walls, and the snake's own tail.
8. Complete words to earn bonus points and see the next definition card.

## Development Notes

- This is a binary Rust project, so `Cargo.lock` is intentionally committed.
- Build output in `target/` is ignored.
- The game uses `macroquad` for windowing, input, and drawing, and `rand` for question/enemy randomization.
