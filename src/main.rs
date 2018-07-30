extern crate piston_window;
extern crate ai_behavior;
extern crate sprite;
extern crate find_folder;
extern crate image;
extern crate imageproc;
extern crate rusttype;

mod vgui;
use vgui::SpriteMeta;
// use std::rc::Rc;
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

    // let assets = find_folder::Search::ParentsThenKids(3, 3)
    //     .for_folder("assets").unwrap();
    let mut scene = Scene::new();
    // let tex = Rc::new(Texture::from_path(
    //         &mut window.factory,
    //         assets.join("rust.png"),
    //         Flip::None,
    //         &TextureSettings::new()
    //     ).unwrap());
    // let mut sprite = Sprite::from_texture(tex.clone());
    // sprite.set_position(width as f64 / 2.0, height as f64 / 2.0);
    // let id = scene.add_child(sprite);

    let font = vgui::load_font("FiraSans-Regular.ttf").expect("Cannot load font.");
    let mut menu = vgui::Menu::new(&["Item 1", "Item 2", "Item 3","Item 1", "Item 2", "Item 3","Item 1", "Item 2", "Item 3"], &font);
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
