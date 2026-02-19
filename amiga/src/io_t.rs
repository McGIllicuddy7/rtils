use std::{net::TcpStream, sync::Arc};

use raylib::{
    color::Color,
    ffi::{KeyboardKey, MouseButton},
    math::{Rectangle, Vector2},
};
use serde::{Deserialize, Serialize};

use crate::{
    BStream, HANDLE, Handle, HandleUpdate, Ptr,
    device::{TEXT_OFFSET, TEXT_SCALE},
    get_handle,
    input::Input,
    malloc,
};

#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub struct Sprite {
    pub colors: Arc<[[Col; 16]; 16]>,
}
#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub struct LargeSprite {
    pub colors: Arc<[[Col; 32]; 32]>,
}
#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub struct Texture {
    pub data: Arc<[Col]>,
    pub w: i32,
    pub h: i32,
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum Cmd {
    DrawPixel {
        x: i32,
        y: i32,
        color: Col,
    },
    Update(HandleUpdate),
    DrawCharacter {
        x: i32,
        y: i32,
        color: Col,
        c: char,
    },
    DrawRect {
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        color: Col,
    },
    DrawBox {
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        color: Col,
    },
    DrawCircle {
        x: i32,
        y: i32,
        rx10: i32,
        color: Col,
    },
    DrawString {
        text: String,
        x: i32,
        y: i32,
        w: i32,
        color: Col,
        wrap: bool,
    },
    DrawSprite {
        x: i32,
        y: i32,
        h: i32,
        w: i32,
        sprite: Sprite,
    },
    DrawLargeSprite {
        x: i32,
        y: i32,
        h: i32,
        w: i32,
        sprite: LargeSprite,
    },
    DrawTexture {
        x: i32,
        y: i32,
        h: i32,
        w: i32,
        texture: Texture,
    },
    DrawLine {
        start_x: i32,
        end_x: i32,
        start_y: i32,
        end_y: i32,
        color: Col,
    },
    BeginScissor {
        x: i32,
        y: i32,
        w: i32,
        h: i32,
    },
    EndScissor {
        x: i32,
        y: i32,
        w: i32,
        h: i32,
    },
    BeginDrawing,
    EndDrawing,
}
#[repr(i8)]
#[derive(PartialEq, Eq, Clone, Debug, Copy, Serialize, Deserialize)]
pub enum Col {
    Empty,
    Black,
    White,
    Blue,
    Red,
    Green,
    Yellow,
    Violet,
    DarkBlue,
    DarkRed,
    DarkGreen,
    DarkYellow,
    DarkViolet,
    Gray,
    DarkGray,
    LightBlack,
    Cyan,
    DarkCyan,
    Orchid,
    DarkOrchid,
    ColorCount,
}
impl Col {
    pub fn from_index(i: i8) -> Self {
        match i {
            0 => Self::Empty,
            1 => Self::Black,
            2 => Self::White,
            3 => Self::Blue,
            4 => Self::Red,
            5 => Self::Green,
            6 => Self::Yellow,
            7 => Self::Violet,
            8 => Self::DarkBlue,
            9 => Self::DarkRed,
            10 => Self::DarkGreen,
            11 => Self::DarkYellow,
            12 => Self::DarkViolet,
            13 => Self::Gray,
            14 => Self::DarkGray,
            15 => Self::LightBlack,
            16 => Self::Cyan,
            17 => Self::DarkCyan,
            18 => Self::Orchid,
            19 => Self::DarkOrchid,
            _ => {
                todo!()
            }
        }
    }
    pub fn as_color(&self) -> Color {
        match self {
            Col::Empty => Color::new(0, 0, 0, 0),
            Col::Black => Color::BLACK,
            Col::White => Color::WHITE,
            Col::Blue => Color::BLUE,
            Col::Red => Color::RED,
            Col::Green => Color::GREEN,
            Col::Yellow => Color::YELLOW,
            Col::Violet => Color::VIOLET,
            Col::DarkBlue => Color::DARKBLUE,
            Col::DarkRed => Color::DARKRED,
            Col::DarkGreen => Color::DARKGREEN,
            Col::DarkYellow => Color::GOLDENROD,
            Col::DarkViolet => Color::DARKVIOLET,
            Col::Gray => Color::LIGHTGRAY,
            Col::DarkGray => Color::GRAY,
            Col::LightBlack => Color::DARKGRAY,
            Col::Cyan => Color::CYAN,
            Col::DarkCyan => Color::DARKCYAN,
            Col::Orchid => Color::ORCHID,
            Col::DarkOrchid => Color::DARKORCHID,
            Col::ColorCount => todo!(),
        }
    }
    pub fn from_color(c: Color) -> Self {
        let mut min_c = Col::Empty;
        let mut min = 100000;
        if c.a < 128 {
            return Self::Empty;
        }
        for i in 0..Self::ColorCount as i8 {
            let c1 = Self::from_index(i).as_color();
            let delta = ((c.r as i64 - c1.r as i64) * (c.r as i64 - c1.r as i64)
                + (c.g as i64 - c1.g as i64) * (c.g as i64 - c1.g as i64)
                + (c.b as i64 - c1.b as i64) * (c.b as i64 - c1.b as i64))
                .isqrt();
            if delta < min {
                min_c = Self::from_index(i);
                min = delta;
            }
        }
        min_c
    }
}
pub fn setup() -> Option<BStream<Cmd>> {
    if let Ok(at) = TcpStream::connect("127.0.0.1:12000")
        && false
    {
        let mut hand = HANDLE.lock().unwrap();
        *hand = Some(Handle {
            input: Input::new(),
            queue: BStream::from_stream(at),
            should_close: false,
            pressed_key: None,
        });
        None
    } else {
        let (out, ix) = BStream::create();
        let mut hand = HANDLE.lock().unwrap();
        *hand = Some(Handle {
            input: Input::new(),
            queue: ix,
            should_close: false,
            pressed_key: None,
        });
        Some(out)
    }
}

pub fn measure_text_height(text: &str, width: i32) -> i32 {
    let mut dx = 0;
    let mut dy = TEXT_SCALE as i32;
    for _ in text.chars() {
        dx += TEXT_OFFSET;
        if dx > width {
            dx = 0;
            dy += TEXT_SCALE as i32;
        }
    }
    dy
}

pub fn get_mouse_x() -> i32 {
    get_handle().input.mouse_x
}

pub fn get_mouse_y() -> i32 {
    get_handle().input.mouse_y
}

pub fn get_mouse_pos() -> (i32, i32) {
    let h = get_handle();
    (h.input.mouse_x, h.input.mouse_y)
}

pub fn get_mouse_delta() -> Vector2 {
    let h = get_handle();
    Vector2::new(h.input.mouse_dx, h.input.mouse_dy)
}

pub fn is_key_down(key_code: KeyboardKey) -> bool {
    let h = get_handle();
    h.input.codes.get(&(key_code as i32)).unwrap().down
}

pub fn is_key_pressed(key_code: KeyboardKey) -> bool {
    let h = get_handle();
    h.input.codes.get(&(key_code as i32)).unwrap().pressed
}

pub fn is_key_released(key_code: KeyboardKey) -> bool {
    let h = get_handle();
    h.input.codes.get(&(key_code as i32)).unwrap().released
}

pub fn is_mouse_button_down(button: MouseButton) -> bool {
    let h = get_handle();
    h.input.mouse.get(&(button as i32)).unwrap().down
}

pub fn is_mouse_button_pressed(button: MouseButton) -> bool {
    let h = get_handle();
    h.input.mouse.get(&(button as i32)).unwrap().pressed
}

pub fn is_mouse_button_released(button: MouseButton) -> bool {
    let h = get_handle();
    h.input.mouse.get(&(button as i32)).unwrap().released
}

pub fn get_pressed_key() -> Option<char> {
    let h = get_handle();
    h.pressed_key
}

pub fn begin_drawing() {
    let mut h = get_handle();
    let Ok(x) = h.queue.receive_wait() else {
        h.should_close = true;
        return;
    };
    match x {
        Cmd::Update(x) => {
            h.input = x.input;
            h.pressed_key = x.pressed_key;
            h.should_close = x.should_close;
        }
        _ => {}
    }
    let _ = h.queue.send(Cmd::BeginDrawing);
}

pub fn end_drawing() {
    let h = get_handle();
    let _ = h.queue.send(Cmd::EndDrawing);
}

pub fn should_close() -> bool {
    let hd = get_handle();
    hd.should_close
}

pub fn draw_button(
    text: &str,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    color: Col,
    text_color: Col,
) -> bool {
    draw_box(x, y, w, h, color);
    draw_text(text, x + 5, y + 5, w - 10, text_color, true);
    let rect = Rectangle::new(x as f32, y as f32, w as f32, h as f32);
    let pos = get_mouse_pos();
    rect.check_collision_point_rec(Vector2::new(pos.0 as f32, pos.1 as f32))
        && is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT)
}

pub fn draw_pixel(x: i32, y: i32, color: Col) {
    let h = get_handle();
    let _ = h.queue.send(Cmd::DrawPixel { x, y, color });
}

pub fn draw_char(c: char, x: i32, y: i32, color: Col) {
    let h = get_handle();
    let _ = h.queue.send(Cmd::DrawCharacter { x, y, color, c });
}

pub fn draw_rect(x: i32, y: i32, w: i32, h: i32, color: Col) {
    let hd = get_handle();
    let _ = hd.queue.send(Cmd::DrawRect { x, y, w, h, color });
}

pub fn draw_box(x: i32, y: i32, w: i32, h: i32, color: Col) {
    let hd = get_handle();
    let _ = hd.queue.send(Cmd::DrawBox { x, y, w, h, color });
}

pub fn draw_circle(x: i32, y: i32, rx10: i32, color: Col) {
    let hd = get_handle();
    let _ = hd.queue.send(Cmd::DrawCircle { x, y, rx10, color });
}

pub fn begin_scissor(x: i32, y: i32, w: i32, h: i32) {
    let hd = get_handle();
    let cmd = Cmd::BeginScissor { x, y, w, h };
    let _ = hd.queue.send(cmd);
}

pub fn end_scissor(x: i32, y: i32, w: i32, h: i32) {
    let hd = get_handle();
    let cmd = Cmd::EndScissor { x, y, w, h };
    let _ = hd.queue.send(cmd);
}

pub fn draw_sprite(x: i32, y: i32, w: i32, h: i32, sprite: Sprite) {
    let hd = get_handle();
    let _ = hd.queue.send(Cmd::DrawSprite {
        x,
        y,
        h,
        w,
        sprite: sprite,
    });
}
pub fn draw_large_sprite(x: i32, y: i32, w: i32, h: i32, sprite: LargeSprite) {
    let hd = get_handle();
    let _ = hd.queue.send(Cmd::DrawLargeSprite {
        x,
        y,
        h,
        w,
        sprite: sprite,
    });
}
pub fn draw_texture(x: i32, y: i32, w: i32, h: i32, texture: Texture) {
    let hd = get_handle();
    let _ = hd.queue.send(Cmd::DrawTexture {
        x,
        y,
        h,
        w,
        texture,
    });
}

pub fn draw_text(text: &str, x: i32, y: i32, w: i32, color: Col, wrap: bool) {
    let hd = get_handle();
    let _ = hd.queue.send(Cmd::DrawString {
        text: text.to_string(),
        x,
        y,
        w,
        color,
        wrap,
    });
}

pub fn draw_line(x0: i32, x1: i32, y0: i32, y1: i32, color: Col) {
    let hd = get_handle();
    let _ = hd.queue.send(Cmd::DrawLine {
        start_x: x0,
        end_x: x1,
        start_y: y0,
        end_y: y1,
        color,
    });
}
pub enum BufferedCommand<'a> {
    DrawPixel {
        x: i32,
        y: i32,
        color: Col,
    },
    DrawCharacter {
        x: i32,
        y: i32,
        color: Col,
        c: char,
    },
    DrawRect {
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        color: Col,
    },
    DrawBox {
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        color: Col,
    },
    DrawCircle {
        x: i32,
        y: i32,
        rx10: i32,
        color: Col,
    },
    DrawString {
        text: String,
        x: i32,
        y: i32,
        w: i32,
        color: Col,
        wrap: bool,
    },
    DrawLine {
        start_x: i32,
        end_x: i32,
        start_y: i32,
        end_y: i32,
        color: Col,
    },
    BeginScissor {
        x: i32,
        y: i32,
        w: i32,
        h: i32,
    },
    EndScissor {
        x: i32,
        y: i32,
        w: i32,
        h: i32,
    },
    DrawButton {
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        on_click: &'a mut dyn FnMut(),
    },
    DrawSprite {
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        sprite: Sprite,
    },
    DrawLargeSprite {
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        sprite: LargeSprite,
    },
    DrawTexture {
        x: i32,
        y: i32,
        h: i32,
        w: i32,
        texture: Texture,
    },
}
impl<'a> BufferedCommand<'a> {
    pub fn bounds(&self) -> Rectangle {
        match self {
            BufferedCommand::DrawPixel { x, y, color: _ } => {
                Rectangle::new(*x as f32, *y as f32, 1.0, 1.0)
            }
            BufferedCommand::DrawCharacter {
                x,
                y,
                color: _,
                c: _,
            } => Rectangle::new(*x as f32, *y as f32, TEXT_OFFSET as f32, TEXT_SCALE),
            BufferedCommand::DrawRect {
                x,
                y,
                w,
                h,
                color: _,
            } => Rectangle::new(*x as f32, *y as f32, *w as f32, *h as f32),
            BufferedCommand::DrawBox {
                x,
                y,
                w,
                h,
                color: _,
            } => Rectangle::new(*x as f32, *y as f32, *w as f32, *h as f32),
            BufferedCommand::DrawCircle {
                x,
                y,
                rx10,
                color: _,
            } => Rectangle::new(*x as f32, *y as f32, *rx10 as f32 * 2., *rx10 as f32 * 2.),
            BufferedCommand::DrawString {
                text,
                x,
                y,
                w,
                color: _,
                wrap: _,
            } => Rectangle::new(
                *x as f32,
                *y as f32,
                *w as f32 * 2.,
                measure_text_height(&text, *w) as f32,
            ),
            BufferedCommand::DrawLine {
                start_x,
                end_x,
                start_y,
                end_y,
                color: _,
            } => {
                let x = if *start_x > *end_x { *end_x } else { *start_x };
                let y = if *start_y > *end_y { *end_y } else { *start_y };
                let w = start_x.abs_diff(*end_x) as i32;
                let h = start_y.abs_diff(*end_y) as i32;
                Rectangle::new(x as f32, y as f32, w as f32, h as f32)
            }
            BufferedCommand::DrawButton {
                x,
                y,
                w,
                h,
                on_click: _,
            } => Rectangle::new(*x as f32, *y as f32, *w as f32, *h as f32),
            BufferedCommand::BeginScissor { x, y, w, h } => {
                Rectangle::new(*x as f32, *y as f32, *w as f32, *h as f32)
            }
            BufferedCommand::EndScissor { x, y, w, h } => {
                Rectangle::new(*x as f32, *y as f32, *w as f32, *h as f32)
            }
            BufferedCommand::DrawSprite {
                x,
                y,
                w,
                h,
                sprite: _,
            } => Rectangle::new(*x as f32, *y as f32, *w as f32, *h as f32),
            BufferedCommand::DrawLargeSprite {
                x,
                y,
                w,
                h,
                sprite: _,
            } => Rectangle::new(*x as f32, *y as f32, *w as f32, *h as f32),
            BufferedCommand::DrawTexture {
                x,
                y,
                w,
                h,
                texture: _,
            } => Rectangle::new(*x as f32, *y as f32, *w as f32, *h as f32),
        }
    }

    pub fn min_max(&self) -> (Vector2, Vector2) {
        let rct = self.bounds();
        let v0 = Vector2::new(rct.x, rct.y);
        let v1 = Vector2::new(rct.width, rct.height) + v0;
        (v0, v1)
    }
    pub fn shift(&mut self, dx: i32, dy: i32) {
        match self {
            BufferedCommand::DrawPixel { x, y, color: _ } => {
                *x += dx;
                *y += dy;
            }
            BufferedCommand::DrawCharacter {
                x,
                y,
                color: _,
                c: _,
            } => {
                *x += dx;
                *y += dy;
            }
            BufferedCommand::DrawRect {
                x,
                y,
                w: _,
                h: _,
                color: _,
            } => {
                *x += dx;
                *y += dy;
            }
            BufferedCommand::DrawBox {
                x,
                y,
                w: _,
                h: _,
                color: _,
            } => {
                *x += dx;
                *y += dy;
            }
            BufferedCommand::DrawCircle {
                x,
                y,
                rx10: _,
                color: _,
            } => {
                *x += dx;
                *y += dy;
            }
            BufferedCommand::DrawString {
                text: _,
                x,
                y,
                w: _,
                color: _,
                wrap: _,
            } => {
                *x += dx;
                *y += dy;
            }
            BufferedCommand::DrawLine {
                start_x,
                end_x,
                start_y,
                end_y,
                color: _,
            } => {
                *start_x += dx;
                *start_y += dy;
                *end_x += dx;
                *end_y += dy;
            }
            BufferedCommand::DrawButton {
                x,
                y,
                w: _,
                h: _,
                on_click: _,
            } => {
                *x += dx;
                *y += dy;
            }
            BufferedCommand::BeginScissor { x, y, w: _, h: _ } => {
                *x += dx;
                *y += dy;
            }
            BufferedCommand::EndScissor { x, y, w: _, h: _ } => {
                *x += dx;
                *y += dy;
            }
            BufferedCommand::DrawSprite {
                x,
                y,
                w: _,
                h: _,
                sprite: _,
            } => {
                *x += dx;
                *y += dy;
            }
            BufferedCommand::DrawLargeSprite {
                x,
                y,
                w: _,
                h: _,
                sprite: _,
            } => {
                *x += dx;
                *y += dy;
            }
            BufferedCommand::DrawTexture {
                x,
                y,
                w: _,
                h: _,
                texture: _,
            } => {
                *x += dx;
                *y += dy;
            }
        }
    }
}
pub struct CommandBuffer<'a> {
    commands: Vec<BufferedCommand<'a>>,
}

pub fn end_command_buffer(buffer: CommandBuffer<'_>) {
    let hd = get_handle();
    for i in buffer.commands {
        match i {
            BufferedCommand::DrawPixel { x, y, color } => {
                let cmd = Cmd::DrawPixel { x, y, color };
                let _ = hd.queue.send(cmd);
            }
            BufferedCommand::DrawCharacter { x, y, color, c } => {
                let cmd = Cmd::DrawCharacter { x, y, color, c };
                let _ = hd.queue.send(cmd);
            }
            BufferedCommand::DrawRect { x, y, w, h, color } => {
                let cmd = Cmd::DrawRect { x, y, w, h, color };
                let _ = hd.queue.send(cmd);
            }
            BufferedCommand::DrawBox { x, y, w, h, color } => {
                let cmd = Cmd::DrawBox { x, y, w, h, color };
                let _ = hd.queue.send(cmd);
            }
            BufferedCommand::DrawCircle { x, y, rx10, color } => {
                let cmd = Cmd::DrawCircle { x, y, rx10, color };
                let _ = hd.queue.send(cmd);
            }
            BufferedCommand::DrawString {
                text,
                x,
                y,
                w,
                color,
                wrap,
            } => {
                let cmd = Cmd::DrawString {
                    text,
                    x,
                    y,
                    w,
                    color,
                    wrap,
                };
                let _ = hd.queue.send(cmd);
            }
            BufferedCommand::DrawLine {
                start_x,
                end_x,
                start_y,
                end_y,
                color,
            } => {
                let cmd = Cmd::DrawLine {
                    start_x,
                    end_x,
                    start_y,
                    end_y,
                    color,
                };
                let _ = hd.queue.send(cmd);
            }
            BufferedCommand::DrawButton {
                x,
                y,
                w,
                h,
                on_click,
            } => {
                let bx = Rectangle::new(x as f32, y as f32, w as f32, h as f32);
                if bx.check_collision_point_rec(Vector2::new(
                    get_mouse_x() as f32,
                    get_mouse_y() as f32,
                )) && is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT)
                {
                    on_click();
                }
            }
            BufferedCommand::BeginScissor { x, y, w, h } => {
                let cmd = Cmd::BeginScissor { x, y, w, h };
                let _ = hd.queue.send(cmd);
            }
            BufferedCommand::EndScissor { x, y, w, h } => {
                let cmd = Cmd::EndScissor { x, y, w, h };
                let _ = hd.queue.send(cmd);
            }
            BufferedCommand::DrawSprite { x, y, w, h, sprite } => {
                let cmd = Cmd::DrawSprite { x, y, w, h, sprite };
                let _ = hd.queue.send(cmd);
            }
            BufferedCommand::DrawLargeSprite { x, y, w, h, sprite } => {
                let cmd = Cmd::DrawLargeSprite { x, y, w, h, sprite };
                let _ = hd.queue.send(cmd);
            }
            BufferedCommand::DrawTexture {
                x,
                y,
                w,
                h,
                texture,
            } => {
                let cmd = Cmd::DrawTexture {
                    x,
                    y,
                    w,
                    h,
                    texture,
                };
                let _ = hd.queue.send(cmd);
            }
        }
    }
}

pub fn begin_command_buffer<'a>() -> CommandBuffer<'a> {
    CommandBuffer {
        commands: Vec::new(),
    }
}

pub fn queue_draw_pixel(buffer: &mut CommandBuffer<'_>, x: i32, y: i32, color: Col) {
    buffer
        .commands
        .push(BufferedCommand::DrawPixel { x, y, color });
}

pub fn queue_draw_char(buffer: &mut CommandBuffer<'_>, c: char, x: i32, y: i32, color: Col) {
    buffer
        .commands
        .push(BufferedCommand::DrawCharacter { x, y, color, c });
}

pub fn queue_draw_rect(buffer: &mut CommandBuffer<'_>, x: i32, y: i32, w: i32, h: i32, color: Col) {
    buffer
        .commands
        .push(BufferedCommand::DrawRect { x, y, w, h, color });
}

pub fn queue_draw_box(buffer: &mut CommandBuffer<'_>, x: i32, y: i32, w: i32, h: i32, color: Col) {
    buffer
        .commands
        .push(BufferedCommand::DrawBox { x, y, w, h, color });
}

pub fn queue_begin_scissor(buffer: &mut CommandBuffer<'_>, x: i32, y: i32, w: i32, h: i32) {
    buffer
        .commands
        .push(BufferedCommand::BeginScissor { x, y, w, h });
}
pub fn queue_end_scissor(buffer: &mut CommandBuffer<'_>, x: i32, y: i32, w: i32, h: i32) {
    buffer
        .commands
        .push(BufferedCommand::EndScissor { x, y, w, h });
}
pub fn queue_draw_circle(buffer: &mut CommandBuffer<'_>, x: i32, y: i32, rx10: i32, color: Col) {
    buffer
        .commands
        .push(BufferedCommand::DrawCircle { x, y, rx10, color });
}

pub fn queue_draw_text(
    buffer: &mut CommandBuffer<'_>,
    text: &str,
    x: i32,
    y: i32,
    w: i32,
    color: Col,
    wrap: bool,
) {
    buffer.commands.push(BufferedCommand::DrawString {
        text: text.to_string(),
        x,
        y,
        w,
        color,
        wrap,
    });
}

pub fn queue_draw_line(
    buffer: &mut CommandBuffer<'_>,
    x0: i32,
    x1: i32,
    y0: i32,
    y1: i32,
    color: Col,
) {
    buffer.commands.push(BufferedCommand::DrawLine {
        start_x: x0,
        end_x: x1,
        start_y: y0,
        end_y: y1,
        color,
    });
}

pub fn queue_draw_button<'a>(
    buffer: &mut CommandBuffer<'a>,
    text: &str,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    color: Col,
    text_color: Col,
    on_click: &'a mut dyn FnMut(),
) {
    queue_draw_box(buffer, x, y, w, h, color);
    queue_draw_text(buffer, text, x, y, w, text_color, true);
    let cmd = BufferedCommand::DrawButton {
        x,
        y,
        w,
        h,
        on_click,
    };
    buffer.commands.push(cmd);
}

pub fn queue_draw_sprite<'a>(
    buffer: &mut CommandBuffer<'a>,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    sprite: Sprite,
) {
    buffer.commands.push(BufferedCommand::DrawSprite {
        x,
        y,
        h,
        w,
        sprite: sprite,
    })
}
pub fn queue_draw_large_sprite<'a>(
    buffer: &mut CommandBuffer<'a>,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    sprite: LargeSprite,
) {
    buffer.commands.push(BufferedCommand::DrawLargeSprite {
        x,
        y,
        h,
        w,
        sprite: sprite,
    })
}

pub fn queue_draw_texture<'a>(
    buffer: &mut CommandBuffer<'a>,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    texture: Texture,
) {
    buffer.commands.push(BufferedCommand::DrawTexture {
        x,
        y,
        h,
        w,
        texture,
    })
}

pub fn queue_draw_queue<'a>(buffer: &mut CommandBuffer<'a>, cmds: CommandBuffer<'a>) {
    for i in cmds.commands {
        buffer.commands.push(i);
    }
}

pub fn buffer_bounds<'a>(buffer: &'a CommandBuffer) -> Rectangle {
    if buffer.commands.is_empty() {
        return Rectangle::new(0.0, 0.0, 0.0, 0.0);
    }
    let (mut min, mut max) = buffer.commands[0].min_max();
    for i in &buffer.commands {
        let (mn, mx) = i.min_max();
        if mn.x < min.x {
            min.x = mn.x;
        }
        if mn.y < min.y {
            min.y = mn.y;
        }
        if mx.x > max.x {
            max.x = mx.x;
        }
        if mx.y > max.y {
            max.y = mx.y;
        }
    }
    let delta = max - min;
    Rectangle::new(min.x, min.y, delta.x, delta.y)
}

pub fn buffer_shift<'a>(buffer: &mut CommandBuffer<'a>, shift_x: i32, shift_y: i32) {
    for i in &mut buffer.commands {
        i.shift(shift_x, shift_y);
    }
}

pub fn buffer_purge_outside<'a>(buffer: CommandBuffer<'a>, rect: Rectangle) -> CommandBuffer<'a> {
    let cmds = buffer
        .commands
        .into_iter()
        .filter(|i| i.bounds().check_collision_recs(&rect))
        .collect();
    CommandBuffer { commands: cmds }
}

//zero in this is the top left of the scrollbox
pub fn draw_scroll_box<'a>(
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    scroll_amount: f32,
    func: impl FnOnce(&mut CommandBuffer<'a>, i32, i32),
) -> f32 {
    let mut cmds = begin_command_buffer();
    draw_box(x, y, w, h, Col::Violet);
    draw_box(
        x + w - 5,
        y + ((h - 10) as f32 * scroll_amount) as i32,
        5,
        10,
        Col::Violet,
    );
    begin_scissor(x, y, w, h);
    func(&mut cmds, w, h);
    buffer_shift(&mut cmds, x, y);
    let bounds = buffer_bounds(&cmds);
    buffer_shift(
        &mut cmds,
        0,
        ((-bounds.height as i32 + h) as f32 * scroll_amount) as i32,
    );
    cmds = buffer_purge_outside(cmds, Rectangle::new(x as f32, y as f32, w as f32, h as f32));
    end_command_buffer(cmds);
    end_scissor(x - 1, y - 1, w + 2, h + 2);
    let mut amnt = scroll_amount;
    amnt -= get_handle().input.scroll_amount * 1. / (bounds.height - h as f32).abs();
    if amnt < 0.0 {
        amnt = 0.0;
    } else if amnt > 1.0 {
        amnt = 1.0;
    }
    amnt
}

//zero in this is the bottom left of the scrollbox
pub fn draw_scroll_box_rev<'a>(
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    scroll_amount: f32,
    func: impl FnOnce(&mut CommandBuffer<'a>, i32, i32),
) -> f32 {
    let mut cmds = begin_command_buffer();
    draw_box(x, y, w, h, Col::Violet);
    draw_box(
        x + w - 5,
        y + ((h - 10) as f32 * scroll_amount) as i32,
        5,
        10,
        Col::Violet,
    );
    begin_scissor(x, y, w, h);
    func(&mut cmds, w, h);
    buffer_shift(&mut cmds, x, y);
    let bounds = buffer_bounds(&cmds);
    let dh = if cmds.commands.is_empty() {
        0
    } else {
        cmds.commands[0].bounds().height as i32
    };

    buffer_shift(
        &mut cmds,
        0,
        (-(bounds.height - (h * 2 - dh) as f32) * scroll_amount) as i32 + h,
    );
    cmds = buffer_purge_outside(cmds, Rectangle::new(x as f32, y as f32, w as f32, h as f32));
    end_command_buffer(cmds);
    end_scissor(x, y, w, h);
    let mut amnt = scroll_amount;
    amnt -= get_handle().input.scroll_amount * 1. / (bounds.height - h as f32).abs();
    if amnt < 0.0 {
        amnt = 0.0;
    } else if amnt > 1.0 {
        amnt = 1.0;
    }
    amnt
}

pub fn load_sprite(path: &str) -> Option<Sprite> {
    let mut colors = [[Col::Empty; _]; _];
    let Ok(mut img) = raylib::prelude::Image::load_image(path) else {
        return None;
    };
    img.resize_nn(colors[0].len() as i32, colors.len() as i32);
    for i in 0..colors.len() {
        for j in 0..colors[0].len() {
            colors[i][j] = Col::from_color(img.get_color(j as i32, i as i32));
        }
    }
    Some(Sprite {
        colors: Arc::new(colors),
    })
}

pub fn load_large_sprite(path: &str) -> Option<LargeSprite> {
    let mut colors = [[Col::Empty; _]; _];
    let Ok(mut img) = raylib::prelude::Image::load_image(path) else {
        return None;
    };
    img.resize_nn(colors[0].len() as i32, colors.len() as i32);
    for i in 0..colors.len() {
        for j in 0..colors[0].len() {
            colors[i][j] = Col::from_color(img.get_color(j as i32, i as i32));
        }
    }
    Some(LargeSprite {
        colors: Arc::new(colors),
    })
}

pub fn load_texture(path: &str) -> Option<Texture> {
    let Ok(mut img) = raylib::prelude::Image::load_image(path) else {
        return None;
    };
    let w = img.width();
    let h = img.height();
    img.resize_nn(w / 2, h / 2);
    let w = img.width();
    let h = img.height();
    let mut data = vec![Col::Empty; (img.width() * img.height()) as usize];

    // img.resize_nn(out.w as i32, outh as i32);
    for i in 0..h {
        for j in 0..w {
            data[(i * w + j) as usize] = Col::from_color(img.get_color(j as i32, i as i32));
        }
    }
    Some(Texture {
        data: data.into(),
        w,
        h,
    })
}
