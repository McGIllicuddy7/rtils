use std::collections::{BTreeMap, HashMap};

use crate::{SysHandle, rtils::rtils_useful::BPipe};

use super::common::*;
pub struct Dos {
    pub pallete: Pallete,
    pub render_texture: Option<RenderTexture2D>,
    pub loaded_textures: HashMap<String, Texture2D>,
    pub shader: Option<Shader>,
    pub w: i32,
    pub h: i32,
    pub scan_line: i32,
}

impl Default for Dos {
    fn default() -> Self {
        Self::new()
    }
}

impl Dos {
    pub fn draw(&mut self, handle: &mut RaylibDrawHandle, _thread: &RaylibThread) {
        let loc = self
            .shader
            .as_ref()
            .unwrap()
            .get_shader_location("scanline");
        self.shader
            .as_mut()
            .unwrap()
            .set_shader_value(loc, self.scan_line as f32 / (self.h as f32));
        let ploc = self.shader.as_ref().unwrap().get_shader_location("pallete");
        self.shader
            .as_mut()
            .unwrap()
            .set_shader_value_v(ploc, &self.pallete.as_rl_vec());
        self.scan_line += 1;
        self.scan_line %= self.h;
        handle.draw_shader_mode(self.shader.as_mut().unwrap(), |mut handle| {
            handle.draw_texture_pro(
                self.render_texture.as_ref().unwrap(),
                Rectangle::new(0.0, 0.0, SCREEN_WIDTH as f32, -(SCREEN_HEIGHT as f32)),
                Rectangle::new(0.0, 0.0, self.w as f32, self.h as f32),
                Vector2::zero(),
                0.0,
                Color::WHITE,
            );
        });
    }

    pub fn new() -> Self {
        Self {
            pallete: Pallete::new(BColor {
                r: 255,
                g: 255,
                b: 0,
                a: 255,
            }),
            shader: None,
            render_texture: None,
            loaded_textures: HashMap::new(),
            w: SCREEN_WIDTH,
            h: SCREEN_HEIGHT,
            scan_line: 0,
        }
    }

    #[allow(unused)]
    pub fn setup(&mut self, handle: &mut RaylibHandle, thread: &RaylibThread) {
        self.render_texture = Some(
            handle
                .load_render_texture(thread, SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
                .unwrap(),
        );
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
    pub fn load_image(
        &mut self,
        name: String,
        width: i32,
        height: i32,
        data: Arc<[BColor]>,
        handle: &mut RaylibHandle,
        thread: &RaylibThread,
    ) {
        let mut image = raylib::prelude::Image::gen_image_color(width, height, Color::WHITE);
        for i in 0..height {
            for j in 0..width {
                image.draw_pixel(j, i, data[(i * width + j) as usize].as_rl_color());
            }
        }
        let tex = handle.load_texture_from_image(thread, &image).unwrap();
        self.dos.loaded_textures.insert(name, tex);
    }

    pub fn update_cmds(&mut self, handle: &mut RaylibHandle, thread: &RaylibThread) {
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
                    DrawCall::LoadImage {
                        name,
                        width,
                        height,
                        data,
                    } => {
                        self.load_image(name, width, height, data, handle, thread);
                    }
                    DrawCall::UnloadedImage { name } => {
                        self.dos.loaded_textures.remove(&name);
                    }
                    _ => continue,
                }
            }
        }
    }

    pub fn run_loop(&mut self, mut handle: RaylibHandle, thread: RaylibThread) {
        while !self.should_exit {
            self.update_cmds(&mut handle, &thread);
            if self.should_draw {
                self.draw(&mut handle, &thread);
                if self.should_exit {
                    break;
                }
                self.render(&mut handle, &thread);
            }
        }
        self.dos.render_texture = None;
        self.dos.loaded_textures.clear();
    }

    pub fn draw(&mut self, handle: &mut RaylibHandle, thread: &RaylibThread) {
        if let Some(key) = handle.get_char_pressed() {
            self.input.pressed_keys.push(key);
        }
        if handle.window_should_close() {
            self.should_exit = true;
            self.cmd_pipeline.send(DrawCall::Exiting).unwrap();
            return;
        }
        self.input.left_mouse_down = handle.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT);
        self.input.right_mouse_down = handle.is_mouse_button_down(MouseButton::MOUSE_BUTTON_RIGHT);
        self.input.left_mouse_pressed =
            handle.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT);
        self.input.left_mouse_released =
            handle.is_mouse_button_released(MouseButton::MOUSE_BUTTON_LEFT);
        self.input.right_mouse_released =
            handle.is_mouse_button_released(MouseButton::MOUSE_BUTTON_RIGHT);
        if handle.is_key_pressed(KeyboardKey::KEY_BACKSPACE) {
            self.input.pressed_keys.push(127 as char);
        }
        if handle.is_key_pressed(KeyboardKey::KEY_ENTER) {
            self.input.pressed_keys.push('\n');
        }
        self.input.left_arrow_pressed = handle.is_key_pressed(KeyboardKey::KEY_LEFT);
        self.input.right_arrow_pressed = handle.is_key_pressed(KeyboardKey::KEY_RIGHT);
        let rat = self.dos.w as f32 / (SCREEN_WIDTH as f32);
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
        // drw.draw_fps(100, 100);
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
            DrawCall::DrawImage {
                x,
                y,
                h,
                w,
                contents_ident,
            } => {
                let Some(tex) = map.get(&contents_ident) else {
                    to_load.push(contents_ident.clone());
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
        to_load.dedup();
        for i in to_load {
            let Ok(x) = handle.load_texture(_thread, &i) else {
                continue;
            };
            self.dos.loaded_textures.insert(i, x);
        }
    }
}

pub fn setup(fn_main: impl FnOnce(super::SysHandle) + Send + 'static) {
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

    let w = (h * 4) / 3; //get_monitor_width(0);
    handle.set_window_size(w, h);
    handle.set_window_position(50, 50);
    let text = handle
        .load_render_texture(&thread, SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
        .unwrap();
    let mut rt = DosRt {
        dos: Dos {
            shader: Some(handle.load_shader(
                &thread,
                Some("./src/vert.glsl"),
                Some("./src/retro.glsl"),
            )),
            loaded_textures: HashMap::new(),
            pallete: Pallete::new(BColor {
                r: 0,
                g: 255,
                b: 255,
                a: 0,
            }),
            render_texture: Some(text),
            h,
            w,
            scan_line: 0,
        },
        cmd_pipeline: cmd1,
        frame: Vec::new(),
        recieving_frame: false,
        should_draw: false,
        should_exit: false,
        input: inp.clone(),
    };

    let (max, ratios) = text_ratios(&handle);
    let sys = SysHandle::new(cmd2, SCREEN_WIDTH, SCREEN_HEIGHT, ratios, max);
    let tj = std::thread::spawn(move || fn_main(sys));
    rt.run_loop(handle, thread);
    tj.join().unwrap();
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
