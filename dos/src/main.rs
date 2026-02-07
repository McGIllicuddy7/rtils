use dos::{SysHandle, setup, tilemap::FOREGROUND_LAYER, tilemap::TileMap};
fn main() {
    setup(main_func);
}

pub fn main_func(mut handle: SysHandle) {
    let draw_table = vec![];
    let mut map = TileMap::new(50, 50, "bg.png".to_string(), draw_table);
    let sprite_img = map.load_image("image.png");
    let _sprite = map.create_sprite(10, 10, 1, 1, sprite_img, FOREGROUND_LAYER);
    while !handle.should_exit() {
        handle.begin_drawing();
        handle.begin_div(800, 600);
        let (clicked, selected) = map.draw(300, 50, 500, 500, &mut handle, false);
        handle.end_div();
        if clicked.is_some() || selected.is_some() {
            println!("{:#?}, {:#?}", clicked, selected);
        }
        handle.end_drawing();
    }
}
