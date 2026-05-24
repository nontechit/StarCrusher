use crate::assets;
use crate::screen;
use macroquad::prelude::*;

const PLAYER_Y: f32 = 680.0;
const MOBILE_PLAYER_Y: f32 = 1086.0;
const MOBILE_PLAYER_SCALE_MULT: f32 = 3.0;

/// Player ship position and state.
pub struct Player {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Player {
    /// Creates a new player centered at the bottom of the screen.
    pub fn new(screen_w: f32) -> Self {
        let w = 32.0;
        Player {
            x: (screen_w - w) / 2.0,
            y: if screen::portrait_layout() {
                MOBILE_PLAYER_Y
            } else {
                PLAYER_Y
            },
            width: w,
            height: 32.0,
        }
    }

    pub fn draw_scale(&self) -> f32 {
        if screen::portrait_layout() {
            screen::portrait_gameplay_scale() * MOBILE_PLAYER_SCALE_MULT
        } else {
            screen::portrait_gameplay_scale()
        }
    }

    pub fn effective_width(&self) -> f32 {
        self.width * self.draw_scale()
    }

    /// Handles input and updates player position each frame.
    pub fn update(&mut self, screen_w: f32) {
        let speed = 5.0 * screen::frame_step();
        if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
            self.x -= speed;
        }
        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
            self.x += speed;
        }

        // Clamp to screen bounds (account for border thickness of 4px on each side)
        let margin = 8.0;
        let ship_w = self.effective_width();
        self.x = self.x.max(margin).min(screen_w - ship_w - margin);
    }

    /// Draws the player ship at current position.
    pub fn draw(&self) {
        assets::draw_player_ship(self.x, self.y, self.draw_scale());
    }

    /// Returns the center point of the ship (for bullet spawn).
    pub fn center_x(&self) -> f32 {
        self.x + self.effective_width() / 2.0
    }

    /// Returns the top edge Y coordinate (bullet spawns here).
    pub fn top_y(&self) -> f32 {
        let scale = self.draw_scale();
        self.y - self.height * 0.4 * scale
    }

    /// Checks if a point is within the player's hitbox.
    pub fn contains_point(&self, px: f32, py: f32) -> bool {
        let scale = self.draw_scale();
        let margin = 6.0 * scale;
        let ship_w = self.effective_width();
        px >= self.x - margin
            && px <= self.x + ship_w + margin
            && py >= self.y - self.height * 0.5 * scale - margin
            && py <= self.y + self.height * 0.3 * scale + margin
    }
}

/// A bullet fired by the player (moving upward).
#[derive(Debug, Clone)]
pub struct Bullet {
    pub x: f32,
    pub y: f32,
    /// Speed in pixels per frame.
    pub speed: f32,
}

impl Bullet {
    /// Creates a new bullet at the player's position.
    pub fn new(x: f32, y: f32) -> Self {
        Bullet { x, y, speed: 8.0 }
    }

    /// Updates bullet position each frame. Returns true if off-screen.
    pub fn update(&mut self) -> bool {
        self.y -= self.speed * screen::frame_step();
        self.y < -10.0 // Off top of screen
    }

    /// Draws the bullet at current position.
    pub fn draw(&self) {
        assets::draw_bullet(self.x, self.y);
    }

    /// Checks if this bullet collides with a rectangle (enemy hitbox).
    #[allow(dead_code)]
    pub fn hits_rect(&self, rx: f32, ry: f32, rw: f32, rh: f32) -> bool {
        let margin = 4.0; // Bullet collision radius
        self.x >= rx - margin
            && self.x <= rx + rw + margin
            && self.y >= ry - margin
            && self.y <= ry + rh + margin
    }

    /// Checks if this bullet collides with another point (for enemy bullets).
    #[allow(dead_code)]
    pub fn hits_point(&self, px: f32, py: f32) -> bool {
        let dist = ((self.x - px).powi(2) + (self.y - py).powi(2)).sqrt();
        dist < 8.0
    }
}

/// A bullet fired by enemies (moving downward toward player).
#[derive(Debug, Clone)]
pub struct EnemyBullet {
    pub x: f32,
    pub y: f32,
    /// Speed in pixels per frame.
    pub speed: f32,
}

impl EnemyBullet {
    /// Creates a new enemy bullet at the given position.
    pub fn new(x: f32, y: f32) -> Self {
        EnemyBullet { x, y, speed: 4.0 }
    }

    /// Updates position each frame. Returns true if off-screen (bottom).
    pub fn update(&mut self) -> bool {
        self.y += self.speed * screen::frame_step();
        self.y > 730.0 // Off bottom of the 1280x720 virtual screen
    }

    /// Draws the enemy bullet at current position.
    pub fn draw(&self) {
        assets::draw_enemy_bullet(self.x, self.y);
    }

    /// Checks if this bullet hits a point (player hitbox).
    #[allow(dead_code)]
    pub fn hits_point(&self, px: f32, py: f32) -> bool {
        let dist = ((self.x - px).powi(2) + (self.y - py).powi(2)).sqrt();
        dist < 10.0
    }
}
