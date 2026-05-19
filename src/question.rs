use crate::levels::Grade;
use rand::Rng;

/// A math question with its correct answer and distractor options.
#[derive(Debug, Clone)]
pub struct Question {
    pub text: String,
    pub correct_answer: i64,
    /// Wrong answers used as distractors on puzzle enemies.
    pub wrong_answers: Vec<i64>,
}

/// Generates a question appropriate for the given grade level.
pub fn generate_question(grade: Grade) -> Question {
    match grade {
        Grade::Preschool => gen_preschool(),
        Grade::Kindergarten => gen_kindergarten(),
        Grade::FirstGrade => gen_first_grade(),
        Grade::SecondGrade => gen_second_grade(),
        Grade::ThirdGrade => gen_third_grade(),
        Grade::FourthGrade => gen_fourth_grade(),
        Grade::FifthGrade => gen_fifth_grade(),
    }
}

/// Preschool: Count objects (1-5). Simple visual counting.
fn gen_preschool() -> Question {
    let mut rng = rand::thread_rng();
    let count = rng.gen_range(1..=5);
    let shapes = ["star", "circle", "square", "heart"];
    let shape = shapes[rng.gen_range(0..shapes.len())];

    // Build a visual representation like:  ★ ★ ★
    let symbols = match shape {
        "star" => "★ ",
        "circle" => "● ",
        "square" => "■ ",
        _ => "♥ ",
    };
    let display = (0..count).map(|_| symbols).collect::<String>();

    Question {
        text: format!("How many {}?\n{}", shape, display),
        correct_answer: count as i64,
        wrong_answers: gen_unique_wrongs(count as i64, 1, 5, 3),
    }
}

/// Kindergarten: Numbers 1-10, simple addition within 5.
fn gen_kindergarten() -> Question {
    let mut rng = rand::thread_rng();
    if rng.gen_bool(0.5) {
        // Simple addition within 5
        let a = rng.gen_range(0..=3);
        let b = rng.gen_range(1..=(5 - a));
        Question {
            text: format!("{} + {} = ?", a, b),
            correct_answer: (a + b) as i64,
            wrong_answers: gen_unique_wrongs((a + b) as i64, 0, 10, 3),
        }
    } else {
        // Counting / number recognition
        let n = rng.gen_range(1..=10);
        Question {
            text: format!("Shoot number {}", number_word(n)),
            correct_answer: n as i64,
            wrong_answers: gen_unique_wrongs(n as i64, 1, 10, 3),
        }
    }
}

/// First Grade: Addition and subtraction within 20.
fn gen_first_grade() -> Question {
    let mut rng = rand::thread_rng();
    if rng.gen_bool(0.5) {
        // Addition within 20
        let a = rng.gen_range(1..=15);
        let b = rng.gen_range(1..=(20 - a));
        Question {
            text: format!("{} + {} = ?", a, b),
            correct_answer: (a + b) as i64,
            wrong_answers: gen_unique_wrongs((a + b) as i64, 1, 20, 3),
        }
    } else {
        // Subtraction within 20
        let a = rng.gen_range(5..=20);
        let b = rng.gen_range(1..=(a));
        Question {
            text: format!("{} - {} = ?", a, b),
            correct_answer: (a - b) as i64,
            wrong_answers: gen_unique_wrongs((a - b) as i64, 0, 20, 3),
        }
    }
}

/// Second Grade: Multiplication tables x1-x5, addition within 100.
fn gen_second_grade() -> Question {
    let mut rng = rand::thread_rng();
    if rng.gen_bool(0.6) {
        // Multiplication (factors up to 5)
        let a = rng.gen_range(1..=5);
        let b = rng.gen_range(1..=10);
        Question {
            text: format!("{} × {} = ?", a, b),
            correct_answer: (a * b) as i64,
            wrong_answers: gen_unique_wrongs((a * b) as i64, 1, 50, 3),
        }
    } else {
        // Addition within 100
        let a = rng.gen_range(10..=80);
        let b = rng.gen_range(1..=(99 - a));
        Question {
            text: format!("{} + {} = ?", a, b),
            correct_answer: (a + b) as i64,
            wrong_answers: gen_unique_wrongs((a + b) as i64, 10, 100, 3),
        }
    }
}

/// Third Grade: Division, fractions basics, multiplication up to 12.
fn gen_third_grade() -> Question {
    let mut rng = rand::thread_rng();
    let variant = rng.gen_range(0..=2);
    match variant {
        // Multiplication tables up to 12
        0 => {
            let a = rng.gen_range(2..=12);
            let b = rng.gen_range(2..=12);
            Question {
                text: format!("{} × {} = ?", a, b),
                correct_answer: (a * b) as i64,
                wrong_answers: gen_unique_wrongs((a * b) as i64, 4, 144, 3),
            }
        }
        // Division with clean answers
        1 => {
            let divisor = rng.gen_range(2..=9);
            let quotient = rng.gen_range(2..=10);
            let dividend = divisor * quotient;
            Question {
                text: format!("{} ÷ {} = ?", dividend, divisor),
                correct_answer: quotient as i64,
                wrong_answers: gen_unique_wrongs(quotient as i64, 1, 20, 3),
            }
        }
        // Fraction basics (numerator identification)
        _ => {
            let denom = rng.gen_range(2..=8);
            let numer = rng.gen_range(1..denom);
            Question {
                text: format!("What is the numerator of {} / {}?", numer, denom),
                correct_answer: numer as i64,
                wrong_answers: gen_unique_wrongs(numer as i64, 0, 8, 3),
            }
        }
    }
}

/// Fourth Grade: Decimals, percentages, multi-step problems.
fn gen_fourth_grade() -> Question {
    let mut rng = rand::thread_rng();
    let variant = rng.gen_range(0..=2);
    match variant {
        // Percentages of round numbers
        0 => {
            let pct_choices = [10, 25, 50];
            let pct = pct_choices[rng.gen_range(0..pct_choices.len())];
            let base = rng.gen_range(1..=20) * 4; // multiples of 4 for clean answers
            Question {
                text: format!("What is {}% of {}?", pct, base),
                correct_answer: ((pct as i64 * base as i64) / 100),
                wrong_answers: gen_unique_wrongs(
                    (pct as i64 * base as i64) / 100,
                    1,
                    (base * 2) as i64,
                    3,
                ),
            }
        }
        // Decimal addition
        1 => {
            let a = rng.gen_range(10..=99);
            let b = rng.gen_range(10..=99);
            let sum = a + b;
            Question {
                text: format!("{} hundredths + {} hundredths = ?", a, b),
                correct_answer: sum as i64,
                wrong_answers: gen_unique_wrongs(sum as i64, 10, 198, 3),
            }
        }
        // Multi-step: e.g., "What is 8 × 7 - 5?"
        _ => {
            let a = rng.gen_range(2..=9);
            let b = rng.gen_range(2..=9);
            let c = rng.gen_range(1..=(a * b));
            Question {
                text: format!("{} × {} - {} = ?", a, b, c),
                correct_answer: (a * b - c) as i64,
                wrong_answers: gen_unique_wrongs((a * b - c) as i64, 0, 81, 3),
            }
        }
    }
}

/// Fifth Grade: Pre-algebra, area/volume, ratios.
fn gen_fifth_grade() -> Question {
    let mut rng = rand::thread_rng();
    let variant = rng.gen_range(0..=2);
    match variant {
        // Solve for x in simple linear equations: ax + b = c
        0 => {
            let a = rng.gen_range(1..=5);
            let x = rng.gen_range(1..=9);
            let b = rng.gen_range(1..=20);
            let c = a * x + b;
            Question {
                text: format!("If {}x + {} = {}, what is x?", a, b, c),
                correct_answer: x as i64,
                wrong_answers: gen_unique_wrongs(x as i64, 1, 20, 3),
            }
        }
        // Area of rectangle / triangle
        1 => {
            let shape = rng.gen_bool(0.5);
            if shape {
                // Rectangle area
                let w = rng.gen_range(2..=12);
                let h = rng.gen_range(2..=12);
                Question {
                    text: format!("Area of rectangle {} × {}?", w, h),
                    correct_answer: (w * h) as i64,
                    wrong_answers: gen_unique_wrongs((w * h) as i64, 1, 200, 3),
                }
            } else {
                // Triangle area with even base for clean division
                let b = rng.gen_range(2..=12) * 2;
                let h = rng.gen_range(2..=10);
                Question {
                    text: format!("Area of triangle (base={}, height={})?", b, h),
                    correct_answer: ((b * h) / 2) as i64,
                    wrong_answers: gen_unique_wrongs(((b * h) / 2) as i64, 1, 200, 3),
                }
            }
        }
        // Ratios: "If the ratio of A:B is 3:5 and total is 24, what is B?"
        _ => {
            let a_part = rng.gen_range(1..=4);
            let b_part = rng.gen_range(a_part + 1..=(a_part * 3));
            let multiplier = rng.gen_range(1..=6);
            let total = (a_part + b_part) * multiplier;
            Question {
                text: format!(
                    "Ratio A:B is {}:{}, total is {}. What is B?",
                    a_part, b_part, total
                ),
                correct_answer: (b_part * multiplier) as i64,
                wrong_answers: gen_unique_wrongs((b_part * multiplier) as i64, 1, total as i64, 3),
            }
        }
    }
}

fn number_word(number: i32) -> &'static str {
    match number {
        1 => "one",
        2 => "two",
        3 => "three",
        4 => "four",
        5 => "five",
        6 => "six",
        7 => "seven",
        8 => "eight",
        9 => "nine",
        10 => "ten",
        _ => "number",
    }
}

/// Generates `count` unique wrong answers in range [min_val, max_val] that differ from correct.
fn gen_unique_wrongs(correct: i64, min_val: i64, max_val: i64, count: usize) -> Vec<i64> {
    let mut rng = rand::thread_rng();
    let mut candidates: Vec<i64> = (min_val..=max_val)
        .filter(|value| *value != correct)
        .collect();

    while candidates.len() < count {
        let offset = candidates.len() as i64 + 1;
        for value in [correct - offset, correct + offset] {
            if value != correct && !candidates.contains(&value) {
                candidates.push(value);
            }
            if candidates.len() >= count {
                break;
            }
        }
    }

    for i in (1..candidates.len()).rev() {
        let j = rng.gen_range(0..=i);
        candidates.swap(i, j);
    }

    candidates.truncate(count);
    candidates
}
