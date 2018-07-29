extern crate piston_window;
extern crate ai_behavior;
extern crate sprite;
extern crate find_folder;
extern crate image;
extern crate imageproc;
extern crate rusttype;

use std::rc::Rc;

use piston_window::*;
use sprite::*;
use ai_behavior::{
    Action
};

use imageproc::drawing::draw_text_mut;
use image::{Rgba, RgbaImage};
use rusttype::{FontCollection, Scale};

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
    let id;
    let mut scene = Scene::new();
    let tex = Rc::new(Texture::from_path(
            &mut window.factory,
            assets.join("rust.png"),
            Flip::None,
            &TextureSettings::new()
        ).unwrap());
    let mut sprite = Sprite::from_texture(tex.clone());
    sprite.set_position(width as f64 / 2.0, height as f64 / 2.0);

    let mut sprite2 = Sprite::from_texture(tex.clone());
    sprite2.set_position(300.0, 50.0);
    id = scene.add_child(sprite);
    let id2 = scene.add_child(sprite2);

    let mut image = RgbaImage::new(100, 100);
    let font = Vec::from(include_bytes!("../assets/FiraSans-Regular.ttf") as &[u8]);
    let font = FontCollection::from_bytes(font).unwrap().into_font().unwrap();
    let height = 32.0;
    let scale = Scale { x: height, y: height };
    draw_text_mut(&mut image, Rgba([0u8, 0u8, 255u8, 255u8]), 0, 0, scale, &font, "Hola!");
    let tex3 = Rc::new(Texture::from_image(
        &mut window.factory,
        &image,
        &TextureSettings::new()
    ).unwrap());
    let mut sprite3 = Sprite::from_texture(tex3.clone());
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
