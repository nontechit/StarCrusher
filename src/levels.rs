use macroquad::prelude::*;

/// Represents each grade level from Preschool through 5th Grade.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Grade {
    Preschool,
    Kindergarten,
    FirstGrade,
    SecondGrade,
    ThirdGrade,
    FourthGrade,
    FifthGrade,
}

impl Grade {
    /// Display name for the grade level.
    pub fn display_name(&self) -> &'static str {
        match self {
            Grade::Preschool => "Preschool",
            Grade::Kindergarten => "Kindergarten",
            Grade::FirstGrade => "1st Grade",
            Grade::SecondGrade => "2nd Grade",
            Grade::ThirdGrade => "3rd Grade",
            Grade::FourthGrade => "4th Grade",
            Grade::FifthGrade => "5th Grade",
        }
    }

    /// Returns the next grade, or None if already at 5th.
    pub fn next(&self) -> Option<Grade> {
        match self {
            Grade::Preschool => Some(Grade::Kindergarten),
            Grade::Kindergarten => Some(Grade::FirstGrade),
            Grade::FirstGrade => Some(Grade::SecondGrade),
            Grade::SecondGrade => Some(Grade::ThirdGrade),
            Grade::ThirdGrade => Some(Grade::FourthGrade),
            Grade::FourthGrade => Some(Grade::FifthGrade),
            Grade::FifthGrade => None,
        }
    }

    /// Index position (0 = Preschool, 6 = Fifth).
    pub fn index(&self) -> usize {
        match self {
            Grade::Preschool => 0,
            Grade::Kindergarten => 1,
            Grade::FirstGrade => 2,
            Grade::SecondGrade => 3,
            Grade::ThirdGrade => 4,
            Grade::FourthGrade => 5,
            Grade::FifthGrade => 6,
        }
    }

    /// Primary color for enemies at this grade level.
    pub fn enemy_color(&self) -> Color {
        match self {
            Grade::Preschool => RED,
            Grade::Kindergarten => ORANGE,
            Grade::FirstGrade => YELLOW,
            Grade::SecondGrade => GREEN,
            Grade::ThirdGrade => Color::new(0.0, 1.0, 1.0, 1.0),
            Grade::FourthGrade => BLUE,
            Grade::FifthGrade => PURPLE,
        }
    }

    /// Returns the LevelConfig for this grade.
    pub fn config(&self) -> LevelConfig {
        match self {
            Grade::Preschool => LevelConfig {
                rows: 2,
                cols: 4,
                enemy_move_speed: 0.3,
                fire_interval_ms: 800,
                question_gate_count: 1,
            },
            Grade::Kindergarten => LevelConfig {
                rows: 2,
                cols: 5,
                enemy_move_speed: 0.4,
                fire_interval_ms: 700,
                question_gate_count: 1,
            },
            Grade::FirstGrade => LevelConfig {
                rows: 3,
                cols: 6,
                enemy_move_speed: 0.5,
                fire_interval_ms: 600,
                question_gate_count: 1,
            },
            Grade::SecondGrade => LevelConfig {
                rows: 3,
                cols: 7,
                enemy_move_speed: 0.6,
                fire_interval_ms: 500,
                question_gate_count: 2,
            },
            Grade::ThirdGrade => LevelConfig {
                rows: 4,
                cols: 8,
                enemy_move_speed: 0.7,
                fire_interval_ms: 450,
                question_gate_count: 2,
            },
            Grade::FourthGrade => LevelConfig {
                rows: 4,
                cols: 9,
                enemy_move_speed: 0.8,
                fire_interval_ms: 400,
                question_gate_count: 2,
            },
            Grade::FifthGrade => LevelConfig {
                rows: 5,
                cols: 10,
                enemy_move_speed: 1.0,
                fire_interval_ms: 350,
                question_gate_count: 3,
            },
        }
    }

    /// Math topics taught at this grade (displayed on gate screen).
    pub fn math_topics(&self) -> &'static str {
        match self {
            Grade::Preschool => "Counting objects (1-5)",
            Grade::Kindergarten => "Numbers 1-10, simple addition",
            Grade::FirstGrade => "Addition & subtraction within 20",
            Grade::SecondGrade => "Multiplication tables x1-x5",
            Grade::ThirdGrade => "Division, fractions basics",
            Grade::FourthGrade => "Decimals, percentages",
            Grade::FifthGrade => "Pre-algebra, area/volume, ratios",
        }
    }
}

/// Configuration for a grade level's gameplay difficulty.
#[derive(Debug, Clone)]
pub struct LevelConfig {
    /// Number of rows in the enemy grid.
    pub rows: usize,
    /// Number of columns in the enemy grid.
    pub cols: usize,
    /// Horizontal movement speed multiplier (pixels per frame base).
    pub enemy_move_speed: f32,
    /// Minimum interval between enemy fire attempts (milliseconds).
    pub fire_interval_ms: u64,
    /// Number of questions in the gate after clearing this wave.
    pub question_gate_count: usize,
}
