use crate::levels::Grade;
use serde::Serialize;
use std::cell::RefCell;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifeLossReason {
    WrongTarget,
    EnemyHit,
}

impl LifeLossReason {
    fn as_str(self) -> &'static str {
        match self {
            LifeLossReason::WrongTarget => "wrong_target",
            LifeLossReason::EnemyHit => "enemy_hit",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameOverReason {
    LivesExhausted,
    EnemiesReachedBottom,
}

impl GameOverReason {
    fn as_str(self) -> &'static str {
        match self {
            GameOverReason::LivesExhausted => "lives_exhausted",
            GameOverReason::EnemiesReachedBottom => "enemies_reached_bottom",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameEvent {
    CorrectMathInvadersHit {
        grade: Grade,
        score: u32,
        points_awarded: u32,
    },
    LifeLost {
        grade: Grade,
        lives_remaining: u8,
        reason: LifeLossReason,
    },
    WaveCleared {
        grade: Grade,
        wave: usize,
        score: u32,
        points_awarded: u32,
    },
    GateAnswer {
        grade: Grade,
        correct: bool,
        score: u32,
        gates_remaining: usize,
    },
    GradeAdvanced {
        from_grade: Grade,
        to_grade: Grade,
        wave: usize,
        score: u32,
    },
    FinalVictory {
        score: u32,
    },
    ReadingSnakeComplete {
        adventure_active: bool,
        nightmare_mode: bool,
    },
    MathPongComplete {
        adventure_active: bool,
    },
    GameOver {
        grade: Grade,
        wave: usize,
        score: u32,
        reason: GameOverReason,
    },
}

/// JSON-ready envelope for the platform-hosted shell to POST or queue.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PlatformEventPayload {
    pub platform_kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<PlatformScorePayload>,
    pub client_context: serde_json::Value,
    pub leaderboard_eligible: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PlatformScorePayload {
    pub points: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub points_awarded: Option<u32>,
}

pub fn platform_payload(event: &GameEvent) -> PlatformEventPayload {
    match event {
        GameEvent::CorrectMathInvadersHit {
            grade,
            score,
            points_awarded,
        } => PlatformEventPayload {
            platform_kind: "game_challenge_completed",
            score: Some(PlatformScorePayload {
                points: *score,
                points_awarded: Some(*points_awarded),
            }),
            client_context: serde_json::json!({
                "starcrusher_event": "correct_math_invaders_hit",
                "grade": grade.display_name(),
                "grade_index": grade.index(),
            }),
            leaderboard_eligible: false,
        },
        GameEvent::LifeLost {
            grade,
            lives_remaining,
            reason,
        } => PlatformEventPayload {
            platform_kind: "game_challenge_attempted",
            score: None,
            client_context: serde_json::json!({
                "starcrusher_event": "life_lost",
                "reason": reason.as_str(),
                "grade": grade.display_name(),
                "grade_index": grade.index(),
                "lives_remaining": lives_remaining,
            }),
            leaderboard_eligible: false,
        },
        GameEvent::WaveCleared {
            grade,
            wave,
            score,
            points_awarded,
        } => PlatformEventPayload {
            platform_kind: "game_level_completed",
            score: Some(PlatformScorePayload {
                points: *score,
                points_awarded: Some(*points_awarded),
            }),
            client_context: serde_json::json!({
                "starcrusher_event": "wave_cleared",
                "grade": grade.display_name(),
                "grade_index": grade.index(),
                "wave": wave,
            }),
            leaderboard_eligible: false,
        },
        GameEvent::GateAnswer {
            grade,
            correct,
            score: _,
            gates_remaining,
        } => PlatformEventPayload {
            platform_kind: if *correct {
                "learning_assessment_completed"
            } else {
                "game_challenge_attempted"
            },
            score: None,
            client_context: serde_json::json!({
                "starcrusher_event": "gate_answer",
                "correct": correct,
                "grade": grade.display_name(),
                "grade_index": grade.index(),
                "gates_remaining": gates_remaining,
            }),
            leaderboard_eligible: false,
        },
        GameEvent::GradeAdvanced {
            from_grade,
            to_grade,
            wave,
            score: _,
        } => PlatformEventPayload {
            platform_kind: "learning_topic_practiced",
            score: None,
            client_context: serde_json::json!({
                "starcrusher_event": "grade_advanced",
                "from_grade": from_grade.display_name(),
                "to_grade": to_grade.display_name(),
                "wave": wave,
            }),
            leaderboard_eligible: false,
        },
        GameEvent::FinalVictory { score } => PlatformEventPayload {
            platform_kind: "game_level_completed",
            score: Some(PlatformScorePayload {
                points: *score,
                points_awarded: None,
            }),
            client_context: serde_json::json!({
                "starcrusher_event": "final_victory",
            }),
            leaderboard_eligible: true,
        },
        GameEvent::ReadingSnakeComplete {
            adventure_active,
            nightmare_mode,
        } => PlatformEventPayload {
            platform_kind: "game_challenge_completed",
            score: None,
            client_context: serde_json::json!({
                "starcrusher_event": "reading_snake_complete",
                "adventure_active": adventure_active,
                "nightmare_mode": nightmare_mode,
            }),
            leaderboard_eligible: false,
        },
        GameEvent::MathPongComplete { adventure_active } => PlatformEventPayload {
            platform_kind: "game_challenge_completed",
            score: None,
            client_context: serde_json::json!({
                "starcrusher_event": "math_pong_complete",
                "adventure_active": adventure_active,
            }),
            leaderboard_eligible: false,
        },
        GameEvent::GameOver {
            grade,
            wave,
            score,
            reason,
        } => PlatformEventPayload {
            platform_kind: "game_level_completed",
            score: Some(PlatformScorePayload {
                points: *score,
                points_awarded: None,
            }),
            client_context: serde_json::json!({
                "starcrusher_event": "game_over",
                "reason": reason.as_str(),
                "grade": grade.display_name(),
                "grade_index": grade.index(),
                "wave": wave,
            }),
            leaderboard_eligible: true,
        },
    }
}

pub fn serialize_platform_event(event: &GameEvent) -> Result<String, serde_json::Error> {
    serde_json::to_string(&platform_payload(event))
}

pub trait PlatformBridge {
    fn emit(&self, event: &GameEvent);
}

#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
pub struct NoOpPlatformBridge;

impl PlatformBridge for NoOpPlatformBridge {
    fn emit(&self, _event: &GameEvent) {}
}

/// Forwards serialized platform events to the browser shell via JS (wasm32 only).
#[derive(Debug, Clone, Copy, Default)]
pub struct JsForwardingPlatformBridge;

impl PlatformBridge for JsForwardingPlatformBridge {
    fn emit(&self, event: &GameEvent) {
        let Ok(json) = serialize_platform_event(event) else {
            return;
        };
        forward_event_json(&json);
    }
}

fn forward_event_json(json: &str) {
    #[cfg(target_arch = "wasm32")]
    wasm_js::emit_platform_event(json);

    #[cfg(not(target_arch = "wasm32"))]
    let _ = json;
}

#[cfg(target_arch = "wasm32")]
mod wasm_js {
    extern "C" {
        fn boohw_starcrusher_emit_event(ptr: *const u8, len: u32);
        fn boohw_starcrusher_initial_mode() -> u32;
        fn boohw_starcrusher_return_to_landing();
    }

    pub fn emit_platform_event(json: &str) {
        unsafe {
            boohw_starcrusher_emit_event(json.as_ptr(), json.len() as u32);
        }
    }

    pub fn initial_mode_code() -> u32 {
        unsafe { boohw_starcrusher_initial_mode() }
    }

    pub fn return_to_landing() {
        unsafe { boohw_starcrusher_return_to_landing() }
    }
}

/// Returns the shell-hinted starting mode (0 = title, 1 = adventure, 2 = mission).
pub fn initial_mode_code() -> u32 {
    #[cfg(target_arch = "wasm32")]
    {
        wasm_js::initial_mode_code()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        0
    }
}

/// Asks the shell to reload back to the HTML landing page.
pub fn return_to_landing() {
    #[cfg(target_arch = "wasm32")]
    {
        wasm_js::return_to_landing();
    }
}

/// Native/offline builds keep events local; wasm32 forwards to the hosted shell.
#[cfg(target_arch = "wasm32")]
pub type ActivePlatformBridge = JsForwardingPlatformBridge;

#[cfg(not(target_arch = "wasm32"))]
pub type ActivePlatformBridge = NoOpPlatformBridge;

/// Collects serialized platform payloads for tests and future debug tooling.
#[derive(Debug, Default)]
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
pub struct RecordingPlatformBridge {
    payloads: RefCell<Vec<PlatformEventPayload>>,
}

#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
impl RecordingPlatformBridge {
    pub fn take_payloads(&self) -> Vec<PlatformEventPayload> {
        self.payloads.borrow_mut().drain(..).collect()
    }

    pub fn json_lines(&self) -> Result<Vec<String>, serde_json::Error> {
        self.payloads
            .borrow()
            .iter()
            .map(serde_json::to_string)
            .collect()
    }
}

impl PlatformBridge for RecordingPlatformBridge {
    fn emit(&self, event: &GameEvent) {
        self.payloads.borrow_mut().push(platform_payload(event));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn preschool() -> Grade {
        Grade::Preschool
    }

    #[test]
    fn correct_hit_serializes_to_game_challenge_completed() {
        let event = GameEvent::CorrectMathInvadersHit {
            grade: preschool(),
            score: 25,
            points_awarded: 10,
        };
        let payload = platform_payload(&event);

        assert_eq!(payload.platform_kind, "game_challenge_completed");
        assert_eq!(
            payload.score,
            Some(PlatformScorePayload {
                points: 25,
                points_awarded: Some(10),
            })
        );
        assert_eq!(
            payload.client_context["starcrusher_event"],
            "correct_math_invaders_hit"
        );
        assert!(!payload.leaderboard_eligible);
    }

    #[test]
    fn gate_answer_maps_kind_by_correctness() {
        let correct = platform_payload(&GameEvent::GateAnswer {
            grade: preschool(),
            correct: true,
            score: 100,
            gates_remaining: 0,
        });
        let incorrect = platform_payload(&GameEvent::GateAnswer {
            grade: preschool(),
            correct: false,
            score: 100,
            gates_remaining: 1,
        });

        assert_eq!(correct.platform_kind, "learning_assessment_completed");
        assert_eq!(incorrect.platform_kind, "game_challenge_attempted");
        assert_eq!(correct.client_context["correct"], true);
        assert_eq!(incorrect.client_context["correct"], false);
        assert!(correct.score.is_none());
    }

    #[test]
    fn terminal_events_are_leaderboard_eligible() {
        let victory = platform_payload(&GameEvent::FinalVictory { score: 500 });
        let game_over = platform_payload(&GameEvent::GameOver {
            grade: preschool(),
            wave: 2,
            score: 120,
            reason: GameOverReason::LivesExhausted,
        });

        assert!(victory.leaderboard_eligible);
        assert!(game_over.leaderboard_eligible);
    }

    #[test]
    fn non_terminal_events_are_not_leaderboard_eligible() {
        let wave = platform_payload(&GameEvent::WaveCleared {
            grade: preschool(),
            wave: 1,
            score: 200,
            points_awarded: 100,
        });

        assert!(!wave.leaderboard_eligible);
    }

    #[test]
    fn recording_bridge_collects_payloads() {
        let bridge = RecordingPlatformBridge::default();
        bridge.emit(&GameEvent::MathPongComplete {
            adventure_active: true,
        });
        bridge.emit(&GameEvent::FinalVictory { score: 999 });

        let payloads = bridge.take_payloads();
        assert_eq!(payloads.len(), 2);
        assert_eq!(payloads[0].platform_kind, "game_challenge_completed");
        assert_eq!(payloads[1].platform_kind, "game_level_completed");
        assert!(payloads[1].leaderboard_eligible);

        bridge.emit(&GameEvent::FinalVictory { score: 1 });
        let lines = bridge.json_lines().expect("json lines");
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("\"leaderboard_eligible\":true"));
    }

    #[test]
    fn js_forwarding_bridge_is_offline_safe_on_native() {
        let bridge = JsForwardingPlatformBridge;
        bridge.emit(&GameEvent::FinalVictory { score: 42 });
    }

    #[test]
    fn serialize_platform_event_emits_compact_json() {
        let json = serialize_platform_event(&GameEvent::LifeLost {
            grade: preschool(),
            lives_remaining: 3,
            reason: LifeLossReason::WrongTarget,
        })
        .expect("serialize life lost");

        assert!(json.contains("\"platform_kind\":\"game_challenge_attempted\""));
        assert!(json.contains("\"starcrusher_event\":\"life_lost\""));
        assert!(json.contains("\"reason\":\"wrong_target\""));
    }
}
