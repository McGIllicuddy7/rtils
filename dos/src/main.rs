use std::process::exit;

use dos::{SysHandle, setup};

fn main() {
    setup(main_func);
}

pub fn main_func(mut handle: SysHandle) {
    let mut timer = 0.0;
    while !handle.should_exit() {
        handle.begin_drawing();
        handle.begin_div(100, 100, 200, 100, true, dos::SysUiMode::Sequential);
        handle.draw_text(2, 2, 10, 200, "hello world");
        if handle.draw_button(2, 2, 100, 10, "exit") {
            timer = 0.5;
        }
        handle.end_div();
        if timer > 0.0 {
            timer -= 0.016;
            if timer < 0. {
                timer = 0.0;
            }
            handle.draw_text(0, 100, 10, 100, "pressed");
        }
        handle.end_drawing();
    }
}
