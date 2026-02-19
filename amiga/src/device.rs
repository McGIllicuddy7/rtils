pub const TEXT_SCALE: f32 = 10.0;
pub const TEXT_OFFSET: i32 = 10;
pub const SCREEN_WIDTH: i32 = 640;
pub const SCREEN_HEIGHT: i32 = 480;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::mem::zeroed;
use std::net::TcpListener;
use std::sync::Arc;

use crate::BStream;
use crate::Col;
use crate::HandleUpdate;
use crate::LargeSprite;
use crate::Sprite;
use crate::Texture;
use crate::input;
use crate::io_t::Cmd;
use crate::rtils::rtils_useful::BPipe;
use raylib::RaylibHandle;
use raylib::RaylibThread;
use raylib::ffi::KeyboardKey;
use raylib::math::Vector2;
use raylib::prelude::RaylibScissorModeExt;
use raylib::prelude::RaylibShaderModeExt;
use raylib::prelude::RaylibTextureMode;
use raylib::shaders::RaylibShader;
use raylib::text::Font;
use raylib::texture::RenderTexture2D;
use raylib::{
    color::Color,
    math::Rectangle,
    prelude::{RaylibDraw, RaylibTextureModeExt},
    texture::RaylibRenderTexture2D,
};
pub enum ProcessMode {
    Draw,
    Terminal,
}
pub struct Process {
    pub handle: BStream<Cmd>,
    pub x: i32,
    pub y: i32,
    pub scale: f32,
    pub cmds: Vec<Cmd>,
    pub target: RenderTexture2D,
}

pub struct Device {
    pub handle: RaylibHandle,
    pub thread: RaylibThread,
    pub processes: BTreeMap<u64, Process>,
    pub font: Font,
    pub text_scale: f32,
    pub text_offset: i32,
    pub text_height: i32,
    pub incoming_processes: BPipe<BStream<Cmd>>,
    pub active_process: u64,
}

#[derive(Clone)]
pub enum CachedTexture {
    Sprite(Arc<[[Col; 16]; 16]>),
    LargeSprite(Arc<[[Col; 32]; 32]>),
    Texture(Arc<[Col]>),
}
impl std::hash::Hash for CachedTexture {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            CachedTexture::Sprite(x) => {
                x.as_ptr().hash(state);
            }
            CachedTexture::LargeSprite(x) => {
                x.as_ptr().hash(state);
            }
            CachedTexture::Texture(x) => {
                x.as_ptr().hash(state);
            }
        }
    }
}
impl PartialEq for CachedTexture {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Sprite(l0), Self::Sprite(r0)) => {
                /*   println!(
                    "{:#?}, {:#?}, eq:{}",
                    l0.as_ptr(),
                    r0.as_ptr(),
                    l0.as_ptr() == r0.as_ptr()
                );*/
                l0.as_ptr() == r0.as_ptr()
            }
            (Self::LargeSprite(l0), Self::LargeSprite(r0)) => {
                /*println!(
                    "{:#?}, {:#?}, eq:{}",
                    l0.as_ptr(),
                    r0.as_ptr(),
                    l0.as_ptr() == r0.as_ptr()
                );*/
                l0.as_ptr() == r0.as_ptr()
            }
            (Self::Texture(l0), Self::Texture(r0)) => {
                /* println!(
                    "{:#?}, {:#?}, eq:{}",
                    l0.as_ptr(),
                    r0.as_ptr(),
                    l0.as_ptr() == r0.as_ptr()
                );*/
                l0.as_ptr() == r0.as_ptr()
            }
            _ => false,
        }
    }
}
impl Eq for CachedTexture {}
impl CachedTexture {
    pub fn new_sprite(s: Sprite) -> Self {
        Self::Sprite(s.colors.clone())
    }
    pub fn new_large_sprite(s: LargeSprite) -> Self {
        Self::LargeSprite(s.colors.clone())
    }
    pub fn new_texture(s: Texture) -> Self {
        Self::Texture(s.data.clone())
    }
}
pub fn run_draw_cmd<T>(
    cmds: &mut impl Iterator<Item = Cmd>,
    draw: &mut RaylibTextureMode<'_, T>,
    cache: &mut HashMap<CachedTexture, RenderTexture2D>,
    font: &Font,
    text_scale: f32,
    text_offset: i32,
    text_height: i32,
) -> bool {
    let Some(i) = cmds.next() else { return true };
    match i {
        Cmd::DrawPixel { x, y, color } => {
            draw.draw_pixel(x, y, color.as_color());
        }
        Cmd::DrawCharacter { x, y, color, c } => {
            draw.draw_text_codepoint(
                &font,
                c as i32,
                Vector2::new(x as f32, y as f32),
                text_scale,
                color.as_color(),
            );
        }
        Cmd::DrawRect { x, y, w, h, color } => {
            draw.draw_rectangle(x, y, w, h, color.as_color());
        }
        Cmd::DrawBox { x, y, w, h, color } => {
            draw.draw_line(x, y, x + w, y, color.as_color());
            draw.draw_line(x, y, x, y + h, color.as_color());
            draw.draw_line(x + w, y, x + w, y + h, color.as_color());
            draw.draw_line(x, y + h, x + w, y + h, color.as_color());
        }
        Cmd::DrawCircle { x, y, rx10, color } => {
            draw.draw_circle(x, y, (rx10 as f32) / 10.0, color.as_color());
        }
        Cmd::DrawString {
            text,
            x,
            y,
            w,
            color,
            wrap,
        } => {
            let mut dx = x;
            let mut dy = y;
            for i in text.chars() {
                draw.draw_text_codepoint(
                    &font,
                    i as i32,
                    Vector2::new(dx as f32, dy as f32),
                    text_scale,
                    color.as_color(),
                );
                dx += text_offset;
                if (dx > x + w || dx > SCREEN_WIDTH) && wrap {
                    dx = x;
                    dy += text_height;
                }
            }
        }
        Cmd::DrawSprite { x, y, h, w, sprite } => {
            let tw = 16 as f32;
            let th = 16 as f32;
            draw.draw_texture_pro(
                cache
                    .get(&CachedTexture::new_sprite(sprite))
                    .unwrap()
                    .texture(),
                Rectangle::new(0., 0., tw, -th),
                Rectangle::new(x as f32, y as f32, w as f32, h as f32),
                Vector2::zero(),
                0.0,
                Color::WHITE,
            );
        }
        Cmd::DrawLargeSprite { x, y, h, w, sprite } => {
            let tw = 32 as f32;
            let th = 32 as f32;
            draw.draw_texture_pro(
                cache
                    .get(&CachedTexture::new_large_sprite(sprite))
                    .unwrap()
                    .texture(),
                Rectangle::new(0., 0., tw, -th),
                Rectangle::new(x as f32, y as f32, w as f32, h as f32),
                Vector2::zero(),
                0.0,
                Color::WHITE,
            );
        }
        Cmd::DrawTexture {
            x,
            y,
            h,
            w,
            texture,
        } => {
            let tw = texture.w as f32;
            let th = texture.h as f32;
            draw.draw_texture_pro(
                cache
                    .get(&CachedTexture::new_texture(texture))
                    .unwrap()
                    .texture(),
                Rectangle::new(0., 0., tw, -th),
                Rectangle::new(x as f32, y as f32, w as f32, h as f32),
                Vector2::new(0.0, 0.0),
                0.0,
                Color::WHITE,
            );
        }
        Cmd::DrawLine {
            start_x,
            end_x,
            start_y,
            end_y,
            color,
        } => {
            draw.draw_line(start_x, start_y, end_x, end_y, color.as_color());
        }
        Cmd::BeginScissor { x, y, w, h } => {
            let mut mode = draw.begin_scissor_mode(x, y, w, h);
            loop {
                let tmp = run_draw_cmd(
                    cmds,
                    &mut mode,
                    cache,
                    font,
                    text_scale,
                    text_offset,
                    text_height,
                );
                if tmp {
                    return false;
                }
            }
        }
        Cmd::EndScissor {
            x: _,
            y: _,
            w: _,
            h: _,
        } => {
            return true;
        }
        Cmd::BeginDrawing => {
            draw.clear_background(Color::BLACK);
        }
        Cmd::EndDrawing { time_stamp } => {
            draw.draw_text(
                &format!(
                    "{} millisecond delay",
                    std::time::UNIX_EPOCH
                        .elapsed()
                        .unwrap()
                        .as_millis()
                        .abs_diff(time_stamp)
                ),
                10,
                400,
                10,
                Color::WHITE,
            );
        }
        Cmd::Update(_) => {}
    }
    false
}
pub fn draw_loop(
    cmds: Vec<Cmd>,
    handle: &mut RaylibHandle,
    thread: &RaylibThread,
    cache: &mut HashMap<CachedTexture, RenderTexture2D>,
    frame_buffer: &mut RenderTexture2D,
    font: &Font,
    text_scale: f32,
    text_offset: i32,
    text_height: i32,
) {
    for i in &cmds {
        match i {
            Cmd::DrawLargeSprite {
                x: _,
                y: _,
                h: _,
                w: _,
                sprite,
            } => {
                let k = CachedTexture::new_large_sprite(sprite.clone());
                if !cache.contains_key(&k) {
                    let mut rt = handle.load_render_texture(&thread, 32, 32).unwrap();
                    let mut h = handle.begin_texture_mode(thread, &mut rt);
                    for i in 0..32 {
                        for j in 0..32 {
                            h.draw_pixel(j, i, sprite.colors[i as usize][j as usize].as_color());
                        }
                    }
                    drop(h);
                    cache.insert(k, rt);
                }
            }
            Cmd::DrawSprite {
                x: _,
                y: _,
                h: _,
                w: _,
                sprite,
            } => {
                let k = CachedTexture::new_sprite(sprite.clone());
                if !cache.contains_key(&k) {
                    println!("drew");
                    let mut rt = handle.load_render_texture(&thread, 16, 16).unwrap();
                    let mut h = handle.begin_texture_mode(thread, &mut rt);
                    for i in 0..16 {
                        for j in 0..16 {
                            h.draw_pixel(j, i, sprite.colors[i as usize][j as usize].as_color());
                        }
                    }
                    drop(h);
                    cache.insert(k, rt);
                }
            }
            Cmd::DrawTexture {
                x: _,
                y: _,
                h: _,
                w: _,
                texture,
            } => {
                let k = CachedTexture::new_texture(texture.clone());
                if !cache.contains_key(&k) {
                    let w = texture.w;
                    let h = texture.h;
                    let mut rt = handle
                        .load_render_texture(&thread, w as u32, h as u32)
                        .unwrap();
                    let mut hd = handle.begin_texture_mode(thread, &mut rt);
                    for i in 0..h {
                        for j in 0..w {
                            hd.draw_pixel(j, i, texture.data[(i * w + j) as usize].as_color());
                        }
                    }
                    drop(hd);
                    cache.insert(k, rt);
                }
            }
            _ => {
                continue;
            }
        }
    }
    let mut draw = handle.begin_texture_mode(&thread, frame_buffer);
    //  draw.clear_background(Color::BLACK);
    let mut cmds = cmds.into_iter();
    loop {
        let x = run_draw_cmd(
            &mut cmds,
            &mut draw,
            cache,
            font,
            text_scale,
            text_offset,
            text_height,
        );
        if x {
            break;
        }
    }
}

impl Device {
    pub fn update_process(
        &mut self,
        cache: &mut HashMap<CachedTexture, RenderTexture2D>,
        idx: u64,
        updated: &mut bool,
    ) {
        let input = input::generate_input(&mut self.handle, 2.0, 2.0);
        if self.active_process != idx {
            return;
        }
        let inp = HandleUpdate {
            input,
            pressed_key: self.handle.get_char_pressed(),
            should_close: self.handle.window_should_close(),
        };
        let mut should_draw = false;
        let mut cmds = Vec::new();
        for n in self.processes.get_mut(&idx).unwrap().handle.by_ref() {
            if n.is_err() {
                break;
            }
            let tmp = n.unwrap();
            match tmp {
                Cmd::EndDrawing { time_stamp: _ } => {
                    should_draw = true;
                    cmds.push(tmp);
                    break;
                }
                _ => {}
            }
            cmds.push(tmp);
        }
        for i in cmds {
            self.processes.get_mut(&idx).unwrap().cmds.push(i);
        }
        {
            if should_draw {
                self.processes
                    .get_mut(&idx)
                    .unwrap()
                    .handle
                    .send(Cmd::Update(inp))
                    .unwrap();

                *updated = true;
                draw_loop(
                    self.processes.get(&idx).unwrap().cmds.clone(),
                    &mut self.handle,
                    &self.thread,
                    cache,
                    &mut self.processes.get_mut(&idx).unwrap().target,
                    &self.font,
                    self.text_scale,
                    self.text_offset,
                    self.text_height,
                );
            }
            self.processes.get_mut(&idx).unwrap().cmds.clear();
        }
    }
    pub fn run_loop(mut self) {
        let mut scanline_idx = 0.0;
        let mut shader = self.handle.load_shader(
            &self.thread,
            Some("shaders/vert.glsl"),
            Some("shaders/retro.glsl"),
        );
        let fancy = shader.get_shader_location("fancy");
        let scanline = shader.get_shader_location("scanline");
        shader.set_shader_value(fancy, 1);
        let mut idx = 0;
        let mut cache = HashMap::new();
        while !self.handle.window_should_close() {
            {
                if self.handle.is_key_down(KeyboardKey::KEY_LEFT_CONTROL) {
                    if self.handle.is_key_down(KeyboardKey::KEY_ONE) {
                        if self.processes.contains_key(&1) {
                            self.active_process = 1;
                        }
                    }
                    if self.handle.is_key_down(KeyboardKey::KEY_TWO) {
                        if self.processes.contains_key(&2) {
                            self.active_process = 2;
                        }
                    }
                    if self.handle.is_key_down(KeyboardKey::KEY_THREE) {
                        if self.processes.contains_key(&3) {
                            self.active_process = 3;
                        }
                    }
                    if self.handle.is_key_down(KeyboardKey::KEY_FOUR) {
                        if self.processes.contains_key(&4) {
                            self.active_process = 4;
                        }
                    }
                    if self.handle.is_key_down(KeyboardKey::KEY_FIVE) {
                        if self.processes.contains_key(&5) {
                            self.active_process = 5;
                        }
                    }
                    if self.handle.is_key_down(KeyboardKey::KEY_SIX) {
                        if self.processes.contains_key(&6) {
                            self.active_process = 6;
                        }
                    }
                    if self.handle.is_key_down(KeyboardKey::KEY_SEVEN) {
                        if self.processes.contains_key(&7) {
                            self.active_process = 7;
                        }
                    }
                    if self.handle.is_key_down(KeyboardKey::KEY_EIGHT) {
                        if self.processes.contains_key(&8) {
                            self.active_process = 8;
                        }
                    }
                    if self.handle.is_key_down(KeyboardKey::KEY_NINE) {
                        if self.processes.contains_key(&9) {
                            self.active_process = 9;
                        }
                    }
                    if self.handle.is_key_down(KeyboardKey::KEY_ZERO) {
                        if self.processes.contains_key(&10) {
                            self.active_process = 10;
                        }
                    }
                }
                let mut updated = false;
                let procs: Vec<u64> = self.processes.iter().map(|(i, _)| *i).collect();
                for i in procs {
                    self.update_process(&mut cache, i, &mut updated);
                }
                if updated {
                    let mut draw = self.handle.begin_drawing(&self.thread);
                    draw.clear_background(Color::BLACK);
                    shader.set_shader_value(scanline, scanline_idx);
                    scanline_idx += 0.001;
                    if scanline_idx > 1.0 {
                        scanline_idx = 0.0;
                    }
                    draw.draw_shader_mode(&mut shader, |mut draw| {
                        for (idx, i) in &self.processes {
                            if *idx != self.active_process {
                                continue;
                            }
                            let fb = i.target.texture();

                            draw.draw_texture_pro(
                                fb,
                                Rectangle::new(
                                    0 as f32,
                                    0 as f32,
                                    SCREEN_WIDTH as f32,
                                    -SCREEN_HEIGHT as f32,
                                ),
                                Rectangle::new(
                                    0 as f32,
                                    0 as f32,
                                    SCREEN_WIDTH as f32 * 2. * i.scale,
                                    SCREEN_HEIGHT as f32 * 2. * i.scale,
                                ),
                                Vector2::new(i.x as f32, i.y as f32),
                                0.0,
                                Color::WHITE,
                            );
                        }
                    });
                    draw.draw_fps(10, 10);
                    drop(draw);
                    if idx > 200 {
                        if !self.handle.is_window_fullscreen() {
                            //      self.handle.toggle_fullscreen();
                        }
                    } else {
                        idx += 1;
                    }
                }
            }
        }
        self.processes.clear();
        drop(self.font);
        drop(cache);
    }
    pub fn spawn_process(&mut self, proc: Process) -> u64 {
        if self.processes.len() > 10 {
            return 0;
        }
        for i in 1..u64::MAX {
            if !self.processes.contains_key(&i) {
                self.processes.insert(i, proc);
                return i;
            }
        }
        unreachable!()
    }
}

pub fn main_loop(cmd: BStream<Cmd>) {
    let (mut handle, thread) = raylib::init()
        .size(SCREEN_WIDTH * 2, SCREEN_HEIGHT * 2)
        .build();
    let frame_buffer = handle
        .load_render_texture(&thread, SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
        .unwrap();
    let font = handle
        .load_font(&thread, "Commodore Angled v1.2.ttf")
        .unwrap();
    handle.set_exit_key(None);
    handle.set_window_focused();
    handle.set_target_fps(60);
    let proc = Process {
        x: 0,
        y: 0,
        handle: cmd,
        scale: 1.0,
        cmds: Vec::new(),
        target: frame_buffer,
    };
    let text_scale = TEXT_SCALE;
    let text_offset = TEXT_OFFSET;
    let text_height = (text_scale) as i32;
    let (incoming, outgoing) = BPipe::create();
    let mut dev = Device {
        handle,
        thread,
        processes: BTreeMap::new(),
        font,
        text_scale,
        text_offset,
        incoming_processes: incoming,
        text_height,
        active_process: 1,
    };
    if false {
        std::thread::spawn(move || listener(outgoing));
    }
    dev.spawn_process(proc);
    dev.run_loop();
}

pub fn listener(writer: BPipe<BStream<Cmd>>) {
    let stream = TcpListener::bind("127.0.0.1:12000").unwrap();
    stream.set_nonblocking(true).unwrap();
    'outer: loop {
        while let Ok((x, _)) = stream.accept() {
            let s = BStream::from_stream(x);
            if writer.send(s).is_err() {
                break 'outer;
            }
        }
    }
}
