// Star Academy — player progress and persistence.
//
// `PlayerProgress` is the single source of truth for grade selection,
// star accumulation, and per-game best ratings. It round-trips cleanly
// through JSON so it can be stored in browser localStorage via the
// platform bridge (save/load_progress in platform.rs).
//
// Design constraints
// ------------------
// • Grade is remembered between sessions (kids share devices).
// • Stars accumulate passively during play — no quiz gates.
// • Filling STARS_TO_ADVANCE triggers a "Grade Up" ceremony that is
//   offered, never forced.
// • `bests[grade_index][game_index]` records the best 0-3 star rating
//   ever earned at that grade for that game (shown on game cards).

use crate::levels::Grade;
use crate::platform;
use serde::{Deserialize, Serialize};

/// Number of grade levels (Preschool → FifthGrade).
pub const GRADE_COUNT: usize = 7;

/// Number of playable games in Star Academy v1.
pub const GAME_COUNT: usize = 3;

/// Stars needed to fill a grade's meter and trigger the ceremony.
pub const STARS_TO_ADVANCE: u8 = 10;

/// localStorage key used by the platform bridge.
pub const STORAGE_KEY: &str = "star-academy-progress";

/// Buffer size for the load_progress read (must fit the serialised JSON).
const PROGRESS_BUF_BYTES: usize = 2048;

// ---------------------------------------------------------------------------
// AcademyGame — the three v1 games
// ---------------------------------------------------------------------------

/// Identifies each playable game. The discriminant doubles as the
/// `bests[grade][game_index]` column index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcademyGame {
    MeteorCatch   = 0,
    NumberRain    = 1,
    PlasmaBreaker = 2,
}

impl AcademyGame {
    pub fn index(self) -> usize {
        self as usize
    }

    pub fn all() -> [AcademyGame; GAME_COUNT] {
        [Self::MeteorCatch, Self::NumberRain, Self::PlasmaBreaker]
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::MeteorCatch   => "Meteor Catch",
            Self::NumberRain    => "Number Rain",
            Self::PlasmaBreaker => "Plasma Breaker",
        }
    }

    pub fn tagline(self) -> &'static str {
        match self {
            Self::MeteorCatch   => "Shield the right answer",
            Self::NumberRain    => "Tap before it lands",
            Self::PlasmaBreaker => "Break the right blocks",
        }
    }
}

// ---------------------------------------------------------------------------
// PlayerProgress — the data model
// ---------------------------------------------------------------------------

/// All persistent state for one device. Serialises to ~200-byte JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerProgress {
    /// Active grade stored as an index (0 = Preschool … 6 = FifthGrade).
    /// Kept as `u8` so serde round-trips cleanly without requiring Grade
    /// to implement Serialize/Deserialize.
    pub grade_index: u8,

    /// Stars accumulated toward the advance threshold, one counter per
    /// grade. Clamped to STARS_TO_ADVANCE on write.
    pub stars: [u8; GRADE_COUNT],

    /// Best star rating (0-3) per game per grade.
    /// Indexed as `bests[grade_index][AcademyGame::index()]`.
    pub bests: [[u8; GAME_COUNT]; GRADE_COUNT],

    /// Schema version — allows safe forward migration if fields change.
    #[serde(default = "schema_v1")]
    pub version: u32,
}

fn schema_v1() -> u32 {
    1
}

impl Default for PlayerProgress {
    fn default() -> Self {
        Self {
            grade_index: 0,
            stars: [0; GRADE_COUNT],
            bests: [[0; GAME_COUNT]; GRADE_COUNT],
            version: 1,
        }
    }
}

impl PlayerProgress {
    // ------------------------------------------------------------------
    // Grade helpers
    // ------------------------------------------------------------------

    /// Active grade as the enum used everywhere else in the codebase.
    pub fn grade(&self) -> Grade {
        Grade::from_index(self.grade_index as usize)
    }

    /// Switch the active grade and persist immediately.
    pub fn set_grade(&mut self, grade: Grade) {
        self.grade_index = grade.index() as u8;
        self.save();
    }

    // ------------------------------------------------------------------
    // Stars
    // ------------------------------------------------------------------

    /// Stars accumulated at the current grade.
    pub fn current_stars(&self) -> u8 {
        self.stars[self.grade_index as usize]
    }

    /// Add `count` stars to the current grade's meter.
    /// Returns `true` if the meter is now full (≥ STARS_TO_ADVANCE),
    /// signalling that the "Grade Up" ceremony should be offered.
    pub fn add_stars(&mut self, count: u8) -> bool {
        let gi = self.grade_index as usize;
        self.stars[gi] = (self.stars[gi].saturating_add(count)).min(STARS_TO_ADVANCE);
        self.stars[gi] >= STARS_TO_ADVANCE
    }

    /// Drain the current grade's star meter after the ceremony completes.
    pub fn clear_stars_for_current_grade(&mut self) {
        self.stars[self.grade_index as usize] = 0;
    }

    // ------------------------------------------------------------------
    // Best ratings
    // ------------------------------------------------------------------

    /// Record a game session result, keeping only the all-time best.
    /// `stars` is clamped to 0-3.
    pub fn record_best(&mut self, game: AcademyGame, stars: u8) {
        let gi = self.grade_index as usize;
        let col = game.index();
        let clamped = stars.min(3);
        if clamped > self.bests[gi][col] {
            self.bests[gi][col] = clamped;
        }
    }

    /// Best star rating for `game` at the current grade (0-3, 0 = never played).
    pub fn best_for(&self, game: AcademyGame) -> u8 {
        self.bests[self.grade_index as usize][game.index()]
    }

    // ------------------------------------------------------------------
    // Persistence
    // ------------------------------------------------------------------

    /// Load from localStorage. Returns `Default` if nothing is stored or
    /// the stored JSON fails to parse (e.g. after a schema change).
    pub fn load() -> Self {
        platform::load_progress(PROGRESS_BUF_BYTES)
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_default()
    }

    /// Serialise and write to localStorage. Silently drops errors
    /// (private browsing, storage quota exceeded) to avoid crashing
    /// the game loop.
    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string(self) {
            platform::save_progress(STORAGE_KEY, &json);
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_progress_starts_at_preschool_with_no_stars() {
        let p = PlayerProgress::default();
        assert_eq!(p.grade(), Grade::Preschool);
        assert_eq!(p.current_stars(), 0);
        for g in 0..GRADE_COUNT {
            for gm in 0..GAME_COUNT {
                assert_eq!(p.bests[g][gm], 0);
            }
        }
    }

    #[test]
    fn add_stars_returns_true_when_meter_fills() {
        let mut p = PlayerProgress::default();
        assert!(!p.add_stars(9));
        assert!(p.add_stars(1));
        assert_eq!(p.current_stars(), STARS_TO_ADVANCE);
    }

    #[test]
    fn add_stars_clamps_at_threshold() {
        let mut p = PlayerProgress::default();
        p.add_stars(255);
        assert_eq!(p.current_stars(), STARS_TO_ADVANCE);
    }

    #[test]
    fn record_best_keeps_highest_rating() {
        let mut p = PlayerProgress::default();
        p.record_best(AcademyGame::MeteorCatch, 2);
        p.record_best(AcademyGame::MeteorCatch, 1);
        assert_eq!(p.best_for(AcademyGame::MeteorCatch), 2);
        p.record_best(AcademyGame::MeteorCatch, 3);
        assert_eq!(p.best_for(AcademyGame::MeteorCatch), 3);
    }

    #[test]
    fn record_best_clamps_to_three_stars() {
        let mut p = PlayerProgress::default();
        p.record_best(AcademyGame::NumberRain, 99);
        assert_eq!(p.best_for(AcademyGame::NumberRain), 3);
    }

    #[test]
    fn best_for_is_per_grade() {
        let mut p = PlayerProgress::default();
        p.record_best(AcademyGame::PlasmaBreaker, 3);
        // Advance to next grade — best should start at 0
        p.grade_index = Grade::Kindergarten.index() as u8;
        assert_eq!(p.best_for(AcademyGame::PlasmaBreaker), 0);
    }

    #[test]
    fn grade_round_trips_through_index() {
        for grade in [
            Grade::Preschool,
            Grade::Kindergarten,
            Grade::FirstGrade,
            Grade::SecondGrade,
            Grade::ThirdGrade,
            Grade::FourthGrade,
            Grade::FifthGrade,
        ] {
            let mut p = PlayerProgress::default();
            p.grade_index = grade.index() as u8;
            assert_eq!(p.grade(), grade);
        }
    }

    #[test]
    fn progress_serialises_and_deserialises() {
        let mut original = PlayerProgress::default();
        original.grade_index = 2;
        original.add_stars(5);
        original.record_best(AcademyGame::MeteorCatch, 3);
        original.record_best(AcademyGame::NumberRain, 1);

        let json = serde_json::to_string(&original).expect("serialise");
        let loaded: PlayerProgress = serde_json::from_str(&json).expect("deserialise");

        assert_eq!(loaded.grade_index, 2);
        assert_eq!(loaded.current_stars(), 5);
        assert_eq!(loaded.best_for(AcademyGame::MeteorCatch), 3);
        assert_eq!(loaded.best_for(AcademyGame::NumberRain), 1);
        assert_eq!(loaded.version, 1);
    }

    #[test]
    fn missing_version_field_defaults_to_one() {
        // Simulate JSON from a hypothetical pre-version schema.
        let json = r#"{"grade_index":0,"stars":[0,0,0,0,0,0,0],"bests":[[0,0,0],[0,0,0],[0,0,0],[0,0,0],[0,0,0],[0,0,0],[0,0,0]]}"#;
        let loaded: PlayerProgress = serde_json::from_str(json).expect("deserialise");
        assert_eq!(loaded.version, 1);
    }

    #[test]
    fn clear_stars_resets_only_current_grade() {
        let mut p = PlayerProgress::default();
        p.add_stars(10); // fill preschool
        p.grade_index = 1; // switch to kindergarten
        p.add_stars(5);
        p.grade_index = 0; // back to preschool
        p.clear_stars_for_current_grade();
        assert_eq!(p.stars[0], 0);
        assert_eq!(p.stars[1], 5); // kindergarten untouched
    }
}
