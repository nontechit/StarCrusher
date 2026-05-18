# Star Crusher

Star Crusher is an educational Space Invaders-style arcade game for K-5 math practice. Enemies display possible answers to grade-level math questions, and the player shoots the correct answer while avoiding enemy fire and wrong-answer targets.

## Features

- Seven-grade progression from Preschool through 5th Grade.
- Grade-appropriate math questions covering counting, arithmetic, multiplication, division, fractions, percentages, pre-algebra, area, volume, and ratios.
- Space Invaders-inspired enemy waves with increasing grid size, speed, and fire rate.
- Question gates between waves that require typed answers to advance.
- Procedural graphics only; no external assets or fonts required.
- Fixed 800x600 macroquad viewport.

## Controls

- Move: `Left` / `Right` arrow keys or `A` / `D`
- Shoot: `Space`
- Start / continue: `Enter` or `Space`
- Type gate answers with number keys, then press `Enter`
- Delete typed answer characters with `Backspace`

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
src/enemy.rs     Enemy grid, puzzle enemies, enemy movement, explosions
src/player.rs    Player ship, player bullets, enemy bullets
src/ui.rs        HUD, title, game over, victory, and question gate UI
src/assets.rs    Procedural drawing helpers for ships, enemies, stars, effects
```

## Gameplay Loop

1. Start from the title screen.
2. Clear the current enemy wave by shooting enemies.
3. Use the active math question to identify correct puzzle enemies.
4. Answer typed math questions at the wave-complete gate.
5. Advance through each grade until the 5th Grade wave is completed.

## Development Notes

- This is a binary Rust project, so `Cargo.lock` is intentionally committed.
- Build output in `target/` is ignored.
- The game uses `macroquad` for windowing, input, and drawing, and `rand` for question/enemy randomization.
