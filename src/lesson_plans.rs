// Lesson plan data, compiled at build time from D:/lesson-plans/.
//
// We don't read JSON from disk at runtime (the game runs in WASM where there
// is no filesystem). Instead, the curated content from each lesson plan is
// transcribed into the `&'static [LessonPlan]` slices below. The original
// JSON id is preserved on each entry so the link to the source plan stays
// explicit and an automated check could re-verify it later.
//
// Source layout: D:/lesson-plans/<Grade>/<Subject>/<ID>.json
//
// Pre-Kindergarten content is hand-tuned below (Frog Lane special-cases the
// PK-MATH-* ids for bespoke rendering). Kindergarten through 5th Grade live
// in `lesson_plans_gen.rs`, generated from the corpus by
// tools/generate-lesson-plans-rs.mjs — regenerate rather than editing it.

use crate::lesson_plans_gen as gen;
use crate::levels::Grade;

// ─────────────────────────────────────────────────────────────────────────────
//  Types
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Subject {
    Mathematics,
    Literacy,
}

/// Math concept family — drives Frog Lane's per-lesson rendering mode.
///
/// The first seven are the original PreK modes. The last three were added for
/// upper-grade (K-5) lessons so arithmetic content plays like the skill it
/// teaches instead of falling back to plain counting:
/// - `SkipCount`: each crossing advances an arithmetic sequence
///   (multiplication tables, division, place value, addition patterns).
/// - `Fractions`: each crossing fills one segment of a fraction bar.
/// - `Decimals`: each crossing counts up in tenths.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MathConcept {
    Counting,
    Shapes,
    Colors,
    SizeComp,
    QuantComp,
    Patterns,
    Sorting,
    SkipCount,
    Fractions,
    Decimals,
}

#[derive(Clone, Copy, Debug)]
pub struct VocabEntry {
    pub term: &'static str,
    /// Part of speech for the term ("noun", "verb", "adjective", …). Shown on
    /// the Reading Snake question card when words come from a lesson plan.
    pub part_of_speech: &'static str,
    pub definition: &'static str,
}

/// Extra data attached to math lessons. Counting lessons start their counter
/// at `start_count` so the on-screen labels walk through the right range
/// (PK-MATH-02 shows 6..10 by starting at 5).
#[derive(Clone, Copy, Debug)]
pub struct MathLessonData {
    pub concept: MathConcept,
    pub goal_hops: u32,
    pub start_count: u32,
    /// Sequence increment per crossing. `SkipCount` counts by this many units
    /// (3, 6, 9, … for step 3); `Decimals` counts by this many tenths
    /// (0.2, 0.4, … for step 2). 1 for every other concept.
    pub step: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct LessonPlan {
    pub id: &'static str,
    pub title: &'static str,
    // Metadata populated for every lesson; not read yet but kept so the data
    // model stays complete for grade/subject filtering.
    #[allow(dead_code)]
    pub grade: Grade,
    #[allow(dead_code)]
    pub subject: Subject,
    pub concept: &'static str,
    pub instruction: &'static str,
    pub success: &'static str,
    pub vocabulary: &'static [VocabEntry],
    /// `Some` for math lessons (Frog Lane uses this), `None` otherwise.
    pub math: Option<MathLessonData>,
}

// ─────────────────────────────────────────────────────────────────────────────
//  PK Mathematics — D:/lesson-plans/Pre-Kindergarten/Mathematics/PK-MATH-01..12
// ─────────────────────────────────────────────────────────────────────────────

pub const PK_MATH_LESSONS: &[LessonPlan] = &[
    LessonPlan {
        id: "PK-MATH-01",
        title: "Counting 1-5",
        grade: Grade::Preschool,
        subject: Subject::Mathematics,
        concept: "One-to-One Correspondence",
        instruction: "Count each crossing! Say the number out loud.",
        success: "You counted to 5! Great counting!",
        vocabulary: &[VocabEntry {
            term: "count",
            part_of_speech: "verb",
            definition: "To say the number names one by one while pointing to each object",
        }],
        math: Some(MathLessonData { concept: MathConcept::Counting, goal_hops: 5, start_count: 0, step: 1 }),
    },
    LessonPlan {
        id: "PK-MATH-02",
        title: "Counting 6-10",
        grade: Grade::Preschool,
        subject: Subject::Mathematics,
        concept: "Building on Number Sense",
        instruction: "Keep counting! Six, seven, eight, nine, ten!",
        success: "Amazing! You counted all the way to 10!",
        vocabulary: &[VocabEntry {
            term: "count",
            part_of_speech: "verb",
            definition: "To say the number names one by one while pointing to each object",
        }],
        math: Some(MathLessonData { concept: MathConcept::Counting, goal_hops: 5, start_count: 5, step: 1 }),
    },
    LessonPlan {
        id: "PK-MATH-03",
        title: "Counting 11-15",
        grade: Grade::Preschool,
        subject: Subject::Mathematics,
        concept: "Introduction to Teen Numbers",
        instruction: "Teen numbers! Eleven, twelve, thirteen, fourteen, fifteen!",
        success: "Wow, you counted to 15! Teen numbers done!",
        vocabulary: &[VocabEntry {
            term: "teen number",
            part_of_speech: "noun",
            definition: "A number between 10 and 20 that ends in -teen, like eleven (ten-one)",
        }],
        math: Some(MathLessonData { concept: MathConcept::Counting, goal_hops: 5, start_count: 10, step: 1 }),
    },
    LessonPlan {
        id: "PK-MATH-04",
        title: "Counting 16-20",
        grade: Grade::Preschool,
        subject: Subject::Mathematics,
        concept: "Completing the Range",
        instruction: "Almost to 20! Sixteen, seventeen, eighteen, nineteen, TWENTY!",
        success: "You counted all the way to 20! Incredible!",
        vocabulary: &[VocabEntry {
            term: "count",
            part_of_speech: "verb",
            definition: "To say the number names one by one while pointing to each object",
        }],
        math: Some(MathLessonData { concept: MathConcept::Counting, goal_hops: 5, start_count: 15, step: 1 }),
    },
    LessonPlan {
        id: "PK-MATH-05",
        title: "Shapes: Circle, Square, Triangle",
        grade: Grade::Preschool,
        subject: Subject::Mathematics,
        concept: "Shape Recognition",
        instruction: "Frog = CIRCLE   Cars = RECTANGLES   Beacon = TRIANGLE",
        success: "Great shape spotting! Circle, rectangle, triangle!",
        vocabulary: &[
            VocabEntry { term: "circle",   part_of_speech: "noun", definition: "A shape that is perfectly round with no corners or straight sides" },
            VocabEntry { term: "square",   part_of_speech: "noun", definition: "A shape with four equal sides and four right angles" },
            VocabEntry { term: "triangle", part_of_speech: "noun", definition: "A shape with three straight sides and three corners" },
        ],
        math: Some(MathLessonData { concept: MathConcept::Shapes, goal_hops: 5, start_count: 0, step: 1 }),
    },
    LessonPlan {
        id: "PK-MATH-06",
        title: "Shapes: Rectangle, Oval, Star",
        grade: Grade::Preschool,
        subject: Subject::Mathematics,
        concept: "More Shape Recognition",
        instruction: "Logs = OVALS   Cars = RECTANGLES   Beacon = STAR",
        success: "You found the star! Rectangle, oval, star!",
        vocabulary: &[
            VocabEntry { term: "rectangle", part_of_speech: "noun", definition: "A shape with four straight sides and four right angles, like a book" },
            VocabEntry { term: "oval",      part_of_speech: "noun", definition: "A round shape that is stretched out on one side, like an egg" },
            VocabEntry { term: "star",      part_of_speech: "noun", definition: "A five-pointed shape with sharp corners that reach outward" },
        ],
        math: Some(MathLessonData { concept: MathConcept::Shapes, goal_hops: 5, start_count: 0, step: 1 }),
    },
    LessonPlan {
        id: "PK-MATH-07",
        title: "Colors: Red, Blue, Green, Yellow",
        grade: Grade::Preschool,
        subject: Subject::Mathematics,
        concept: "Color Recognition",
        instruction: "RED road, BLUE & GREEN rivers, YELLOW goal!",
        success: "You crossed all the colors!",
        vocabulary: &[VocabEntry {
            term: "color",
            part_of_speech: "noun",
            definition: "The name we give to how things look — like red, blue, green, or yellow",
        }],
        math: Some(MathLessonData { concept: MathConcept::Colors, goal_hops: 5, start_count: 0, step: 1 }),
    },
    LessonPlan {
        id: "PK-MATH-08",
        title: "Colors: Orange, Purple, Brown",
        grade: Grade::Preschool,
        subject: Subject::Mathematics,
        concept: "More Color Recognition",
        instruction: "ORANGE road, PURPLE & BROWN rivers, WHITE goal!",
        success: "Amazing! Orange, Purple, Brown — you crossed them all!",
        vocabulary: &[VocabEntry {
            term: "color",
            part_of_speech: "noun",
            definition: "Names for how things look — like orange, purple, brown, black, white, or gray",
        }],
        math: Some(MathLessonData { concept: MathConcept::Colors, goal_hops: 5, start_count: 0, step: 1 }),
    },
    LessonPlan {
        id: "PK-MATH-09",
        title: "Big vs. Small",
        grade: Grade::Preschool,
        subject: Subject::Mathematics,
        concept: "Size Comparison",
        instruction: "ORANGE = BIG.   TEAL = small.",
        success: "You spotted big AND small!",
        vocabulary: &[
            VocabEntry { term: "big",   part_of_speech: "adjective", definition: "Something that takes up a lot of space, like a big bus" },
            VocabEntry { term: "small", part_of_speech: "adjective", definition: "Something that takes up little space, like a small toy car" },
        ],
        math: Some(MathLessonData { concept: MathConcept::SizeComp, goal_hops: 5, start_count: 0, step: 1 }),
    },
    LessonPlan {
        id: "PK-MATH-10",
        title: "More, Fewer, and Equal",
        grade: Grade::Preschool,
        subject: Subject::Mathematics,
        concept: "Basic Quantity Comparisons",
        instruction: "Which side has MORE? Which has FEWER? Count them!",
        success: "Great comparing! More, fewer, equal -- you did it!",
        vocabulary: &[
            VocabEntry { term: "more",  part_of_speech: "adjective", definition: "Having a greater quantity of something, like more apples" },
            VocabEntry { term: "fewer", part_of_speech: "adjective", definition: "Having a smaller quantity of something, like fewer cookies" },
            VocabEntry { term: "equal", part_of_speech: "adjective", definition: "The same amount or number, like two groups that have the same count" },
        ],
        math: Some(MathLessonData { concept: MathConcept::QuantComp, goal_hops: 5, start_count: 0, step: 1 }),
    },
    LessonPlan {
        id: "PK-MATH-11",
        title: "Simple Patterns: AB and AAB",
        grade: Grade::Preschool,
        subject: Subject::Mathematics,
        concept: "Pattern Recognition",
        instruction: "Road: RED-BLUE-RED-BLUE (AB).   River: RED-RED-BLUE (AAB).",
        success: "You spotted the patterns! Patterns repeat!",
        vocabulary: &[VocabEntry {
            term: "pattern",
            part_of_speech: "noun",
            definition: "A sequence that repeats over and over again, like a song that keeps going",
        }],
        math: Some(MathLessonData { concept: MathConcept::Patterns, goal_hops: 5, start_count: 0, step: 1 }),
    },
    LessonPlan {
        id: "PK-MATH-12",
        title: "Sorting by Attribute",
        grade: Grade::Preschool,
        subject: Subject::Mathematics,
        concept: "Sort and Classify",
        instruction: "BIG cars = RED.   Small cars = BLUE.   Two attributes!",
        success: "Perfect! Color AND size -- two attributes!",
        vocabulary: &[
            VocabEntry { term: "sort",      part_of_speech: "verb", definition: "To separate things into groups based on something they have in common" },
            VocabEntry { term: "attribute", part_of_speech: "noun", definition: "A characteristic — like color, shape, size, or texture that helps us sort" },
        ],
        math: Some(MathLessonData { concept: MathConcept::Sorting, goal_hops: 5, start_count: 0, step: 1 }),
    },
];

// ─────────────────────────────────────────────────────────────────────────────
//  PK Literacy — D:/lesson-plans/Pre-Kindergarten/Literacy/PK-LIT-01..15
//  Reading Snake pulls its spelling word bank from these lessons' vocab.
// ─────────────────────────────────────────────────────────────────────────────

pub const PK_LIT_LESSONS: &[LessonPlan] = &[
    LessonPlan {
        id: "PK-LIT-01",
        title: "Letter Recognition A-M",
        grade: Grade::Preschool,
        subject: Subject::Literacy,
        concept: "Print Awareness",
        instruction: "Spell the word from the lesson.",
        success: "Great spelling!",
        vocabulary: &[
            VocabEntry { term: "letter",    part_of_speech: "noun", definition: "A symbol that represents a sound in words" },
            VocabEntry { term: "uppercase", part_of_speech: "noun", definition: "Capital letters like A, B, C" },
        ],
        math: None,
    },
    LessonPlan {
        id: "PK-LIT-02",
        title: "Letter Recognition N-Z",
        grade: Grade::Preschool,
        subject: Subject::Literacy,
        concept: "Print Awareness",
        instruction: "Spell the word from the lesson.",
        success: "Great spelling!",
        vocabulary: &[
            VocabEntry { term: "letter",    part_of_speech: "noun", definition: "A symbol that represents a sound in words" },
            VocabEntry { term: "uppercase", part_of_speech: "noun", definition: "Capital letters like A, B, C" },
        ],
        math: None,
    },
    LessonPlan {
        id: "PK-LIT-03",
        title: "Beginning Sounds",
        grade: Grade::Preschool,
        subject: Subject::Literacy,
        concept: "Phonemic Awareness",
        instruction: "Spell the word from the lesson.",
        success: "Great spelling!",
        vocabulary: &[
            VocabEntry { term: "sound",     part_of_speech: "noun", definition: "What we hear when someone talks" },
            VocabEntry { term: "beginning", part_of_speech: "noun", definition: "The first sound you hear in a word, like /b/ in ball" },
        ],
        math: None,
    },
    LessonPlan {
        id: "PK-LIT-04",
        title: "Short Vowel Sounds (E)",
        grade: Grade::Preschool,
        subject: Subject::Literacy,
        concept: "Phonics",
        instruction: "Spell the word from the lesson.",
        success: "Great spelling!",
        vocabulary: &[VocabEntry {
            term: "vowel",
            part_of_speech: "noun",
            definition: "Letters a, e, i, o, u that make special sounds in words",
        }],
        math: None,
    },
    LessonPlan {
        id: "PK-LIT-05",
        title: "Sound Recognition (F, G)",
        grade: Grade::Preschool,
        subject: Subject::Literacy,
        concept: "Phonics",
        instruction: "Spell the word from the lesson.",
        success: "Great spelling!",
        vocabulary: &[VocabEntry {
            term: "sound",
            part_of_speech: "noun",
            definition: "What we hear when someone talks — like /f/ for f or /g/ for g",
        }],
        math: None,
    },
    LessonPlan {
        id: "PK-LIT-06",
        title: "Short Vowel Sounds",
        grade: Grade::Preschool,
        subject: Subject::Literacy,
        concept: "Phonics",
        instruction: "Spell the word from the lesson.",
        success: "Great spelling!",
        vocabulary: &[VocabEntry {
            term: "vowel",
            part_of_speech: "noun",
            definition: "A quick, brief letter sound — like /a/ in ant, /e/ in bed, /i/ in sit",
        }],
        math: None,
    },
    LessonPlan {
        id: "PK-LIT-07",
        title: "Predicting in Stories",
        grade: Grade::Preschool,
        subject: Subject::Literacy,
        concept: "Comprehension",
        instruction: "Spell the word from the lesson.",
        success: "Great spelling!",
        vocabulary: &[
            VocabEntry { term: "predict",   part_of_speech: "verb", definition: "To guess what might happen next based on clues" },
            VocabEntry { term: "character", part_of_speech: "noun", definition: "A person or animal that appears in a story" },
        ],
        math: None,
    },
    LessonPlan {
        id: "PK-LIT-08",
        title: "Rhyming Words",
        grade: Grade::Preschool,
        subject: Subject::Literacy,
        concept: "Phonemic Awareness",
        instruction: "Spell the word from the lesson.",
        success: "Great spelling!",
        vocabulary: &[VocabEntry {
            term: "rhyme",
            part_of_speech: "noun",
            definition: "Words that end with the same sound, like cat and hat",
        }],
        math: None,
    },
    LessonPlan {
        id: "PK-LIT-09",
        title: "Counting Syllables",
        grade: Grade::Preschool,
        subject: Subject::Literacy,
        concept: "Phonemic Awareness",
        instruction: "Spell the word from the lesson.",
        success: "Great spelling!",
        vocabulary: &[VocabEntry {
            term: "syllable",
            part_of_speech: "noun",
            definition: "A beat in words — banana has three beats, cat has one beat",
        }],
        math: None,
    },
    LessonPlan {
        id: "PK-LIT-10",
        title: "Writing My Name",
        grade: Grade::Preschool,
        subject: Subject::Literacy,
        concept: "Print Awareness",
        instruction: "Spell the word from the lesson.",
        success: "Great spelling!",
        vocabulary: &[VocabEntry {
            term: "name",
            part_of_speech: "noun",
            definition: "The special word that identifies who you are, like Emma or Noah",
        }],
        math: None,
    },
    LessonPlan {
        id: "PK-LIT-11",
        title: "Letter-Picture Matching",
        grade: Grade::Preschool,
        subject: Subject::Literacy,
        concept: "Phonemic Awareness",
        instruction: "Spell the word from the lesson.",
        success: "Great spelling!",
        vocabulary: &[VocabEntry {
            term: "match",
            part_of_speech: "verb",
            definition: "To find the same thing — like pairing letter A with an apple picture",
        }],
        math: None,
    },
    LessonPlan {
        id: "PK-LIT-12",
        title: "Story Sequencing",
        grade: Grade::Preschool,
        subject: Subject::Literacy,
        concept: "Comprehension",
        instruction: "Spell the word from the lesson.",
        success: "Great spelling!",
        vocabulary: &[
            VocabEntry { term: "first", part_of_speech: "adjective", definition: "The thing that happens before everything else" },
            VocabEntry { term: "next",  part_of_speech: "adjective", definition: "The thing that comes after first in the sequence" },
        ],
        math: None,
    },
    LessonPlan {
        id: "PK-LIT-13",
        title: "Following Directions",
        grade: Grade::Preschool,
        subject: Subject::Literacy,
        concept: "Listening",
        instruction: "Spell the word from the lesson.",
        success: "Great spelling!",
        vocabulary: &[VocabEntry {
            term: "direction",
            part_of_speech: "noun",
            definition: "A way to tell someone what to do — like stand up or clap your hands",
        }],
        math: None,
    },
    LessonPlan {
        id: "PK-LIT-14",
        title: "Singing the Alphabet",
        grade: Grade::Preschool,
        subject: Subject::Literacy,
        concept: "Print Awareness",
        instruction: "Spell the word from the lesson.",
        success: "Great spelling!",
        vocabulary: &[VocabEntry {
            term: "melody",
            part_of_speech: "noun",
            definition: "The tune or music of a song — like how the ABC song sounds",
        }],
        math: None,
    },
    LessonPlan {
        id: "PK-LIT-15",
        title: "Telling Stories",
        grade: Grade::Preschool,
        subject: Subject::Literacy,
        concept: "Oral Language",
        instruction: "Spell the word from the lesson.",
        success: "Great spelling!",
        vocabulary: &[VocabEntry {
            term: "story",
            part_of_speech: "noun",
            definition: "A make-believe tale with characters and a beginning, middle, and end",
        }],
        math: None,
    },
];

// ─────────────────────────────────────────────────────────────────────────────
//  Accessors
// ─────────────────────────────────────────────────────────────────────────────

/// Math lessons that apply to the given grade.
pub fn math_lessons_for_grade(grade: Grade) -> &'static [LessonPlan] {
    match grade {
        Grade::Preschool    => PK_MATH_LESSONS,
        Grade::Kindergarten => gen::K_MATH_LESSONS,
        Grade::FirstGrade   => gen::G1_MATH_LESSONS,
        Grade::SecondGrade  => gen::G2_MATH_LESSONS,
        Grade::ThirdGrade   => gen::G3_MATH_LESSONS,
        Grade::FourthGrade  => gen::G4_MATH_LESSONS,
        Grade::FifthGrade   => gen::G5_MATH_LESSONS,
    }
}

/// Literacy lessons that apply to the given grade.
pub fn literacy_lessons_for_grade(grade: Grade) -> &'static [LessonPlan] {
    match grade {
        Grade::Preschool    => PK_LIT_LESSONS,
        Grade::Kindergarten => gen::K_LIT_LESSONS,
        Grade::FirstGrade   => gen::G1_LIT_LESSONS,
        Grade::SecondGrade  => gen::G2_LIT_LESSONS,
        Grade::ThirdGrade   => gen::G3_LIT_LESSONS,
        Grade::FourthGrade  => gen::G4_LIT_LESSONS,
        Grade::FifthGrade   => gen::G5_LIT_LESSONS,
    }
}

/// All vocabulary terms across the literacy lessons for the grade, flattened
/// into a unique list. Returns owned `(term, part_of_speech, definition)`
/// tuples so the caller doesn't pull `LessonPlan` into its API.
pub fn literacy_vocab_for_grade(grade: Grade) -> Vec<(String, String, String)> {
    let mut out: Vec<(String, String, String)> = Vec::new();
    for lesson in literacy_lessons_for_grade(grade) {
        for v in lesson.vocabulary {
            let upper = v.term.to_ascii_uppercase();
            let upper = sanitize_word(&upper);
            if upper.is_empty() {
                continue;
            }
            if out.iter().any(|(w, _, _)| *w == upper) {
                continue;
            }
            out.push((upper, v.part_of_speech.to_string(), v.definition.to_string()));
        }
    }
    out
}

/// Strip non-letter characters and collapse whitespace, since Reading Snake
/// stores a single word per entry.
fn sanitize_word(s: &str) -> String {
    s.chars().filter(|c| c.is_ascii_alphabetic()).collect()
}

// ─────────────────────────────────────────────────────────────────────────────
//  Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pk_math_set_is_complete() {
        assert_eq!(PK_MATH_LESSONS.len(), 12);
        for (i, lesson) in PK_MATH_LESSONS.iter().enumerate() {
            assert!(lesson.id.starts_with("PK-MATH-"));
            assert!(lesson.math.is_some(), "math lesson {} missing math data", lesson.id);
            let expected = format!("PK-MATH-{:02}", i + 1);
            assert_eq!(lesson.id, expected);
        }
    }

    #[test]
    fn pk_lit_set_is_complete() {
        assert_eq!(PK_LIT_LESSONS.len(), 15);
        for (i, lesson) in PK_LIT_LESSONS.iter().enumerate() {
            assert!(lesson.id.starts_with("PK-LIT-"));
            assert!(lesson.math.is_none(), "literacy lesson {} should not have math data", lesson.id);
            let expected = format!("PK-LIT-{:02}", i + 1);
            assert_eq!(lesson.id, expected);
        }
    }

    #[test]
    fn counting_lessons_walk_the_full_range() {
        let counting: Vec<&LessonPlan> = PK_MATH_LESSONS
            .iter()
            .filter(|l| l.math.map(|m| m.concept == MathConcept::Counting).unwrap_or(false))
            .collect();
        assert_eq!(counting.len(), 4);
        let mut max_seen = 0u32;
        for lesson in counting {
            let m = lesson.math.unwrap();
            max_seen = max_seen.max(m.start_count + m.goal_hops);
        }
        assert_eq!(max_seen, 20, "counting lessons should walk 1..20 collectively");
    }

    #[test]
    fn literacy_vocab_is_grade_keyed_and_non_empty() {
        let vocab = literacy_vocab_for_grade(Grade::Preschool);
        assert!(!vocab.is_empty(), "PK literacy vocab should not be empty");
        // Every entry should be uppercase ascii letters.
        for (word, _, _) in &vocab {
            assert!(word.chars().all(|c| c.is_ascii_uppercase()), "non-uppercase: {}", word);
            assert!(!word.is_empty());
        }
        // Spot-check some terms we transcribed from PK-LIT-01..15
        assert!(vocab.iter().any(|(w, _, _)| w == "LETTER"));
        assert!(vocab.iter().any(|(w, _, _)| w == "RHYME"));
        assert!(vocab.iter().any(|(w, _, _)| w == "STORY"));
    }

    #[test]
    fn literacy_vocab_deduplicates_across_lessons() {
        // PK-LIT-01 and PK-LIT-02 both define "letter" — should appear once.
        let vocab = literacy_vocab_for_grade(Grade::Preschool);
        let letter_count = vocab.iter().filter(|(w, _, _)| w == "LETTER").count();
        assert_eq!(letter_count, 1);
    }

    const ALL_GRADES: [Grade; 7] = [
        Grade::Preschool,
        Grade::Kindergarten,
        Grade::FirstGrade,
        Grade::SecondGrade,
        Grade::ThirdGrade,
        Grade::FourthGrade,
        Grade::FifthGrade,
    ];

    #[test]
    fn every_grade_has_math_and_literacy_lessons() {
        for grade in ALL_GRADES {
            let math = math_lessons_for_grade(grade);
            assert!(math.len() >= 8, "{grade:?} has only {} math lessons", math.len());
            assert!(math.iter().all(|l| l.math.is_some()), "{grade:?} math lesson missing math data");
            assert!(math.iter().all(|l| l.grade == grade), "{grade:?} math bank holds foreign lessons");

            let lit = literacy_lessons_for_grade(grade);
            assert!(lit.len() >= 8, "{grade:?} has only {} literacy lessons", lit.len());
            assert!(lit.iter().all(|l| l.grade == grade), "{grade:?} literacy bank holds foreign lessons");
        }
    }

    #[test]
    fn every_grade_has_playable_vocab() {
        for grade in ALL_GRADES {
            let vocab = literacy_vocab_for_grade(grade);
            assert!(vocab.len() >= 10, "{grade:?} has only {} vocab words", vocab.len());
            for (word, pos, def) in &vocab {
                // Reading Snake constraints: letters only, <= 12 chars.
                assert!(word.chars().all(|c| c.is_ascii_uppercase()), "{grade:?}: bad word {word}");
                assert!(word.len() <= 12, "{grade:?}: word too long {word}");
                assert!(!pos.is_empty(), "{grade:?}: {word} missing part of speech");
                assert!(!def.is_empty(), "{grade:?}: {word} missing definition");
            }
        }
    }

    #[test]
    fn grades_receive_distinct_content() {
        // The whole point of per-grade banks: a 5th grader must not get
        // preschool material.
        let pk: Vec<String> = literacy_vocab_for_grade(Grade::Preschool).into_iter().map(|(w, _, _)| w).collect();
        let g5: Vec<String> = literacy_vocab_for_grade(Grade::FifthGrade).into_iter().map(|(w, _, _)| w).collect();
        assert_ne!(pk, g5, "5th grade still serves preschool vocab");

        let pk_ids: Vec<&str> = math_lessons_for_grade(Grade::Preschool).iter().map(|l| l.id).collect();
        let k_ids: Vec<&str> = math_lessons_for_grade(Grade::Kindergarten).iter().map(|l| l.id).collect();
        assert_ne!(pk_ids, k_ids, "kindergarten still serves PK math lessons");
        assert!(k_ids.iter().all(|id| id.starts_with("K-MATH-")));
    }

    #[test]
    fn every_math_lesson_has_valid_sequence_data() {
        for grade in ALL_GRADES {
            for lesson in math_lessons_for_grade(grade) {
                let m = lesson.math.expect("math lesson missing math data");
                assert!(m.goal_hops > 0, "{}: zero goal_hops", lesson.id);
                assert!(m.step >= 1, "{}: step must be at least 1", lesson.id);
                if m.concept == MathConcept::Decimals {
                    assert!(m.step < 10, "{}: decimal step is whole tenths", lesson.id);
                }
            }
        }
    }

    #[test]
    fn upper_grades_use_upper_grade_concepts() {
        // 2nd-5th arithmetic must not all collapse into plain Counting:
        // the corpus has multiplication, fractions, and decimals lessons,
        // and the generator must map them onto the dedicated modes.
        let upper = [
            Grade::SecondGrade,
            Grade::ThirdGrade,
            Grade::FourthGrade,
            Grade::FifthGrade,
        ];
        let concepts: Vec<MathConcept> = upper
            .iter()
            .flat_map(|&g| math_lessons_for_grade(g).iter().filter_map(|l| l.math.map(|m| m.concept)))
            .collect();
        assert!(
            concepts.contains(&MathConcept::SkipCount),
            "no upper-grade SkipCount lessons"
        );
        assert!(
            concepts.contains(&MathConcept::Fractions),
            "no upper-grade Fractions lessons"
        );
        assert!(
            concepts.contains(&MathConcept::Decimals),
            "no upper-grade Decimals lessons"
        );
        let counting = concepts.iter().filter(|&&c| c == MathConcept::Counting).count();
        assert!(
            counting * 2 < concepts.len(),
            "most upper-grade math lessons still fall back to Counting ({counting}/{})",
            concepts.len()
        );
    }
}
