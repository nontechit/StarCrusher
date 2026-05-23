use macroquad::prelude::*;

pub const DESKTOP_W: f32 = 1280.0;
pub const DESKTOP_H: f32 = 720.0;
pub const PORTRAIT_W: f32 = 720.0;
pub const PORTRAIT_H: f32 = 1280.0;
pub const WINDOW_W: i32 = 1920;
pub const WINDOW_H: i32 = 1080;

/// Desktop virtual canvas (legacy const aliases).
pub const SCREEN_W: f32 = DESKTOP_W;
pub const SCREEN_H: f32 = DESKTOP_H;

/// Portrait mobile UI text multiplier.
pub const MOBILE_TEXT_SCALE: f32 = 1.75;

pub fn screen_w() -> f32 {
    if portrait_layout() {
        PORTRAIT_W
    } else {
        DESKTOP_W
    }
}

pub fn screen_h() -> f32 {
    if portrait_layout() {
        PORTRAIT_H
    } else {
        DESKTOP_H
    }
}

pub fn mobile_text_size(base: u16) -> u16 {
    if portrait_layout() {
        ((base as f32) * MOBILE_TEXT_SCALE).round() as u16
    } else {
        base
    }
}

pub fn portrait_layout() -> bool {
    screen_height() > screen_width() * 1.05
}

pub fn portrait_gameplay_scale() -> f32 {
    if portrait_layout() {
        1.45
    } else {
        1.0
    }
}

pub fn frame_step() -> f32 {
    (get_frame_time() * 60.0).clamp(0.25, 2.0)
}

pub fn window_conf() -> Conf {
    Conf {
        window_title: "Star Crusher".to_string(),
        window_width: WINDOW_W,
        window_height: WINDOW_H,
        fullscreen: true,
        window_resizable: false,
        high_dpi: true,
        ..Default::default()
    }
}

pub fn use_virtual_screen() {
    set_camera(&virtual_camera());
}

fn virtual_camera() -> Camera2D {
    let w = screen_w();
    let h = screen_h();
    Camera2D {
        target: vec2(w / 2.0, h / 2.0),
        zoom: vec2(2.0 / w, 2.0 / h),
        ..Default::default()
    }
}

pub fn enter_fullscreen() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        request_new_screen_size(WINDOW_W as f32, WINDOW_H as f32);
        set_fullscreen(true);
    }
}

/// Maps screen/touch coordinates to the active virtual game space using the active camera.
pub fn to_virtual_position(position: Vec2) -> Vec2 {
    virtual_camera().screen_to_world(position)
}

pub fn primary_tap_position() -> Option<Vec2> {
    for touch in touches() {
        if touch.phase == TouchPhase::Started {
            return Some(to_virtual_position(touch.position));
        }
    }

    if is_mouse_button_pressed(MouseButton::Left) {
        let (x, y) = mouse_position();
        return Some(to_virtual_position(vec2(x, y)));
    }

    None
}

pub fn primary_pointer_position() -> Option<Vec2> {
    for touch in touches() {
        if matches!(
            touch.phase,
            TouchPhase::Started | TouchPhase::Stationary | TouchPhase::Moved
        ) {
            return Some(to_virtual_position(touch.position));
        }
    }

    if is_mouse_button_down(MouseButton::Left) {
        let (x, y) = mouse_position();
        return Some(to_virtual_position(vec2(x, y)));
    }

    None
}

pub fn primary_release_position() -> Option<Vec2> {
    for touch in touches() {
        if touch.phase == TouchPhase::Ended {
            return Some(to_virtual_position(touch.position));
        }
    }

    if is_mouse_button_released(MouseButton::Left) {
        let (x, y) = mouse_position();
        return Some(to_virtual_position(vec2(x, y)));
    }

    None
}
