use raylib::prelude::*;
pub use serde::{Deserialize, Serialize};
pub use std::sync::Arc;
use std::{
    collections::{BTreeMap, HashMap},
    time::Duration,
};

use crate::rtils::rtils_useful::BPipe;
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
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Sprite {
    pub name: String,
}
impl Sprite {}

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
    DrawSprite {
        x: i32,
        y: i32,
        h: i32,
        w: i32,
        contents: Sprite,
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
    LoadedImage {
        name: String,
        width: i32,
        height: i32,
        color: BColor,
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
    pub mode: SysUiMode,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserInput {
    pressed_keys: Vec<char>,
    mouse_x: i32,
    mouse_y: i32,
    mouse_dx: f32,
    mouse_dy: f32,
    scroll_amount: i32,
    left_mouse_down: bool,
    right_mouse_down: bool,
    left_mouse_pressed: bool,
    right_mouse_pressed: bool,
    left_mouse_released: bool,
    right_mouse_released: bool,
    left_arrow_pressed: bool,
    right_arrow_pressed: bool,
}
impl UserInput {
    pub fn new() -> Self {
        let inp = UserInput {
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
        };
        inp
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
pub struct SysHandle {
    handle: BPipe<DrawCall>,
    cx: i32,
    cy: i32,
    w: i32,
    h: i32,
    padding_x: i32,
    padding_y: i32,
    ui_mode: SysUiMode,
    text_ratios: BTreeMap<char, f64>,
    max_ratio: f64,
    div_stack: Vec<Div>,
    text_color: BColor,
    background_color: BColor,
    object_color: BColor,
    object_pressed_color: BColor,
    shadows: bool,
    outline: bool,
    user_input: UserInput,
    should_exit: bool,
    queue: Vec<DrawCall>,
    text_box_data: HashMap<String, TextBoxData>,
    scroll_box_data: HashMap<String, f32>,
}
pub fn text_ratios(handle: &RaylibHandle) -> (f64, BTreeMap<char, f64>) {
    let mut out = BTreeMap::new();
    let mut text = String::new();
    let mut max = 0.0;
    for i in 1..127_u8 {
        let c = i as char;
        if !c.is_control() {
            text.push(c);
            let mes = handle.measure_text(&text, 10);
            let xs = (mes as f64 + 1.) / 10.0;
            if xs > max {
                max = xs;
            }
            out.insert(c, xs);
            text.clear();
        }
    }
    (max, out)
}

#[derive(Clone)]
pub struct TextBoxData {
    pub selected: bool,
    pub text: String,
    pub cursor: usize,
    pub selected_section: Option<(usize, usize)>,
}
impl TextBoxData {
    pub fn new() -> Self {
        Self {
            selected: false,
            text: String::new(),
            cursor: 0,
            selected_section: None,
        }
    }
}
impl SysHandle {
    pub fn should_exit(&self) -> bool {
        self.should_exit
    }
    pub fn char_width(&self, c: char, h: i32) -> Option<i32> {
        if let Some(r) = self.text_ratios.get(&c) {
            let out = (h as f64 * r) as i32;
            Some(out)
        } else {
            if c == '\n' || c == '\r' {
                None
            } else {
                Some((self.max_ratio * h as f64) as i32)
            }
        }
    }
    pub fn measure_text(&self, text: &str, h: i32) -> Option<i32> {
        if text.contains('\r') || text.contains('\n') {
            None
        } else {
            let mut out = 0;
            for i in text.chars() {
                let rat = *self.text_ratios.get(&i)?;
                let w = (h as f64 * rat) as i32;
                out += w;
            }
            Some(out)
        }
    }

    pub fn char_location(
        &self,
        index: usize,
        x: i32,
        y: i32,
        text: &str,
        text_height: i32,
        max_w: i32,
    ) -> Pos2 {
        let mut cx = x;
        let mut cy = y;
        let mut id = 0;
        for i in text.chars() {
            if i == '\r' {
                cx = x;
            } else if i == '\n' {
                cx = x;
                cy += text_height;
            } else {
                let wp = self.char_width(i, text_height).unwrap();
                if cx + wp >= max_w + x {
                    cx = x + wp;
                    cy += text_height;
                } else {
                    if id == index {
                        return Pos2 { x: cx, y: cy };
                    }
                    cx += wp;
                }
            }

            id += 1;
        }
        return Pos2 { x: cx, y: cy };
    }
    pub fn nearest_char_to(
        &self,
        pos_x: i32,
        pos_y: i32,
        x: i32,
        y: i32,
        text: &str,
        text_height: i32,
        max_w: i32,
    ) -> usize {
        if text.is_empty() {
            return 0;
        }
        let mut cx = x;
        let mut cy = y;
        let mut id = 0;
        let mut min_id = text.len() - 1;
        let mut min_dist = 100000;
        for i in text.chars() {
            if i == '\r' {
                cx = x;
            } else if i == '\n' {
                cx = x;
                cy += text_height;
            } else {
                let wp = self.char_width(i, text_height).unwrap();
                if cx + wp >= max_w + x {
                    cx = x + wp;
                    cy += text_height;
                } else {
                    cx += wp;
                }
            }
            let dist = (pos_y - cy) * (pos_y - cy) + (pos_x - cx) * (pos_x - cx);
            if dist < min_dist {
                min_id = id;
                min_dist = dist;
            }
            id += 1;
        }
        min_id
    }

    pub fn split_by_required_line(&self, text: &str, h: i32, max_w: i32) -> Vec<String> {
        let mut out = Vec::new();
        let mut current = String::new();
        let mut w = 0;
        for i in text.chars() {
            if i == '\r' {
                current.push(i);
                out.push(current);
                current = String::new();
                w = 0;
            } else if i == '\n' {
                out.push(current);
                current = String::new();
                w = 0;
            } else {
                let wp = self.char_width(i, h).unwrap();
                if w + wp >= max_w {
                    w = wp;
                    out.push(current);
                    current = String::new();
                    current.push(i);
                } else {
                    current.push(i);
                    w += wp;
                }
            }
        }
        if !current.is_empty() {
            out.push(current);
        }
        out
    }

    pub fn text_get_height_and_lines(&self, text: &str, h: i32, max_w: i32) -> (i32, Vec<String>) {
        let split = self.split_by_required_line(text, h, max_w);
        let mut count = 0;
        for i in &split {
            if !i.ends_with('\r') {
                count += h;
            }
        }
        (count, split)
    }

    pub fn set_sys_ui_mode(&mut self, mode: SysUiMode) {
        self.ui_mode = mode;
    }

    pub fn get_div(&self) -> Div {
        if !self.div_stack.is_empty() {
            let out = self.div_stack[self.div_stack.len() - 1];
            out
        } else {
            Div {
                x: 0,
                y: 0,
                w: self.w,
                h: self.h,
                vertical: false,
                mode: SysUiMode::Sequential,
            }
        }
    }

    pub fn get_absolute_pos(&self, pos: Pos2) -> Pos2 {
        match self.ui_mode {
            SysUiMode::Absolute => pos,
            SysUiMode::Relative => {
                let div = self.get_div();
                Pos2 {
                    x: div.x + pos.x,
                    y: div.y + pos.y,
                }
            }
            SysUiMode::Sequential => {
                let out = Pos2 {
                    x: self.cx + pos.x,
                    y: self.cy + pos.y,
                };
                out
            }
        }
    }

    pub fn update_cursor(&mut self, prev_bounds: Rect) {
        match self.ui_mode {
            SysUiMode::Sequential => {}
            SysUiMode::Absolute | SysUiMode::Relative => {
                return;
            }
        }
        let div = self.get_div();
        if div.vertical {
            self.cx = div.x;
            self.cy = prev_bounds.y + prev_bounds.h + 5;
        } else {
            self.cy = div.y;
            self.cx = prev_bounds.x + prev_bounds.w + 5;
        }
    }

    pub fn draw_text_exp(&mut self, x: i32, y: i32, h: i32, max_w: i32, text: &str) {
        let (height, texts) = self.text_get_height_and_lines(text, h, max_w);
        let base = self.get_absolute_pos(Pos2 { x, y });
        let mut current = base;
        for i in texts {
            self.queue.push(DrawCall::DrawText {
                x: current.x,
                y: current.y,
                size: h,
                contents: i,
                color: self.text_color,
            });
            current.y += h;
        }
        self.update_cursor(Rect {
            x: base.x,
            y: base.y,
            w: max_w,
            h: height,
        });
    }

    pub fn draw_text(&mut self, h: i32, max_w: i32, text: &str) {
        self.draw_text_exp(self.padding_x, self.padding_y, h, max_w, text);
    }

    pub fn draw_box_exp(&mut self, x: i32, y: i32, w: i32, h: i32) {
        let base = self.get_absolute_pos(Pos2 { x, y });
        self.queue.push(DrawCall::Rectangle {
            x,
            y,
            w,
            h,
            color: self.object_color,
            drop_shadow: self.shadows,
            outline: self.outline,
        });
        self.update_cursor(Rect {
            x: base.x,
            y: base.y,
            w: w,
            h: h,
        });
    }

    pub fn draw_box(&mut self, w: i32, h: i32) {
        self.draw_box_exp(self.padding_x, self.padding_y, w, h)
    }

    pub fn draw_circle_exp(&mut self, x: i32, y: i32, rad: i32) {
        let base = self.get_absolute_pos(Pos2 { x, y });
        self.queue.push(DrawCall::Circle {
            x,
            y,
            rad,
            color: self.object_color,
            drop_shadow: self.shadows,
            outline: self.outline,
        });
        self.update_cursor(Rect {
            x: base.x,
            y: base.y,
            w: rad * 2,
            h: rad * 2,
        });
    }

    pub fn draw_circle(&mut self, rad: i32) {
        self.draw_circle_exp(self.padding_x, self.padding_y, rad)
    }

    pub fn draw_sprite_exp(&mut self, x: i32, y: i32, w: i32, h: i32, sprite: Sprite) {
        let base = self.get_absolute_pos(Pos2 { x, y });
        self.queue.push(DrawCall::DrawSprite {
            x,
            y,
            h,
            w,
            contents: sprite,
        });
        self.update_cursor(Rect {
            x: base.x,
            y: base.y,
            w,
            h,
        });
    }

    pub fn draw_sprite(&mut self, w: i32, h: i32, sprite: Sprite) {
        self.draw_sprite_exp(self.padding_x, self.padding_y, w, h, sprite)
    }

    pub fn draw_pixels(&mut self, points: Vec<(Pos2, BColor)>, width: f32) {
        if points.len() == 0 {
            return;
        }
        let mut mpoints = points;
        mpoints.iter_mut().for_each(|(x, _)| {
            *x = self.get_absolute_pos(*x);
        });
        let mut min = mpoints[0].0;
        let mut max = mpoints[0].0;
        mpoints.iter().for_each(|(x, _)| {
            if x.x < min.x {
                min.x = x.x;
            }
            if x.y < min.y {
                min.y = x.y;
            }
            if x.x > max.x {
                max.x = x.x;
            }
            if x.y > max.y {
                max.y = x.y;
            }
        });
        self.queue.push(DrawCall::DrawPixels {
            points: mpoints,
            width,
        });
        let w = max.x - min.x;
        let h = max.y - min.y;
        self.update_cursor(Rect {
            x: min.x,
            y: min.y,
            w,
            h,
        });
    }

    pub fn draw_button_exp(
        &mut self,
        x: i32,
        y: i32,
        w: i32,
        text_height: i32,
        text: &str,
    ) -> bool {
        let (mut h, texts) = self.text_get_height_and_lines(text, text_height, w - 5);
        h += 10;
        let pos = self.get_absolute_pos(Pos2 { x: x, y: y });
        let did_hit = self.user_input.mouse_x >= pos.x
            && self.user_input.mouse_y >= pos.y
            && self.user_input.mouse_x < pos.x + w
            && self.user_input.mouse_y < pos.y + h;
        let col = if did_hit && self.user_input.left_mouse_down {
            self.object_pressed_color
        } else {
            self.object_color
        };
        self.queue.push(DrawCall::Rectangle {
            x: pos.x,
            y: pos.y,
            w,
            h,
            color: col,
            drop_shadow: self.shadows,
            outline: self.outline,
        });
        let mut current = pos;
        current.x += 5;
        current.y += 5;
        for i in texts {
            self.queue.push(DrawCall::DrawText {
                x: current.x,
                y: current.y,
                size: text_height,
                contents: i,
                color: self.text_color,
            });
            current.y += text_height;
        }
        self.update_cursor(Rect {
            x: pos.x,
            y: pos.y,
            w,
            h,
        });

        self.user_input.mouse_x >= pos.x
            && self.user_input.mouse_y >= pos.y
            && self.user_input.mouse_x < pos.x + w
            && self.user_input.mouse_y < pos.y + h
            && self.user_input.left_mouse_released
    }

    pub fn draw_button(&mut self, w: i32, text_height: i32, text: &str) -> bool {
        self.draw_button_exp(self.padding_x, self.padding_y, w, text_height, text)
    }
    pub fn begin_div_exp(
        &mut self,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        vertical: bool,
        mode: SysUiMode,
    ) {
        let pos = self.get_absolute_pos(Pos2 { x, y });
        self.div_stack.push(Div {
            x: pos.x,
            y: pos.y,
            w,
            h,
            vertical,
            mode,
        });
        self.ui_mode = mode;
        match self.ui_mode {
            SysUiMode::Absolute | SysUiMode::Relative => {}
            SysUiMode::Sequential => {
                self.cx = pos.x;
                self.cy = pos.y;
            }
        }
    }

    pub fn begin_div(&mut self, w: i32, h: i32) {
        self.begin_div_exp(
            self.padding_x,
            self.padding_y,
            w,
            h,
            true,
            SysUiMode::Sequential,
        );
    }

    pub fn end_div(&mut self) {
        let x = self.div_stack.pop().unwrap();
        self.ui_mode = self.get_div().mode;
        self.update_cursor(Rect {
            x: x.x,
            y: x.y,
            w: x.w,
            h: x.h,
        });
    }

    pub fn begin_drawing(&mut self) {
        self.user_input.reset();
        while let Ok(Some(x)) = self.handle.recieve() {
            match x {
                DrawCall::Update { input } => {
                    self.user_input = input;
                }
                DrawCall::Exiting => {
                    self.should_exit = true;
                }
                _ => {
                    continue;
                }
            }
        }
        self.div_stack.clear();
        self.queue.clear();
        self.queue.push(DrawCall::BeginDrawing);
        self.queue.push(DrawCall::ClearBackground {
            color: self.background_color,
        });
        self.cx = 0;
        self.cy = 0;
        self.ui_mode = SysUiMode::Sequential;
    }

    pub fn end_drawing(&mut self) {
        self.queue.push(DrawCall::EndDrawing);
        self.handle.send_multiple(&mut self.queue).unwrap();
        self.div_stack.clear();
        self.queue.clear();
        self.cx = 0;
        self.cy = 0;
        self.ui_mode = SysUiMode::Sequential;
        std::thread::sleep(Duration::from_millis(8));
    }

    pub fn set_cursor(&mut self, x: i32, y: i32) {
        self.cx = x;
        self.cy = y;
    }

    pub fn draw_text_scroll_box_exp<'a, T>(
        &mut self,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        text_height: i32,
        amount: f32,
        upside_down: bool,
        objects: &[T],
        as_string: impl Fn(&T) -> String,
    ) -> f32 {
        let base = self.get_absolute_pos(Pos2 { x, y });
        let mut height = 0;
        let strings: Vec<String> = objects
            .iter()
            .flat_map(|i| {
                let (dh, cons) = self.text_get_height_and_lines(&as_string(i), text_height, w - 10);
                height += dh + 10;
                cons
            })
            .collect();
        self.queue.push(DrawCall::Rectangle {
            x: base.x,
            y: base.y,
            w,
            h,
            color: self.object_color,
            drop_shadow: self.shadows,
            outline: self.outline,
        });
        self.queue.push(DrawCall::Scissor {
            x: base.x,
            y: base.y,
            h,
            w,
        });
        let delta = height as f32 * amount;
        let base_y = if !upside_down {
            base.y as f32 - delta
        } else {
            base.y as f32 - delta + h as f32 - text_height as f32
        };
        let mut y = base_y;
        for i in strings {
            let pos = Pos2 {
                x: base.x,
                y: y as i32,
            };
            let er = i.ends_with('\r');
            if !(y as i32 + text_height < base.y || y as i32 > base.y + h) {
                self.queue.push(DrawCall::DrawText {
                    x: pos.x + 2,
                    y: pos.y,
                    size: text_height,
                    contents: i,
                    color: self.text_color,
                });
            }
            if !er {
                if !upside_down {
                    y += text_height as f32;
                } else {
                    y -= text_height as f32;
                }
            }
        }
        let bx = Rect {
            x: base.x + w - 8,
            y: base.y + ((h as f32 - 16.0) * amount) as i32,
            w: 12,
            h: 16,
        };
        self.queue.push(DrawCall::Rectangle {
            x: bx.x,
            y: bx.y,
            w: bx.w,
            h: bx.h,
            color: self.object_color,
            drop_shadow: false,
            outline: true,
        });
        self.queue.push(DrawCall::DrawPixels {
            points: vec![
                (
                    Pos2 { x: bx.x, y: base.y },
                    BColor::from_rl_color(Color::DARKGRAY),
                ),
                (
                    Pos2 {
                        x: bx.x,
                        y: base.y + h,
                    },
                    BColor::from_rl_color(Color::DARKGRAY),
                ),
            ],
            width: 1.0,
        });

        self.queue.push(DrawCall::EndScissor);
        let hovered = self.user_input.mouse_x >= base.x
            && self.user_input.mouse_y >= base.y
            && self.user_input.mouse_x < base.x + w
            && self.user_input.mouse_y < base.y + h;
        let mut out = amount;
        let bx = Rect {
            x: bx.x - 5,
            y: bx.y - 5,
            w: bx.w + 10,
            h: bx.h + 10,
        };
        if self.user_input.left_mouse_down
            && bx.check_collision(Pos2 {
                x: self.user_input.mouse_x,
                y: self.user_input.mouse_y,
            })
        {
            out += self.user_input.mouse_dy as f32 / h as f32 * 1.5;
        } else if hovered {
            out -= self.user_input.scroll_amount as f32 / h as f32;
        }

        self.update_cursor(Rect {
            x: base.x,
            y: base.y,
            w,
            h,
        });
        out.clamp(0.0, 1.)
    }

    pub fn draw_button_scroll_box_exp<'a, T>(
        &mut self,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        text_height: i32,
        amount: f32,
        upside_down: bool,
        objects: &[T],
        as_string: impl Fn(&T) -> String,
    ) -> (f32, Option<usize>) {
        let base = self.get_absolute_pos(Pos2 { x, y });
        let mut height = 0;
        let strings: Vec<(i32, Vec<String>)> = objects
            .iter()
            .map(|i| {
                let (dh, cons) = self.text_get_height_and_lines(&as_string(i), text_height, w - 10);
                height += dh + 10;
                (dh + 5, cons)
            })
            .collect();
        self.queue.push(DrawCall::Rectangle {
            x: base.x,
            y: base.y,
            w,
            h,
            color: self.object_color,
            drop_shadow: self.shadows,
            outline: self.outline,
        });
        self.queue.push(DrawCall::Scissor {
            x: base.x,
            y: base.y,
            h,
            w,
        });
        let delta = height as f32 * amount;
        let base_y = if !upside_down {
            base.y as f32 - delta
        } else {
            base.y as f32 - delta + h as f32 - text_height as f32
        };
        let mut y = base_y;
        let mut idx = 0;
        let mut hit = None;
        for i in strings {
            let pos = Pos2 {
                x: base.x,
                y: y as i32,
            };
            if !(y as i32 + (i.0 as i32) < base.y || (y as i32) > base.y + h) {
                let did_hit = self.user_input.mouse_x >= pos.x
                    && self.user_input.mouse_y >= pos.y
                    && self.user_input.mouse_x < pos.x + w
                    && self.user_input.mouse_y < pos.y + i.0;
                let col = if did_hit && self.user_input.left_mouse_down {
                    self.object_pressed_color
                } else {
                    self.object_color
                };
                self.queue.push(DrawCall::Rectangle {
                    x: pos.x,
                    y: pos.y,
                    w: w - 11,
                    h: i.0,
                    color: col,
                    drop_shadow: self.shadows,
                    outline: self.outline,
                });
                let mut current = pos;
                current.x += 2;
                current.y += 2;
                for i in i.1 {
                    self.queue.push(DrawCall::DrawText {
                        x: current.x,
                        y: current.y,
                        size: text_height,
                        contents: i,
                        color: self.text_color,
                    });
                    current.y += text_height;
                }

                if did_hit && self.user_input.left_mouse_released {
                    hit = Some(idx);
                }
            }
            if !upside_down {
                y += i.0 as f32 + 5.0;
            } else {
                y -= i.0 as f32 + 5.0;
            }
            idx += 1;
        }
        let bx = Rect {
            x: base.x + w - 8,
            y: base.y + ((h as f32 - 16.0) * amount) as i32,
            w: 12,
            h: 16,
        };
        self.queue.push(DrawCall::Rectangle {
            x: bx.x,
            y: bx.y,
            w: bx.w,
            h: bx.h,
            color: self.object_color,
            drop_shadow: false,
            outline: true,
        });
        self.queue.push(DrawCall::DrawPixels {
            points: vec![
                (
                    Pos2 { x: bx.x, y: base.y },
                    BColor::from_rl_color(Color::DARKGRAY),
                ),
                (
                    Pos2 {
                        x: bx.x,
                        y: base.y + h,
                    },
                    BColor::from_rl_color(Color::DARKGRAY),
                ),
            ],
            width: 1.0,
        });
        self.queue.push(DrawCall::EndScissor);
        let hovered = self.user_input.mouse_x >= base.x
            && self.user_input.mouse_y >= base.y
            && self.user_input.mouse_x < base.x + w
            && self.user_input.mouse_y < base.y + h;
        let mut out = amount;
        let bx = Rect {
            x: bx.x - 5,
            y: bx.y - 5,
            w: bx.w + 10,
            h: bx.h + 10,
        };
        if self.user_input.left_mouse_down
            && bx.check_collision(Pos2 {
                x: self.user_input.mouse_x,
                y: self.user_input.mouse_y,
            })
        {
            out += self.user_input.mouse_dy as f32 / h as f32 * 1.5;
        } else if hovered {
            out -= self.user_input.scroll_amount as f32 / h as f32;
        }
        self.update_cursor(Rect {
            x: base.x,
            y: base.y,
            w,
            h,
        });
        (out.clamp(0.0, 1.0), hit)
    }

    pub fn draw_button_scroll_box_saved_exp<T>(
        &mut self,
        name: &str,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        text_height: i32,
        upside_down: bool,
        objects: &[T],
        as_string: impl Fn(&T) -> String,
    ) -> Option<usize> {
        let amnt = if let Some(k) = self.scroll_box_data.get(name) {
            *k
        } else {
            self.scroll_box_data.insert(name.to_string(), 0.0);
            0.0
        };
        let (a, x) = self.draw_button_scroll_box_exp(
            x,
            y,
            w,
            h,
            text_height,
            amnt,
            upside_down,
            objects,
            as_string,
        );
        *self.scroll_box_data.get_mut(name).unwrap() = a;
        x
    }

    pub fn draw_text_scroll_box_saved_exp<T>(
        &mut self,
        name: &str,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        text_height: i32,
        upside_down: bool,
        objects: &[T],
        as_string: impl Fn(&T) -> String,
    ) {
        let amnt = if let Some(k) = self.scroll_box_data.get(name) {
            *k
        } else {
            self.scroll_box_data.insert(name.to_string(), 0.0);
            0.0
        };
        let a = self.draw_text_scroll_box_exp(
            x,
            y,
            w,
            h,
            text_height,
            amnt,
            upside_down,
            objects,
            as_string,
        );
        *self.scroll_box_data.get_mut(name).unwrap() = a;
    }

    pub fn text_user_input_exp(
        &mut self,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        text_height: i32,
        inp: TextBoxData,
    ) -> (Option<String>, TextBoxData) {
        let pos = self.get_absolute_pos(Pos2 { x, y });
        let bx = Rect {
            x: pos.x,
            y: pos.y,
            w,
            h,
        };
        let mut out_selected = inp.selected;
        let mut output = inp.text;
        let mut out_cursor = inp.cursor;
        let selected_section = None;
        let mut returned = None;
        if self.user_input.left_mouse_released {
            if bx.check_collision(Pos2 {
                x: self.user_input.mouse_x,
                y: self.user_input.mouse_y,
            }) {
                out_selected = true;
            } else {
                out_selected = false;
            }
        }
        if out_selected {
            for i in self.user_input.pressed_keys.clone() {
                if i == '\n' {
                    returned = Some(output);
                    output = String::new();
                } else if i as i32 == 127 {
                    if out_cursor < output.len() && out_cursor > 0 {
                        output.remove(out_cursor - 1);
                        if out_cursor != 0 {
                            out_cursor -= 1;
                        }
                    } else if out_cursor != 0 {
                        out_cursor -= 1;
                        output.pop();
                    }
                } else {
                    if out_cursor >= output.len() {
                        output.push(i);
                    } else {
                        output.insert(out_cursor, i);
                    }
                    out_cursor += 1;
                }
            }
            if self.user_input.left_arrow_pressed {
                if out_cursor > 0 {
                    out_cursor -= 1;
                }
            }
            if self.user_input.right_arrow_pressed {
                if out_cursor <= output.len() {
                    out_cursor += 1;
                }
            }
            out_cursor = out_cursor.clamp(0, output.len() + 1);
        }
        self.queue.push(DrawCall::Rectangle {
            x: pos.x,
            y: pos.y,
            w,
            h,
            color: if out_selected {
                self.object_pressed_color
            } else {
                self.object_color
            },
            drop_shadow: self.shadows,
            outline: self.outline,
        });
        self.queue.push(DrawCall::Scissor {
            x: pos.x,
            y: pos.y,
            h,
            w,
        });
        let (th, txt) = self.text_get_height_and_lines(&output, text_height, w - 2);
        let mut cursor = pos;
        if th > h {
            cursor.y -= th - h;
        }
        cursor.x += 2;
        if self.user_input.left_mouse_pressed {
            let x = self.user_input.mouse_x;
            let y = self.user_input.mouse_y;
            let o2 = output.clone() + " ";
            let id = self.nearest_char_to(x, y, cursor.x, cursor.y, &o2, text_height, w);
            out_cursor = id;
        }
        let lok = self.char_location(out_cursor, cursor.x, cursor.y, &output, text_height, w);
        self.queue.push(DrawCall::Rectangle {
            x: lok.x - 2,
            y: lok.y,
            w: 2,
            h: text_height,
            color: self.text_color,
            drop_shadow: false,
            outline: false,
        });
        for i in txt {
            self.queue.push(DrawCall::DrawText {
                x: cursor.x,
                y: cursor.y,
                size: text_height,
                contents: i,
                color: self.text_color,
            });
            cursor.y += text_height;
        }
        self.queue.push(DrawCall::EndScissor);
        self.update_cursor(Rect {
            x: pos.x,
            y: pos.y,
            w,
            h,
        });
        (
            returned,
            TextBoxData {
                selected: out_selected,
                text: output,
                cursor: out_cursor,
                selected_section,
            },
        )
    }

    pub fn text_user_input_saved_exp(
        &mut self,
        name: &str,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        text_height: i32,
    ) -> Option<String> {
        let data = if let Some(k) = self.text_box_data.get(name) {
            k.clone()
        } else {
            let tmp = TextBoxData::new();
            self.text_box_data.insert(name.to_string(), tmp.clone());
            tmp
        };
        let (out, d2) = self.text_user_input_exp(x, y, w, h, text_height, data);
        *self.text_box_data.get_mut(name).unwrap() = d2;
        out
    }
}

pub struct Dos {
    pub image: Image,
    pub render_texture: Option<RenderTexture2D>,
    pub loaded_textures: HashMap<String, Texture2D>,
    pub w: i32,
    pub h: i32,
}

impl Dos {
    pub fn draw(&mut self, handle: &mut RaylibDrawHandle, _thread: &RaylibThread) {
        handle.draw_texture_pro(
            self.render_texture.as_ref().unwrap(),
            Rectangle::new(0.0, 0.0, 640., -480.0),
            Rectangle::new(0.0, 0.0, self.w as f32, self.h as f32),
            Vector2::zero(),
            0.0,
            Color::WHITE,
        );
    }

    pub fn new() -> Self {
        Self {
            image: Image::gen_image_color(640, 480, Color::WHITE),
            render_texture: None,
            loaded_textures: HashMap::new(),
            w: 640,
            h: 480,
        }
    }

    pub fn setup(&mut self, handle: &mut RaylibHandle, thread: &RaylibThread) {
        self.render_texture = Some(handle.load_render_texture(thread, 640, 480).unwrap());
    }
}

pub struct DosRt {
    pub dos: Dos,
    pub cmd_pipeline: BPipe<DrawCall>,
    pub frame: Vec<DrawCall>,
    pub recieving_frame: bool,
    pub should_draw: bool,
    pub should_exit: bool,
    pub input: UserInput,
}

impl DosRt {
    pub fn update_cmds(&mut self) {
        while let Ok(Some(next)) = self.cmd_pipeline.recieve() {
            if self.recieving_frame {
                match next {
                    DrawCall::EndDrawing => {
                        self.should_draw = true;
                        self.recieving_frame = false;
                        break;
                    }
                    DrawCall::Exiting => {
                        self.should_exit = true;
                        self.recieving_frame = false;
                        break;
                    }
                    _ => {
                        self.frame.push(next);
                    }
                }
            } else {
                match next {
                    DrawCall::BeginDrawing => {
                        self.recieving_frame = true;
                    }
                    _ => continue,
                }
            }
        }
    }

    pub fn run_loop(&mut self, mut handle: RaylibHandle, thread: RaylibThread) {
        while !self.should_exit {
            self.update_cmds();
            if self.should_draw {
                self.draw(&mut handle, &thread);
                if self.should_exit {
                    break;
                }
                self.render(&mut handle, &thread);
            }
        }
    }

    pub fn draw(&mut self, handle: &mut RaylibHandle, thread: &RaylibThread) {
        if let Some(key) = handle.get_char_pressed() {
            self.input.pressed_keys.push(key);
        }
        if handle.window_should_close() {
            self.should_exit = true;
            self.dos.render_texture = None;
            self.cmd_pipeline.send(DrawCall::Exiting).unwrap();
            return;
        }
        if handle.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
            self.input.left_mouse_down = true;
        } else {
            self.input.left_mouse_down = false;
        }
        if handle.is_mouse_button_down(MouseButton::MOUSE_BUTTON_RIGHT) {
            self.input.right_mouse_down = true;
        } else {
            self.input.right_mouse_down = false;
        }
        if handle.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            self.input.left_mouse_pressed = true;
        } else {
            self.input.left_mouse_pressed = false;
        }
        if handle.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT) {
            self.input.left_mouse_released = true;
        } else {
            self.input.left_mouse_released = false;
        }
        if handle.is_mouse_button_released(MouseButton::MOUSE_BUTTON_RIGHT) {
            self.input.right_mouse_released = true;
        } else {
            self.input.right_mouse_released = false;
        }
        if handle.is_key_pressed(KeyboardKey::KEY_BACKSPACE) {
            self.input.pressed_keys.push(127 as char);
        }
        if handle.is_key_pressed(KeyboardKey::KEY_ENTER) {
            self.input.pressed_keys.push('\n');
        }
        if handle.is_key_pressed(KeyboardKey::KEY_LEFT) {
            self.input.left_arrow_pressed = true;
        } else {
            self.input.left_arrow_pressed = false;
        }
        if handle.is_key_pressed(KeyboardKey::KEY_RIGHT) {
            self.input.right_arrow_pressed = true;
        } else {
            self.input.right_arrow_pressed = false;
        }
        let rat = self.dos.w as f32 / 640.0;
        self.input.mouse_x = (handle.get_mouse_x() as f32 / rat) as i32;
        self.input.mouse_y = (handle.get_mouse_y() as f32 / rat) as i32;
        let delt = handle.get_mouse_delta();
        self.input.mouse_dx = delt.x / rat;
        self.input.mouse_dy = delt.y / rat;
        self.input.scroll_amount = handle.get_mouse_wheel_move() as i32;
        self.cmd_pipeline
            .send(DrawCall::Update {
                input: self.input.clone(),
            })
            .unwrap();
        let mut drw = handle.begin_drawing(thread);
        drw.clear_background(Color::BLACK);
        self.dos.draw(&mut drw, thread);
        //  drw.draw_fps(100, 100);
    }

    pub fn run_draw_call<T>(
        map: &mut HashMap<String, Texture2D>,
        draw: &mut RaylibTextureMode<'_, T>,
        it: &mut dyn Iterator<Item = DrawCall>,
        to_load: &mut Vec<String>,
    ) -> Option<()> {
        let i = it.next()?;
        match i {
            DrawCall::ClearBackground { color } => {
                draw.clear_background(color.as_rl_color());
            }
            DrawCall::Rectangle {
                x,
                y,
                w,
                h,
                color,
                drop_shadow,
                outline,
            } => {
                if outline {
                    draw.draw_rectangle(x - 1, y - 1, w + 2, h + 2, Color::DARKGRAY);
                }
                if drop_shadow {
                    draw.draw_rectangle(x + 1, y + 1, w, h, Color::DARKGRAY);
                }
                draw.draw_rectangle(x, y, w, h, color.as_rl_color());
            }
            DrawCall::DrawPixels { points, width } => {
                for i in 0..points.len() - 1 {
                    let v1 = points[i].0;
                    let v2 = points[i + 1].0;
                    let col = points[i].1;
                    draw.draw_line_ex(
                        Vector2::new(v1.x as f32, v1.y as f32),
                        Vector2::new(v2.x as f32, v2.y as f32),
                        width,
                        col.as_rl_color(),
                    );
                }
            }
            DrawCall::DrawText {
                x,
                y,
                size,
                contents,
                color,
            } => {
                draw.draw_text(&contents, x, y, size, color.as_rl_color());
            }
            DrawCall::DrawSprite {
                x,
                y,
                h,
                w,
                contents,
            } => {
                let Some(tex) = map.get(&contents.name) else {
                    to_load.push(contents.name.clone());
                    return Some(());
                };
                draw.draw_texture_pro(
                    tex,
                    Rectangle {
                        x: 0.0,
                        y: 0.0,
                        width: tex.width() as f32,
                        height: tex.height() as f32,
                    },
                    Rectangle {
                        x: x as f32,
                        y: y as f32,
                        width: w as f32,
                        height: h as f32,
                    },
                    Vector2::zero(),
                    0.0,
                    Color::WHITE,
                );
            }
            DrawCall::Circle {
                x,
                y,
                rad,
                color,
                drop_shadow,
                outline,
            } => {
                if outline {
                    draw.draw_circle(x, y, rad as f32 + 2., Color::DARKGRAY);
                }
                if drop_shadow {
                    draw.draw_circle(x + 1, y + 1, rad as f32 + 1., Color::BLACK);
                }
                draw.draw_circle(x, y, rad as f32, color.as_rl_color());
            }
            DrawCall::Scissor { x, y, h, w } => {
                let mut sz = draw.begin_scissor_mode(x, y, w, h);
                loop {
                    if Self::run_draw_call(map, &mut sz, it, to_load).is_none() {
                        break;
                    }
                }
            }
            DrawCall::EndScissor => {
                return None;
            }
            _ => {}
        }
        Some(())
    }
    pub fn render(&mut self, handle: &mut RaylibHandle, _thread: &RaylibThread) {
        self.recieving_frame = false;
        self.should_draw = false;
        self.cmd_pipeline
            .send(DrawCall::Update {
                input: self.input.clone(),
            })
            .unwrap();
        self.input.pressed_keys.clear();
        let mut draw =
            handle.begin_texture_mode(_thread, self.dos.render_texture.as_mut().unwrap());
        let mut to_load = Vec::new();
        let mut drain = self.frame.drain(0..self.frame.len());
        loop {
            if Self::run_draw_call(
                &mut self.dos.loaded_textures,
                &mut draw,
                &mut drain,
                &mut to_load,
            )
            .is_none()
            {
                break;
            }
        }
        drop(draw);
        for i in to_load {
            let Ok(x) = handle.load_texture(_thread, &i) else {
                continue;
            };
            self.dos.loaded_textures.insert(i, x);
        }
    }
}

pub fn setup(fn_main: impl FnOnce(SysHandle) + Send + 'static) {
    let (cmd1, cmd2) = BPipe::create();
    let inp = UserInput {
        pressed_keys: Vec::new(),
        mouse_dx: 0.,
        mouse_dy: 0.,
        mouse_x: 0,
        mouse_y: 0,
        scroll_amount: 0,
        left_mouse_down: false,
        right_mouse_down: false,
        left_mouse_pressed: false,
        right_mouse_pressed: false,
        left_mouse_released: false,
        right_mouse_released: false,
        left_arrow_pressed: false,
        right_arrow_pressed: false,
    };

    let (mut handle, thread) = raylib::init().title("bridget").build();

    let h = raylib::window::get_monitor_height(0) - 100;
    println!("{}", h);
    let w = (4 * h) / 3;
    handle.set_window_size(w, h);
    handle.set_window_position(50, 50);
    let text = handle.load_render_texture(&thread, 640, 480).unwrap();
    let mut rt = DosRt {
        dos: Dos {
            loaded_textures: HashMap::new(),
            image: Image::gen_image_color(640, 480, Color::BLACK),
            render_texture: Some(text),
            h,
            w,
        },
        cmd_pipeline: cmd1,
        frame: Vec::new(),
        recieving_frame: false,
        should_draw: false,
        should_exit: false,
        input: inp.clone(),
    };

    let (max, ratios) = text_ratios(&handle);
    let sys = SysHandle {
        should_exit: false,
        handle: cmd2,
        cx: 0,
        cy: 0,
        w: 640,
        h: 480,
        padding_x: 4,
        padding_y: 4,
        ui_mode: SysUiMode::Sequential,
        text_ratios: ratios,
        max_ratio: max,
        div_stack: Vec::new(),
        queue: Vec::new(),
        background_color: BColor {
            r: 32,
            g: 32,
            b: 32,
            a: 255,
        },
        object_color: BColor {
            r: 64,
            g: 64,
            b: 64,
            a: 255,
        },
        object_pressed_color: BColor {
            r: 48,
            g: 48,
            b: 48,
            a: 255,
        },
        text_color: BColor {
            r: 192,
            g: 192,
            b: 192,
            a: 255,
        },
        user_input: inp,
        shadows: false,
        outline: true,
        text_box_data: HashMap::new(),
        scroll_box_data: HashMap::new(),
    };
    let tj = std::thread::spawn(move || fn_main(sys));
    rt.run_loop(handle, thread);
    tj.join().unwrap();
}
