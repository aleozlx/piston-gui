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

fn menu_from_h5group<'a>(group: &h5ls_reader::H5Group, font: &'a rusttype::Font, root: bool) -> vgui::Menu<'a> {
    let ref mut group_entries: Vec<String> = group.children.keys().cloned().collect();
    let ref mut menu_entries = if root { Vec::new() } else { vec![String::from("..")] };
    menu_entries.append(group_entries);
    vgui::Menu::new(menu_entries, font)
}

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

    let ref fname_h5meta = std::path::PathBuf::from("/home/alex/datasets/ucm-sample.h5.txt");
    let mut menu;
    match h5ls_reader::parse(fname_h5meta) {
        Ok(root) => {
            menu = menu_from_h5group(&root, &font, true);
            let mut s_menu = menu.make_sprite(&mut window.factory);
            s_menu.set_position(15.0, 15.0);
            menu.uuid_self = Some(scene.add_child(s_menu));
        },
        Err(_) => { panic!("IO Error"); }
    }

    while let Some(e) = window.next() {
        scene.event(&e);

        window.draw_2d(&e, |c, g| {
            clear([1.0, 1.0, 1.0, 1.0], g);
            scene.draw(c.transform, g);
        });

        if let Some(Button::Keyboard(key)) = e.press_args() {
            match key {
                Key::Down => {
                    let (sid, shift) = menu.mv(1);
                    scene.run(sid, &shift);
                },
                Key::Up => {
                    let (sid, shift) = menu.mv(-1);
                    scene.run(sid, &shift);
                },
                Key::Right => {
                    if let Some(key) = menu.get() {
                        // if let Some(id) = menu.uuid_self {
                        //     scene.remove_child(id);
                        // }
                        println!("{}", &key);
                    }
                }
                _ => {}
            }
        };
    }
}
