// Star Academy shared color palette — GW blue & gold.
//
// Deep navy / royal-blue backgrounds with gold accents, applied consistently
// across all Star Academy games (Meteor Catch, Number Rain, Plasma Breaker)
// and the hub.  Each game re-exports these into its local `C_*` constants so
// the per-game draw code stays unchanged while the palette is centralized.
//
// Correct/wrong feedback colors are intentionally kept green/crimson: they are
// gameplay signals, not theme decoration, and must stay clearly distinct.

use macroquad::prelude::Color;

/// Deep navy — primary background (top of gradient).
pub const BG_TOP: Color = Color { r: 0.01, g: 0.05, b: 0.18, a: 1.0 };
/// Deeper royal navy — bottom of gradient.
pub const BG_BOTTOM: Color = Color { r: 0.02, g: 0.08, b: 0.28, a: 1.0 };
/// Panel / header fill (translucent royal navy).
pub const PANEL: Color = Color { r: 0.04, g: 0.10, b: 0.30, a: 0.94 };
/// Card fill (hub game cards).
pub const CARD: Color = Color { r: 0.05, g: 0.11, b: 0.30, a: 1.0 };
/// Royal blue — primary interactive element (meteors, drops, balls).
pub const ROYAL: Color = Color { r: 0.13, g: 0.31, b: 0.72, a: 1.0 };
/// Royal blue glow.
pub const ROYAL_GLOW: Color = Color { r: 0.25, g: 0.45, b: 0.95, a: 0.32 };
/// GW gold — accents, paddles, highlights.
pub const GOLD: Color = Color { r: 1.0, g: 0.78, b: 0.16, a: 1.0 };
/// Gold glow.
pub const GOLD_GLOW: Color = Color { r: 1.0, g: 0.82, b: 0.32, a: 0.35 };
/// Question prompt text — bright gold.
pub const QUESTION: Color = Color { r: 1.0, g: 0.82, b: 0.28, a: 1.0 };
/// Secondary label text — muted blue-gray.
pub const LABEL: Color = Color { r: 0.70, g: 0.76, b: 0.92, a: 1.0 };
/// Home / back button fill.
pub const BUTTON: Color = Color { r: 0.09, g: 0.18, b: 0.44, a: 0.96 };
/// Correct feedback — green (gameplay signal, kept distinct).
pub const CORRECT: Color = Color { r: 0.30, g: 0.80, b: 0.45, a: 1.0 };
/// Wrong feedback — crimson (gameplay signal, kept distinct).
pub const WRONG: Color = Color { r: 0.90, g: 0.26, b: 0.28, a: 1.0 };
/// Star pip — gold (earned).
pub const STAR_ON: Color = Color { r: 1.0, g: 0.80, b: 0.16, a: 1.0 };
/// Star pip — unearned.
pub const STAR_OFF: Color = Color { r: 0.18, g: 0.22, b: 0.40, a: 1.0 };
