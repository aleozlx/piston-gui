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
mod h5meta;
mod h5slice;
use std::rc::Rc;
use std::path::PathBuf;
use vgui::SpritePrototype;
use vgui::MenuAdapter;
use vgui::VGUIFont;
use h5meta::{H5Obj, H5Group};
use h5slice::{H5URI, Dtype, H5Cache};
use piston_window::*;
use sprite::*;

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
    s_menu.set_position(-300.0, 15.0);
    menu.uuid_self = Some(scene.add_child(s_menu));
    scene.run(menu.uuid_self.unwrap(),
        &ai_behavior::Action(Ease(EaseFunction::ExponentialIn,
            Box::new(MoveTo(0.2, 15.0, 15.0)))));
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
    let mut h5pointer = PathBuf::from(&h5root.name());
    let mut menu = vgui::Menu::adapt(h5root.locate_group(&h5pointer).unwrap(), Rc::clone(&font));
    register_menu(&mut scene, &mut menu, &mut window.factory);

    let mut image_cache = H5Cache::new();
    
    // if let Some(im_test) = h5slice::get_one(H5URI{
    //     path: String::from("/home/alex/datasets/ucm-sample.h5"),
    //     h5path: String::from("/source/images"),
    //     query: String::from("13"),
    //     dtype: Dtype::F4
    // }, (224,224)) {
        // let tex = Rc::new(Texture::from_image(
        //     &mut window.factory,
        //     &im_test,
        //     &TextureSettings::new()
        // ).unwrap());
        // let mut sprite_tex = Sprite::from_texture(tex.clone());
        // sprite_tex.set_position(120.0, 320.0);
        // let id_tex = scene.add_child(sprite_tex);

        // scene.run(id_tex, &ai_behavior::Action(Ease(EaseFunction::CircularInOut, Box::new(MoveTo(0.5, 115.0, 320.0)))));
    // }
    // else {
    //     println!("Not found!")
    // }
    
    

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
                                menu = vgui::Menu::adapt(h5root.locate_group(&h5pointer).unwrap(), Rc::clone(&font));
                                register_menu(&mut scene, &mut menu, &mut window.factory); 
                            },
                            _ => {
                                h5pointer.push(entry);
                                match h5root.locate(&h5pointer) {
                                    H5Obj::Group(g) => {
                                        scene.remove_child(menu.uuid_self.unwrap());
                                        menu = vgui::Menu::adapt(g, Rc::clone(&font));
                                        register_menu(&mut scene, &mut menu, &mut window.factory);
                                    },
                                    H5Obj::Dataset(d) => {
                                        if let Some(resolution) = d.resolution_batch_images() {
                                            println!("resolution {}", &resolution);

                                            if let Some(im) = image_cache.request_one(&H5URI{
                                                path: String::from("/home/alex/datasets/ucm-sample.h5"),
                                                h5path: String::from(h5pointer.to_str().unwrap()),
                                                query: String::from("14"),
                                                dtype: Dtype::F4
                                            }, &resolution.into()) {
                                                let tex = Rc::new(Texture::from_image(
                                                    &mut window.factory,
                                                    &im,
                                                    &TextureSettings::new()
                                                ).unwrap());
                                                let mut sprite_tex = Sprite::from_texture(tex.clone());
                                                sprite_tex.set_position(120.0, 320.0);
                                                let _id_tex = scene.add_child(sprite_tex);
                                            }
                                        }
                                        h5pointer.pop();
                                    }
                                }
                            }
                        }
                    }
                },
                Key::Left => {
                    if h5pointer != PathBuf::from("/") {
                        h5pointer.pop();
                        scene.remove_child(menu.uuid_self.unwrap());
                        menu = vgui::Menu::adapt(h5root.locate_group(&h5pointer).unwrap(), Rc::clone(&font));
                        register_menu(&mut scene, &mut menu, &mut window.factory);
                    }
                }
                _ => {}
            }
        };
    }
}
