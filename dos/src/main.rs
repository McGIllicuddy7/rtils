use std::process::exit;

use dos::{SysHandle, setup};

fn main() {
    setup(main_func);
}

pub fn main_func(mut handle: SysHandle) {
    let mut timer = 0.0;
    let mut amnt = 0.0;
    let strings: Vec<String> = (0..30).map(|i| format!("hello :{i}")).collect();
    while !handle.should_exit() {
        handle.begin_drawing();
        handle.set_cursor(100, 100);
        handle.begin_div(200, 380);
        let hit;
        (amnt, hit) =
            handle.draw_button_scroll_box_exp(2, 2, 200, 200, 10, amnt, false, &strings, |x| {
                x.to_string()
            });
        if let Some(h) = hit {
            println!("h:{h}");
        }
        handle.end_drawing();
    }
}
