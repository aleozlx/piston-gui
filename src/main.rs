extern crate piston_window;
extern crate ai_behavior;
extern crate sprite;
extern crate find_folder;
extern crate image;
extern crate imageproc;
extern crate rusttype;
extern crate gfx;
extern crate regex;
extern crate flate2;

mod vgui;
mod h5ls_reader;
use std::rc::Rc;
use std::io::prelude::*;
use std::net::TcpStream;
use std::path::PathBuf;
use vgui::SpritePrototype;
use vgui::MenuAdapter;
use vgui::VGUIFont;
use h5ls_reader::{H5Obj, H5Group, H5Dataset};
use piston_window::*;
use sprite::*;
use flate2::read::GzDecoder;

impl MenuAdapter<H5Group> for vgui::Menu {
    fn adapt(group: &H5Group, font: VGUIFont) -> vgui::Menu {
        let ref mut group_entries: Vec<String> = group.children.keys().cloned().collect();
        let ref mut menu_entries =
            if group.name == "/" { Vec::new() } else { vec![String::from("..")] };
        menu_entries.append(group_entries);
        let mut ret = vgui::Menu::new(menu_entries, font);
        for entry in &mut ret.entries {
            if &entry.label == ".." { continue; }
            if let H5Obj::Dataset(dataset) = &group.children[&entry.label] {
                entry.tag = Some(String::from(dataset.format()));
            }
        }
        return ret;
    }
}

fn register_menu<F, R>(scene: &mut sprite::Scene<piston_window::Texture<R>>, menu: &mut vgui::Menu, factory: &mut F)
    where F: gfx::Factory<R>, R: gfx::Resources {
    let mut s_menu = menu.make_sprite(factory);
    s_menu.set_position(15.0, 15.0);
    menu.uuid_self = Some(scene.add_child(s_menu));   
}

fn test_h5slice() {
    let mut stream = TcpStream::connect("localhost:8000").unwrap();
    let mut buffer_in = Vec::with_capacity(8<<10);
    let mut buffer_out = Vec::with_capacity(4<<20);
    // ignore the Result
    let _ = stream.write("hi\n".as_bytes());
    if let Ok(n) = stream.read_to_end(&mut buffer_in) {
        println!("Read {} bytes from network.", n);
        let mut decoder = GzDecoder::new(&buffer_in[..]);
        match decoder.read_to_end(&mut buffer_out) {
            Ok(n) => {println!("Decompressed into {} bytes.", n);},
            Err(err) => {println!("Error: {}", err);}
        }
    }
    
}

fn main() {
    let (width, height) = (800, 600);
    let opengl = OpenGL::V3_2;
    let mut window: PistonWindow =
        WindowSettings::new("piston: sprite", (width, height))
        .exit_on_esc(true)
        // .fullscreen(true)
        .opengl(opengl)
        .build()
        .unwrap();
    let mut scene = Scene::new();
    let font = vgui::load_font("FiraSans-Regular.ttf").expect("Cannot load font.");
    let h5root = H5Group::parse("/home/alex/datasets/ucm-sample.h5.txt").expect("IO Error");
    let mut h5pointer = PathBuf::from(&h5root.name);
    let mut menu = vgui::Menu::adapt(h5root.locate_group(&h5pointer), Rc::clone(&font));
    register_menu(&mut scene, &mut menu, &mut window.factory);

    test_h5slice();

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
                    if let Some(entry) = menu.get() {
                        match entry.as_ref() {
                            ".." => {
                                scene.remove_child(menu.uuid_self.unwrap());
                                h5pointer.pop();
                                menu = vgui::Menu::adapt(h5root.locate_group(&h5pointer), Rc::clone(&font));
                                register_menu(&mut scene, &mut menu, &mut window.factory); 
                            },
                            _ => {
                                h5pointer.push(entry);
                                if h5root.is_group(&h5pointer) {
                                    scene.remove_child(menu.uuid_self.unwrap());
                                    menu = vgui::Menu::adapt(h5root.locate_group(&h5pointer), Rc::clone(&font));
                                    register_menu(&mut scene, &mut menu, &mut window.factory);
                                }
                                else {
                                    h5pointer.pop();
                                }
                            }
                        }
                    }
                },
                Key::Left => {
                    if h5pointer != PathBuf::from("/") {
                        h5pointer.pop();
                        scene.remove_child(menu.uuid_self.unwrap());
                        menu = vgui::Menu::adapt(h5root.locate_group(&h5pointer), Rc::clone(&font));
                        register_menu(&mut scene, &mut menu, &mut window.factory);
                    }
                }
                _ => {}
            }
        };
    }
}
