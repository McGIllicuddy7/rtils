pub use raylib::prelude::*;
pub use serde::{Deserialize, Serialize};
pub use std::sync::Arc;
use std::{f32, f64};
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

#[derive(Debug, Clone)]
pub struct Pallete {
    pub colors: [BColor; 256],
}
impl Pallete {
    pub fn new(seed: BColor) -> Self {
        let hsv = raylib::color::Color::color_to_hsv(&seed.as_rl_color());
        let theta = if hsv.z == 0.0 || hsv.y == 0.0 {
            0.0
        } else {
            hsv.x
        };
        let mut thetas = [0.0; 15];
        for i in 0..15 {
            thetas[i] = ((theta as f64 + 2.0 * f64::consts::PI / (15.0) * i as f64) as f32)
                / (f32::consts::TAU);
        }

        let mut colors = [BColor {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        }; 256];
        let mut i = 0;
        for value in 0..4 {
            let bv = if value == 3 {
                1.0
            } else if value == 2 {
                0.75
            } else if value == 1 {
                0.5
            } else {
                0.25
            };
            for saturation in 0..4 {
                let sat = if saturation == 3 {
                    1.0
                } else if saturation == 2 {
                    0.5
                } else if saturation == 1 {
                    0.25
                } else {
                    0.0
                };
                for hue in 0..15 {
                    let h = thetas[hue];
                    let s = sat;
                    let v = bv;
                    colors[i] =
                        BColor::from_rl_color(raylib::color::Color::color_from_hsv(h, s, v));
                    i += 1;
                }
            }
        }
        colors[i] = BColor::from_rl_color(Color::WHITE); //1
        i += 1;
        colors[i] = BColor::from_rl_color(Color::BLACK); //2
        i += 1;
        colors[i] = BColor::from_rl_color(Color::BLUE); //3
        i += 1;
        colors[i] = BColor::from_rl_color(Color::GREEN); //4
        i += 1;
        colors[i] = BColor::from_rl_color(Color::RED); //5
        i += 1;
        colors[i] = BColor::from_rl_color(Color::YELLOW); //6
        i += 1;
        colors[i] = BColor::from_rl_color(Color::PURPLE); //7
        i += 1;
        colors[i] = BColor::from_rl_color(Color::GRAY); //8
        i += 1;
        colors[i] = BColor::from_rl_color(Color::DARKGRAY); //9
        i += 1;
        colors[i] = BColor::from_rl_color(Color::DARKBLUE); //10
        i += 1;
        colors[i] = BColor::from_rl_color(Color::DARKRED); //11
        i += 1;
        colors[i] = BColor::from_rl_color(Color::DARKGREEN); //12
        i += 1;
        colors[i] = BColor::from_rl_color(Color::DARKVIOLET); //13
        i += 1;
        colors[i] = BColor::from_rl_color(Color::VIOLET); //14
        i += 1;
        colors[i] = BColor::from_rl_color(Color::CYAN); //15
        i += 1;
        colors[i] = BColor::from_rl_color(Color::DARKCYAN); //16
        let out = Self { colors };
        //   println!("{:#?}", out);
        out
    }
    pub fn as_rl(&self) -> [Color; 256] {
        let mut out = [Color::BLACK; 256];
        for i in 0..256 {
            out[i] = self.colors[i].as_rl_color();
        }
        out
    }
    pub fn as_rl_vec(&self) -> [Vector4; 256] {
        let mut out = [Vector4::new(0.0, 0.0, 0.0, 0.0); 256];
        for i in 0..256 {
            let c = self.colors[i];
            out[i] = Vector4::new(
                c.r as f32 / 256.0,
                c.g as f32 / 256.,
                c.b as f32 / 256.0,
                c.a as f32 / 256.0,
            );
        }
        out
    }
}
