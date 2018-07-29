extern crate piston_window;
extern crate ai_behavior;
extern crate sprite;
extern crate find_folder;
extern crate image;
extern crate imageproc;
extern crate rusttype;

mod vgui;
use std::rc::Rc;
use piston_window::*;
use sprite::*;
use ai_behavior::Action;

fn main() {
    let (width, height) = (300, 300);
    let opengl = OpenGL::V3_2;
    let mut window: PistonWindow =
        WindowSettings::new("piston: sprite", (width, height))
        .exit_on_esc(true)
        .opengl(opengl)
        .build()
        .unwrap();

    let assets = find_folder::Search::ParentsThenKids(3, 3)
        .for_folder("assets").unwrap();
    let mut scene = Scene::new();
    let tex = Rc::new(Texture::from_path(
            &mut window.factory,
            assets.join("rust.png"),
            Flip::None,
            &TextureSettings::new()
        ).unwrap());
    let mut sprite = Sprite::from_texture(tex.clone());
    sprite.set_position(width as f64 / 2.0, height as f64 / 2.0);
    let id = scene.add_child(sprite);

    let font = vgui::load_font("FiraSans-Regular.ttf").expect("Cannot load font.");
    let mut sprite3 = vgui::MenuEntry{ label: String::from("Hola!"), font: &font }.make_sprite(&mut window.factory);
    sprite3.set_position(500.0, 500.0);
    let id3 = scene.add_child(sprite3);

    let rotate = Action(Ease(EaseFunction::ExponentialInOut,
        Box::new(MoveBy(1.0, 0.0, 200.0))));
    scene.run(id3, &rotate);

    while let Some(e) = window.next() {
        scene.event(&e);

        window.draw_2d(&e, |c, g| {
            clear([1.0, 1.0, 1.0, 1.0], g);
            scene.draw(c.transform, g);
        });
    }
}
