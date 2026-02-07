use std::collections::BTreeMap;

use raylib::color::Color;

use crate::{BColor, DrawCall, Pallete, Pos2, Rect, SysHandle, SysUiMode};
pub enum BrushMode {
    Lines,
    Circle,
    Rectangle,
}
pub struct AttractMode {
    pub project_name: String,
}
pub struct PixelArtMode {
    pub primary_color: usize,
    pub secondary_color: usize,
    pub tertiary_color: usize,
    pub pallete: Pallete,
    pub seed_color: BColor,
    pub image: Box<[BColor]>,
    pub width: i16,
    pub height: i16,
    pub old_pos: Pos2,
    pub start_pos: Option<Pos2>,
    pub save_name: String,
    pub brush_mode: BrushMode,
}
pub struct DungeonEditMode {
    pub dungeon: Box<[u16]>,
    pub decorations: Box<[u16]>,
    pub images: BTreeMap<u16, String>,
    pub width: i16,
    pub height: i16,
}
pub enum AppState {
    Attract(AttractMode),
    PixelArt(PixelArtMode),
    Dungeon(DungeonEditMode),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppStateMode {
    Attract,
    PixelArt,
    Dungeon,
}

impl AppState {
    pub fn run(&mut self, mut handle: SysHandle) {
        let mut mode = AppStateMode::Attract;
        while !handle.should_exit() {
            let old_mode = mode;
            match self {
                AppState::Attract(attract_mode) => {
                    mode = attract_mode.update(&mut handle);
                }
                AppState::PixelArt(pixel_art_mode) => {
                    mode = pixel_art_mode.update(&mut handle);
                }
                AppState::Dungeon(dungeon_edit_mode) => todo!(),
            }
            if old_mode != mode {
                match mode {
                    AppStateMode::Attract => {
                        *self = AppState::Attract(AttractMode {
                            project_name: String::new(),
                        });
                    }
                    AppStateMode::PixelArt => {
                        *self = AppState::PixelArt(PixelArtMode {
                            primary_color: 249,
                            secondary_color: 250,
                            tertiary_color: 251,
                            pallete: Pallete::basic(),
                            seed_color: BColor {
                                r: 0,
                                g: 255,
                                b: 255,
                                a: 255,
                            },
                            image: vec![
                                const {
                                    BColor {
                                        r: 0,
                                        g: 0,
                                        b: 25,
                                        a: 255,
                                    }
                                };
                                64 * 64
                            ]
                            .into_boxed_slice(),
                            width: 64,
                            height: 64,
                            save_name: "test.png".to_string(),
                            brush_mode: BrushMode::Circle,
                            start_pos: None,
                            old_pos: handle.get_mouse_pos(),
                        })
                    }
                    AppStateMode::Dungeon => todo!(),
                }
            }
        }
    }
    pub fn setup_run(handle: SysHandle) {
        let mut slf = Self::Attract(AttractMode {
            project_name: "".to_string(),
        });
        slf.run(handle);
    }
}

pub fn run(handle: SysHandle) {
    AppState::setup_run(handle);
}
impl AttractMode {
    pub fn update(&mut self, handle: &mut SysHandle) -> AppStateMode {
        let mut out = AppStateMode::Attract;
        handle.begin_drawing();
        handle.begin_div(300, 600);
        if handle.draw_button(300, 10, "start pixel art") {
            out = AppStateMode::PixelArt;
        }
        handle.end_div();
        handle.end_drawing();
        out
    }
}

impl PixelArtMode {
    pub fn update(&mut self, handle: &mut SysHandle) -> AppStateMode {
        let mut out = AppStateMode::PixelArt;
        handle.begin_drawing();
        handle.begin_div(100, 100);
        if handle.draw_button(100, 10, "exit") {
            out = AppStateMode::Attract;
        }
        handle.draw_text(10, 100, &format!("save as:{:#?}", self.save_name));
        if let Some(sn) = handle.text_user_input_saved_exp("save file", 4, 4, 100, 10, 10) {
            self.save_name = sn + ".png";
        }
        if handle.draw_button(100, 10, "save") {
            let mut image = raylib::prelude::Image::gen_image_color(
                self.width as i32,
                self.height as i32,
                Color::BLACK,
            );
            for i in 0..self.height {
                for j in 0..self.width {
                    image.draw_pixel(
                        j as i32,
                        i as i32,
                        self.image[i as usize * self.width as usize + j as usize].as_rl_color(),
                    );
                }
            }
            image.export_image(&format!("output/{}", self.save_name));
        }
        handle.end_div();
        handle.set_cursor(100, 100);
        handle.begin_div(700, 600);
        handle.set_sys_ui_mode(SysUiMode::Relative);
        let c0 = handle.get_cursor();
        let mut calls = Vec::new();
        let cw = 8;
        for i in 0..self.height {
            for j in 0..self.width {
                let x = (j * cw) as i32 + c0.x;
                let y = (i * cw) as i32 + c0.y;
                if handle.left_mouse_down() {
                    match &self.brush_mode {
                        BrushMode::Lines => {
                            if self.start_pos.is_none() {
                                self.start_pos = Some(Pos2 { x, y });
                            }
                        }
                        BrushMode::Circle => {
                            let p = handle.get_mouse_pos();
                            let xp = (p.x as f64 - c0.x as f64) / (cw as f64);
                            let yp = (p.y as f64 - c0.y as f64) / (cw as f64);
                            let dy = yp - i as f64;
                            let dx = xp - j as f64;
                            let delt = dx * dx + dy * dy;
                            if delt.sqrt() < 1.5 {
                                println!(
                                    "{}, {}, xp,{} yp:{}, p:{:#?}, i:{}, j:{}",
                                    dx, dy, xp, yp, p, i, j
                                );
                                self.image[(i * self.width + j) as usize] =
                                    self.pallete.colors[self.primary_color];
                            }
                        }
                        BrushMode::Rectangle => {}
                    }
                } else {
                    self.start_pos = None;
                }
                calls.push(DrawCall::Rectangle {
                    x,
                    y,
                    w: cw as i32,
                    h: cw as i32,
                    color: self.image[(i * self.width + j) as usize],
                    drop_shadow: false,
                    outline: false,
                });
            }
        }
        handle.send_draw_calls(
            calls,
            Rect {
                x: c0.x,
                y: c0.y,
                w: 512,
                h: 512,
            },
        );
        handle.set_cursor(650, 100);
        let c0 = handle.get_cursor();
        let mut calls = Vec::new();
        for i in 0..16 {
            for j in 0..16 {
                let p = i * 16 + j;
                let x = c0.x + j * 16;
                let y = c0.y + i * 16;
                if handle.left_mouse_released() {
                    let rect = Rect { x, y, w: 12, h: 12 };
                    if rect.check_collision(handle.get_mouse_pos()) {
                        self.primary_color = p as usize;
                    }
                }
                calls.push(DrawCall::Rectangle {
                    x: x + 2,
                    y: y + 2,
                    w: 12,
                    h: 12,
                    color: self.pallete.colors[p as usize],
                    drop_shadow: true,
                    outline: false,
                });
            }
        }
        calls.push(DrawCall::DrawText {
            x: c0.x + 264,
            y: c0.y + 100,
            size: 10,
            contents: "primary color".to_string(),
            color: handle.get_theme().text_color,
        });
        calls.push(DrawCall::Rectangle {
            x: c0.x + 264,
            y: c0.y + 115,
            w: 16,
            h: 16,
            color: self.pallete.colors[self.primary_color],
            drop_shadow: true,
            outline: false,
        });
        calls.push(DrawCall::DrawText {
            x: c0.x + 264,
            y: c0.y + 140,
            size: 10,
            contents: "secondary color".to_string(),
            color: handle.get_theme().text_color,
        });
        calls.push(DrawCall::Rectangle {
            x: c0.x + 264,
            y: c0.y + 155,
            w: 16,
            h: 16,
            color: self.pallete.colors[self.secondary_color],
            drop_shadow: true,
            outline: false,
        });
        calls.push(DrawCall::DrawText {
            x: c0.x + 264,
            y: c0.y + 180,
            size: 10,
            contents: "tertiary color".to_string(),
            color: handle.get_theme().text_color,
        });
        calls.push(DrawCall::Rectangle {
            x: c0.x + 264,
            y: c0.y + 195,
            w: 16,
            h: 16,
            color: self.pallete.colors[self.tertiary_color],
            drop_shadow: true,
            outline: false,
        });
        handle.send_draw_calls(
            calls,
            Rect {
                x: c0.x,
                y: c0.y,
                w: 400,
                h: 256,
            },
        );
        let secondary = Rect {
            x: c0.x + 264,
            y: c0.y + 155,
            w: 16,
            h: 16,
        };
        let tertiary = Rect {
            x: c0.x + 264,
            y: c0.y + 195,
            w: 16,
            h: 16,
        };
        if handle.left_mouse_released() {
            if secondary.check_collision(handle.get_mouse_pos()) {
                let tmp = self.secondary_color;
                self.secondary_color = self.primary_color;
                self.primary_color = tmp;
            }
            if tertiary.check_collision(handle.get_mouse_pos()) {
                let tmp = self.tertiary_color;
                self.tertiary_color = self.primary_color;
                self.primary_color = tmp;
            }
        }
        self.old_pos = handle.get_mouse_pos();
        handle.end_div();
        handle.end_drawing();
        out
    }
}
