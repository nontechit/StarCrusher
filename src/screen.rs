use macroquad::prelude::*;

pub const SCREEN_W: f32 = 1280.0;
pub const SCREEN_H: f32 = 720.0;
pub const WINDOW_W: i32 = 1920;
pub const WINDOW_H: i32 = 1080;

pub fn mobile_text_size(base: u16) -> u16 {
    if screen_height() > screen_width() * 1.15 {
        (base as f32 * 0.72).max(12.0) as u16
    } else {
        base
    }
}

pub fn portrait_layout() -> bool {
    screen_height() > screen_width() * 1.15
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
    Camera2D {
        target: vec2(SCREEN_W / 2.0, SCREEN_H / 2.0),
        zoom: vec2(2.0 / SCREEN_W, 2.0 / SCREEN_H),
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

/// Maps screen/touch coordinates to the 1280x720 virtual game space using the active camera.
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
