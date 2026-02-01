use dos::{Sprite, SysHandle, setup};

fn main() {
    setup(main_func);
}

pub fn main_func(mut handle: SysHandle) {
    let strings: Vec<String> = (0..30).map(|i| format!("hello :{i}")).collect();
    let mut df = 0.0;
    while !handle.should_exit() {
        handle.begin_drawing();
        handle.begin_div(200, 380);
        let (f, hit) = handle.draw_button_image_scroll_box_exp(
            2,
            2,
            200,
            200,
            10,
            df,
            false,
            &strings,
            |x| x.to_string(),
            |_| Sprite {
                name: "image.png".to_string(),
            },
        );
        df = f;
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
