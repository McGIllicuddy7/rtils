pub use amiga::*;

extern crate amiga;
_start! {{
  let mut dy = 0.0;
  let orc = load_sprite("orc.png").unwrap();
  let knight = load_large_sprite("knight.png").unwrap();
  let otexture = load_texture("orc.png").unwrap();
  let map = load_texture("map.png").unwrap();
   while !should_close(){
        begin_drawing();
        let text = "hello world! testing 1 2 3 4 5 6 7 8";
        draw_box(95, 95, 120, measure_text_height(text, 100)+5, Col::White);
        draw_text(text, 100, 100, 100, Col::White, true);
        dy = draw_scroll_box(  250, 100, 100, 100,dy,|cmds, w, _|{
          for i in 0..20{
            queue_draw_text(cmds, &format!("hi :{}",i), 0, i*10, w, Col::White, true);
          }
        });
        draw_texture(000, 000, 100, 100,map.clone());
        draw_sprite(100, 200, 16, 16, orc.clone());
        draw_large_sprite(150, 200, 32, 32, knight.clone());
        draw_texture(300, 100, 100, 100, otexture.clone());

        end_drawing();
    }
}}
