# Star Crusher

Star Crusher is an educational arcade collection wrapped in a light kid-friendly dungeon adventure menu. The current encounters include a Time Pilot-style Math Invaders game where drifting numbered targets display possible answers to grade-level math questions, Math Pong, and Reading Snake, a Snake-inspired mini game where players collect letters in order to spell words.

Current build: `1.4.6`

## Features

- Seven-grade progression from Preschool through 5th Grade.
- Grade-appropriate math questions covering counting, arithmetic, multiplication, division, fractions, percentages, pre-algebra, area, volume, and ratios.
- Start Adventure opens a guided campaign path: intro, first Math Invaders wave, Reading Snake, Math Pong, Nightmare Snake, then continued Math Invaders progression.
- Math Invaders waves with Time Pilot-style drifting numbered targets tied to the active math question.
- Math Invaders shows the active question in a larger top-centered banner, with targets kept below the banner.
- Preschool shape prompts use default-font-safe ASCII markers so shapes display reliably.
- Kindergarten number-recognition prompts use words, such as `Shoot number three`, while targets remain numeric.
- Question gates between waves that require typed answers to advance.
- Question gate prompts and answer input are spaced to avoid overlapping the wave-complete instructions.
- Math Pong mode for launching a straight ball into randomly placed numbered targets.
- Reading Snake mini game for letter order, word recognition, and definition practice, with randomized default or custom spelling lists and Nightmare mode.
- Reading Snake shows definition cards, keeps the active definition visible above the board, and keeps new letter tiles away from the snake head.
- Reading Snake definition cards show part of speech and use larger definition text for easier reading.
- Completing the standard Reading Snake list starts a bonus Nightmare round using the same words in the same randomized order.
- In Start Adventure, completing normal Reading Snake advances directly to Math Pong instead of the standalone bonus round.
- RPG-style title menu with procedural stone paneling, dungeon glyphs, a focused main adventure menu, and a Play Mini Games submenu.
- Game over and victory stat panels are centered with their score and progress text.
- Procedural graphics only; no external assets or fonts required.
- Launches in a 1920x1080 fullscreen window with a fixed 1024x768 virtual playfield.

## Controls

Title menu controls:

- Move menu cursor: `Up` / `Down` arrow keys or `W` / `S`
- Launch selected option: `Enter` or `Space`
- Main menu options: `Start Adventure`, `Play Mini Games`, and `Custom Spelling List`
- Play Mini Games options: `Reading Snake`, `Math Pong`, and `Nightmare Snake`
- Return from Play Mini Games to the main menu: `Esc`
- Continue Start Adventure intro: `Enter` or `Space`
- Return from Start Adventure intro to title: `Esc`
- Return from adventure mini-games to title and cancel the adventure: `Esc`
- Direct shortcut for Math Invaders: `M`
- Direct shortcut for Play Mini Games from the main menu: `P`
- Direct shortcut for Math Pong from Play Mini Games: `P`
- Direct shortcut for Reading Snake: `R`
- Direct shortcut for Reading Snake Nightmare: `N`
- Direct shortcut for Custom Spelling List: `L`

Math Invaders controls:

- Move: `Left` / `Right` arrow keys or `A` / `D`
- Shoot: `Space`
- Start / continue: `Enter` or `Space`
- Return from mini games to title: `Esc`
- Type gate answers with number keys, then press `Enter`
- Delete typed answer characters with `Backspace`

Reading Snake controls:

- Move: arrow keys or `W` / `A` / `S` / `D`
- Start spelling after a definition card: `Enter` or `Space`
- Restart after game over: `Enter` or `Space`
- Return to title: `Esc`

Reading Snake layout and safety:

- The definition card shows the part of speech before each word, and the definition remains visible above the playfield.
- The blank word prompt appears below the playfield.
- After each correct letter, the next target and decoy letters avoid a 6-by-6 area around the snake head.

Reading Snake Nightmare rules:

- Start from title: `N`
- All letter tiles use the same color
- Wrong letters cost one life
- Completing a nightmare word awards one bonus life, up to 9 lives

Spelling-list entry controls:

- Start list entry from title: `L`
- Type `word: definition` pairs separated by semicolons, then press `Enter`
- Press `N` from list entry to play Nightmare with the typed list
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

Or use the included launcher:

```bash
./run-game
```

## Check Compilation

```bash
cargo check
```

## Project Structure

```text
run-game         Convenience launcher that loads rustup environment and runs Cargo
src/main.rs      Game state machine and update/draw loop
src/screen.rs    Window configuration, fullscreen launch, and virtual screen camera
src/levels.rs    Grade progression and difficulty configuration
src/question.rs  Grade-specific math question generation
src/math_pong.rs Math Pong number target mini game
src/reading_snake.rs Reading Snake mini game
src/enemy.rs     Numbered Math Invaders targets, movement, explosions
src/player.rs    Player ship, player bullets, enemy bullets
src/ui.rs        HUD, title, game over, victory, and question gate UI
src/assets.rs    Procedural drawing helpers for ships, enemies, stars, effects
```

## Gameplay Loop

Math Invaders:

1. Choose `Start Adventure` to see the RPG-style intro, then press `Enter` or `Space` through the final prompt to begin.
2. Clear the first Math Invaders wave to enter normal Reading Snake automatically.
3. Complete normal Reading Snake to enter Math Pong automatically.
4. Complete Math Pong to enter Nightmare Snake automatically.
5. Complete Nightmare Snake to return to Math Invaders progression and answer the wave-complete gate.
6. Choose `Math Invaders` from the title menu, or press `M`, to launch standalone Math Invaders immediately.
7. Read the active math question and find the drifting target showing the correct answer.
8. Use the top-centered question banner; targets spawn and drift below it for visibility.
9. Shoot the correct drifting number to score and receive a new question for the remaining targets.
10. Shooting an incorrect number costs one life and leaves that target in play.
11. Clear all numbered targets, then answer typed math questions at the wave-complete gate.
12. Advance through each grade until the 5th Grade wave is completed.

Math Pong:

1. Choose `Play Mini Games`, then choose `Math Pong`, or press `P` from Play Mini Games.
2. Read the math question and identify the correct numbered target.
3. Move the paddle under the correct number before launching the ball.
4. Launch straight upward into the correct number to clear the question.
5. Clear five questions to advance to the next grade.

Reading Snake:

1. Choose `Play Mini Games`, then choose `Reading Snake`, or press `R`, to play with the default word list.
2. Or choose `Custom Spelling List`, type weekly spelling words with definitions, then press `Enter`.
3. Use the format `apple: a fruit; moon: shines at night` for custom definitions.
4. Read the definition card, then press `Enter` or `Space` to start spelling.
5. Use the visible definition above the board and follow the blank word prompt below the board.
6. Steer the snake into the next correct letter.
7. Avoid wrong letters, walls, and the snake's own tail.
8. New letters appear away from the snake head so the player has room to react.
9. Complete every word in the randomized list to unlock a bonus Nightmare pass through those same words in the same order.

Reading Snake Nightmare:

1. Choose `Play Mini Games`, then choose `Nightmare Snake`, or press `N`.
2. Or choose `Custom Spelling List`, type a custom spelling list, then press `N`.
3. Read the definition card and spell the hidden word.
4. Choose carefully because all letter tiles look the same.
5. Complete the word to earn a bonus life.

## Development Notes

- This is a binary Rust project, so `Cargo.lock` is intentionally committed.
- Build output in `target/` is ignored.
- The game uses `macroquad` for windowing, input, and drawing, and `rand` for question/enemy randomization.
