pub use raylib::prelude::*;
pub use serde::{Deserialize, Serialize};
pub use std::sync::Arc;
pub const SCREEN_WIDTH: i32 = 1200;
pub const SCREEN_HEIGHT: i32 = 900;
pub const DEFAULT_THUMBNAIL_SIZE: i32 = 80;

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct BColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Pos2 {
    pub x: i32,
    pub y: i32,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}
impl Rect {
    pub fn check_collision(&self, pos: Pos2) -> bool {
        pos.x >= self.x && pos.y >= self.y && pos.y < self.y + self.h && pos.x < self.x + self.w
    }
}

impl BColor {
    pub const fn as_rl_color(&self) -> Color {
        Color {
            r: self.r,
            g: self.g,
            b: self.b,
            a: self.a,
        }
    }
    pub const fn from_rl_color(c: Color) -> Self {
        Self {
            r: c.r,
            g: c.g,
            b: c.b,
            a: c.a,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum DrawCall {
    BeginDrawing,
    EndDrawing,
    ClearBackground {
        color: BColor,
    },
    Rectangle {
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        color: BColor,
        drop_shadow: bool,
        outline: bool,
    },
    DrawPixels {
        points: Vec<(Pos2, BColor)>,
        width: f32,
    },
    DrawText {
        x: i32,
        y: i32,
        size: i32,
        contents: String,
        color: BColor,
    },
    DrawImage {
        x: i32,
        y: i32,
        h: i32,
        w: i32,
        contents_ident: String,
    },
    Circle {
        x: i32,
        y: i32,
        rad: i32,
        color: BColor,
        drop_shadow: bool,
        outline: bool,
    },
    Update {
        input: UserInput,
    },
    Scissor {
        x: i32,
        y: i32,
        h: i32,
        w: i32,
    },
    LoadImage {
        name: String,
        width: i32,
        height: i32,
        data: Arc<[BColor]>,
    },
    UnloadedImage {
        name: String,
    },
    EndScissor,
    Exiting,
}

#[derive(Debug, Clone, Copy)]
pub enum SysUiMode {
    Relative,
    Sequential,
    Absolute,
}

#[derive(Debug, Clone, Copy)]
pub struct Div {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub vertical: bool,
    pub thumbnail_size: i32,
    pub mode: SysUiMode,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserInput {
    pub pressed_keys: Vec<char>,
    pub mouse_x: i32,
    pub mouse_y: i32,
    pub mouse_dx: f32,
    pub mouse_dy: f32,
    pub scroll_amount: i32,
    pub left_mouse_down: bool,
    pub right_mouse_down: bool,
    pub left_mouse_pressed: bool,
    pub right_mouse_pressed: bool,
    pub left_mouse_released: bool,
    pub right_mouse_released: bool,
    pub left_arrow_pressed: bool,
    pub right_arrow_pressed: bool,
}
impl Default for UserInput {
    fn default() -> Self {
        Self::new()
    }
}

impl UserInput {
    pub fn new() -> Self {
        
        UserInput {
            pressed_keys: Vec::new(),
            mouse_x: 0,
            mouse_y: 0,
            mouse_dx: 0.,
            mouse_dy: 0.,
            left_mouse_down: false,
            right_mouse_down: false,
            left_mouse_pressed: false,
            right_mouse_pressed: false,
            left_mouse_released: false,
            right_mouse_released: false,
            right_arrow_pressed: false,
            left_arrow_pressed: false,
            scroll_amount: 0,
        }
    }
    pub fn reset(&mut self) {
        self.pressed_keys.clear();
        self.left_mouse_pressed = false;
        self.left_mouse_released = false;
        self.right_mouse_released = false;
        self.right_mouse_pressed = false;
        self.right_arrow_pressed = false;
        self.left_arrow_pressed = false;
        self.mouse_dx = 0.;
        self.mouse_dy = 0.;
    }
}
