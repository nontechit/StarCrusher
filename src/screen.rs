use macroquad::prelude::*;

pub const SCREEN_W: f32 = 1024.0;
pub const SCREEN_H: f32 = 768.0;
pub const WINDOW_W: i32 = 1920;
pub const WINDOW_H: i32 = 1080;

pub fn window_conf() -> Conf {
    Conf {
        window_title: "Star Crusher".to_string(),
        window_width: WINDOW_W,
        window_height: WINDOW_H,
        fullscreen: true,
        window_resizable: false,
        ..Default::default()
    }
}

pub fn use_virtual_screen() {
    set_camera(&Camera2D {
        target: vec2(SCREEN_W / 2.0, SCREEN_H / 2.0),
        zoom: vec2(2.0 / SCREEN_W, 2.0 / SCREEN_H),
        ..Default::default()
    });
}

pub fn enter_fullscreen() {
    request_new_screen_size(WINDOW_W as f32, WINDOW_H as f32);
    set_fullscreen(true);
}
