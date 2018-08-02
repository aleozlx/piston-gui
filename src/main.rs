extern crate piston_window;
extern crate ai_behavior;
extern crate sprite;
extern crate find_folder;
extern crate image;
extern crate imageproc;
extern crate rusttype;
extern crate regex;

mod vgui;
mod h5ls_reader;
use vgui::SpriteMeta;
use piston_window::*;
use sprite::*;

fn main() {
    let (width, height) = (800, 600);
    let opengl = OpenGL::V3_2;
    let mut window: PistonWindow =
        WindowSettings::new("piston: sprite", (width, height))
        .exit_on_esc(true)
        .opengl(opengl)
        .build()
        .unwrap();
    let mut scene = Scene::new();
    let font = vgui::load_font("FiraSans-Regular.ttf").expect("Cannot load font.");

    let mut temp_entries = Vec::<String>::new();

    let assets = find_folder::Search::ParentsThenKids(2, 2).for_folder("assets").unwrap();
    let ref fname_h5meta = assets.join("epoch1.h5.txt");
    if let Ok(root) = h5ls_reader::parse(fname_h5meta) {
        temp_entries.append(&mut root.children.keys().cloned().collect());
    }

    let mut menu = vgui::Menu::new(&temp_entries, &font);
    let mut s_menu = menu.make_sprite(&mut window.factory);
    s_menu.set_position(15.0, 15.0);
    scene.add_child(s_menu);

    while let Some(e) = window.next() {
        scene.event(&e);

        window.draw_2d(&e, |c, g| {
            clear([1.0, 1.0, 1.0, 1.0], g);
            scene.draw(c.transform, g);
        });

        if let Some(Button::Keyboard(key)) = e.press_args() {
            if key == Key::Down {
                let (sid, shift) = menu.mv(1);
                scene.run(sid, &shift);
            }
            else if key == Key::Up {
                let (sid, shift) = menu.mv(-1);
                scene.run(sid, &shift);
            }
            
        };
    }
}
