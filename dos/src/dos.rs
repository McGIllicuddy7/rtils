use raylib::prelude::*;
pub use serde::{Deserialize, Serialize};
pub use std::sync::Arc;
use std::{collections::BTreeMap, error::Error};

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
    pub width: i32,
    pub height: i32,
    pub data: Arc<[BColor]>,
}
impl Sprite {
    pub fn sample(&self, x: f64, y: f64, w: f64, h: f64) -> BColor {
        let sw = self.width as f64;
        let sh = self.height as f64;
        let xrt = w / sw;
        let yrt = h / sh;
        let xp = x * xrt;
        let yp = y * yrt;
        let xact = ((xp.round()) as i32).clamp(0, self.width - 1);
        let yact = ((yp.round()) as i32).clamp(0, self.height - 1);
        self.data[xact as usize + yact as usize * self.width as usize]
    }

    pub fn sample_rl(&self, x: f64, y: f64, w: f64, h: f64) -> Color {
        let sw = self.width as f64;
        let sh = self.height as f64;
        let xrt = w / sw;
        let yrt = h / sh;
        let xp = x * xrt;
        let yp = y * yrt;
        let xact = ((xp.round()) as i32).clamp(0, self.width - 1);
        let yact = ((yp.round()) as i32).clamp(0, self.height - 1);
        self.data[xact as usize + yact as usize * self.width as usize].as_rl_color()
    }

    pub fn load_from_file(name: &str) -> Result<Self, Box<dyn Error>> {
        let mut img = Image::load_image(name)?;
        let h = img.height();
        let w = img.width();
        let mut v = Vec::new();
        v.reserve_exact((h * w) as usize);
        for y in 0..h {
            for x in 0..w {
                v.push(BColor::from_rl_color(img.get_color(x, y)));
            }
        }
        Ok(Self {
            width: w,
            height: h,
            data: v.into(),
        })
    }

    pub fn from_image(img: &mut Image) -> Self {
        let h = img.height();
        let w = img.width();
        let mut v = Vec::new();
        v.reserve_exact((h * w) as usize);
        for y in 0..h {
            for x in 0..w {
                v.push(BColor::from_rl_color(img.get_color(x, y)));
            }
        }
        Self {
            width: w,
            height: h,
            data: v.into(),
        }
    }

    pub fn to_image(&self) -> Image {
        let mut out = Image::gen_image_color(self.width, self.height, Color::WHITE);
        for y in 0..self.height {
            for x in 0..self.width {
                let c = self.data[(y * self.width + x) as usize];
                out.draw_pixel(x, y, c.as_rl_color());
            }
        }
        out
    }

    pub fn save_to_file(&self, name: &str) {
        let img = self.to_image();
        img.export_image(name);
    }

    pub fn get(&self, x: i32, y: i32) -> BColor {
        self.data[(y * self.width + x) as usize]
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

#[derive(Serialize, Deserialize, Clone)]
pub struct UserInput {
    pressed_keys: Vec<char>,
    mouse_x: i32,
    mouse_y: i32,
    left_mouse_down: bool,
    right_mouse_down: bool,
    left_mouse_pressed: bool,
    right_mouse_pressed: bool,
    left_mouse_released: bool,
    right_mouse_released: bool,
}
pub struct SysHandle {
    handle: BPipe<DrawCall>,
    cx: i32,
    cy: i32,
    w: i32,
    h: i32,
    ui_mode: SysUiMode,
    text_ratios: BTreeMap<char, f64>,
    max_ratio: f64,
    div_stack: Vec<Div>,
    queue: Vec<DrawCall>,
    text_color: BColor,
    background_color: BColor,
    object_color: BColor,
    shadows: bool,
    outline: bool,
    user_input: UserInput,
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
            let xs = mes as f64 / 10.0;
            if xs > max {
                max = xs;
            }
            out.insert(c, xs);
            text.clear();
        }
    }
    (max, out)
}
impl SysHandle {
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
                let mut txt = [0; 8];
                let utf = i.encode_utf8(&mut txt);
                let wp = self.measure_text(utf, h).unwrap();
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
                count += 1;
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
                vertical: true,
                mode: SysUiMode::Absolute,
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
            self.cy = prev_bounds.y + prev_bounds.h;
        } else {
            self.cy = div.y;
            self.cx = prev_bounds.x + prev_bounds.w;
        }
    }

    pub fn draw_text(&mut self, x: i32, y: i32, h: i32, max_w: i32, text: &str) {
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

    pub fn draw_box(&mut self, x: i32, y: i32, w: i32, h: i32) {
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

    pub fn draw_circle(&mut self, x: i32, y: i32, rad: i32) {
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

    pub fn draw_sprite(&mut self, x: i32, y: i32, w: i32, h: i32, sprite: Sprite) {
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
    pub fn draw_pixels(&mut self, points: Vec<(Pos2, BColor)>) {
        if points.len() == 0 {
            return;
        }
        let mut mpoints = points;
        mpoints.iter_mut().for_each(|(x, c)| {
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
        self.queue.push(DrawCall::DrawPixels { points: mpoints });
        let w = max.x - min.x;
        let h = max.y - min.y;
        self.update_cursor(Rect {
            x: min.x,
            y: min.y,
            w,
            h,
        });
    }

    pub fn draw_button(&mut self, x: i32, y: i32, w: i32, text_height: i32, text: &str) -> bool {
        let (mut h, texts) = self.text_get_height_and_lines(text, text_height, w - 5);
        h += 10;
        let pos = self.get_absolute_pos(Pos2 { x: x, y: y });
        self.queue.push(DrawCall::Rectangle {
            x: pos.x,
            y: pos.y,
            w,
            h,
            color: self.object_color,
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

    pub fn begin_div(&mut self, x: i32, y: i32, w: i32, h: i32, vertical: bool, mode: SysUiMode) {
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
        while let Ok(Some(x)) = self.handle.recieve() {
            match x {
                DrawCall::Update { input } => {
                    self.user_input = input;
                }
                DrawCall::Exiting => {
                    todo!()
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
        self.ui_mode = SysUiMode::Absolute;
    }

    pub fn end_drawing(&mut self) {
        self.queue.push(DrawCall::EndDrawing);
        for i in self.queue.drain(0..self.queue.len()) {
            self.handle.send(i).unwrap();
        }
        self.queue.clear();
        self.div_stack.clear();
        self.queue.clear();
        self.cx = 0;
        self.cy = 0;
        self.ui_mode = SysUiMode::Absolute;
    }
}

pub struct Dos {
    pub image: Image,
    pub render_texture: Option<RenderTexture2D>,
}

impl Dos {
    pub fn draw(&mut self, handle: &mut RaylibDrawHandle, thread: &RaylibThread) {
        let mut h = handle.begin_texture_mode(thread, self.render_texture.as_mut().unwrap());
        for y in 0..self.image.height() {
            for x in 0..self.image.width() {
                h.draw_pixel(x, y, self.image.get_color(x, y));
            }
        }
        drop(h);
        handle.draw_texture_rec(
            self.render_texture.as_ref().unwrap(),
            Rectangle::new(
                0.0,
                0.0,
                handle.get_render_width() as f32,
                handle.get_render_height() as f32,
            ),
            Vector2::new(0.0, 0.0),
            Color::WHITE,
        );
    }

    pub fn new() -> Self {
        Self {
            image: Image::gen_image_color(640, 480, Color::WHITE),
            render_texture: None,
        }
    }

    pub fn setup(&mut self, handle: &mut RaylibHandle, thread: &RaylibThread) {
        self.render_texture = Some(handle.load_render_texture(thread, 640, 480).unwrap());
    }

    pub fn draw_sprite(&mut self, sprite: &Sprite, x: i32, y: i32, w: i32, h: i32) {
        for yp in 0..h {
            for xp in 0..w {
                let c = sprite.sample_rl(xp as f64, yp as f64, w as f64, h as f64);
                self.image.draw_pixel(xp + x, yp + y, c);
            }
        }
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
                self.render(&mut handle, &thread);
            }
            self.draw(&mut handle, &thread);
        }
    }

    pub fn draw(&mut self, handle: &mut RaylibHandle, thread: &RaylibThread) {
        if let Some(key) = handle.get_char_pressed() {
            self.input.pressed_keys.push(key);
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
            self.input.mouse_x = handle.get_mouse_x();
        }
        self.input.mouse_y = handle.get_mouse_y();
        let mut drw = handle.begin_drawing(thread);
        drw.clear_background(Color::BLACK);
        self.dos.draw(&mut drw, thread);
    }

    pub fn render(&mut self, handle: &mut RaylibHandle, thread: &RaylibThread) {
        self.recieving_frame = false;
        self.should_draw = false;
        self.cmd_pipeline
            .send(DrawCall::Update {
                input: self.input.clone(),
            })
            .unwrap();
        self.input.pressed_keys.clear();
        for i in self.frame.drain(0..self.frame.len()) {
            match i {
                DrawCall::ClearBackground { color } => {
                    self.dos.image.clear_background(color.as_rl_color());
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
                        self.dos
                            .image
                            .draw_rectangle(x - 1, y - 1, w + 2, h + 2, Color::DARKGRAY);
                    }
                    if drop_shadow {
                        self.dos
                            .image
                            .draw_rectangle(x + 1, y + 1, w, h, Color::DARKGRAY);
                    }
                    self.dos
                        .image
                        .draw_rectangle(x, y, w, h, color.as_rl_color());
                }
                DrawCall::DrawPixels { points } => {
                    for (p, c) in points {
                        self.dos.image.draw_pixel(p.x, p.y, c.as_rl_color());
                    }
                }
                DrawCall::DrawText {
                    x,
                    y,
                    size,
                    contents,
                    color,
                } => {
                    self.dos
                        .image
                        .draw_text(&contents, x, y, size, color.as_rl_color());
                }
                DrawCall::DrawSprite {
                    x,
                    y,
                    h,
                    w,
                    contents,
                } => {
                    self.dos.draw_sprite(&contents, x, y, w, h);
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
                        self.dos.image.draw_circle(x, y, rad + 2, Color::DARKGRAY);
                    }
                    if drop_shadow {
                        self.dos
                            .image
                            .draw_circle(x + 1, y + 1, rad + 1, Color::BLACK);
                    }
                    self.dos.image.draw_circle(x, y, rad, color.as_rl_color());
                }
                _ => {}
            }
        }
    }
}
