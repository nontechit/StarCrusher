pub fn bool(chance: f32) -> bool {
    macroquad::rand::gen_range(0.0, 1.0) < chance
}

pub fn f32_range(min: f32, max: f32) -> f32 {
    macroquad::rand::gen_range(min, max)
}

pub fn i32_inclusive(min: i32, max: i32) -> i32 {
    macroquad::rand::gen_range(min, max + 1)
}

pub fn usize_exclusive(max: usize) -> usize {
    if max <= 1 {
        0
    } else {
        macroquad::rand::gen_range(0, max as i32) as usize
    }
}

pub fn shuffle<T>(items: &mut [T]) {
    for i in (1..items.len()).rev() {
        let j = usize_exclusive(i + 1);
        items.swap(i, j);
    }
}
