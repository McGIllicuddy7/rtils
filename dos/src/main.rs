use dos::{SysHandle, setup};

fn main() {
    setup(main_func);
}

pub fn main_func(mut handle: SysHandle) {
    let strings: Vec<String> = (0..30).map(|i| format!("hello :{i}")).collect();
    while !handle.should_exit() {
        handle.begin_drawing();
        handle.begin_div(200, 380);
        let hit = handle.draw_button_scroll_box_saved_exp(
            "bx",
            2,
            2,
            200,
            200,
            10,
            false,
            &strings,
            |x| x.to_string(),
        );
        if let Some(h) = hit {
            println!("h:{h}");
        }
        handle.end_div();
        handle.begin_div(200, 380);
        let x = handle.text_user_input_saved_exp("text", 2, 2, 100, 50, 10);
        if let Some(msg) = x {
            println!("{msg}");
        }
        handle.end_div();
        handle.end_drawing();
    }
}
