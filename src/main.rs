extern crate piston_window;
extern crate ai_behavior;
extern crate sprite;
extern crate find_folder;
extern crate image;
extern crate imageproc;
extern crate rusttype;
extern crate yaml_rust;

mod vgui;
use vgui::SpriteMeta;
use piston_window::*;
use sprite::*;
use yaml_rust::YamlLoader;

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
    let vgg16 = &YamlLoader::load_from_str(include_str!("../assets/vgg16-v2.yaml")).unwrap()[0];
    for block in vgg16["nnblocks"].as_vec().unwrap() {
        let blk_name = block["name"].as_str().unwrap();
        
        temp_entries.push(String::from(blk_name));
        // let blk_node = model.insert_with_values(None, None, &[0, 1], &[&blk_name, &0]);

        // for layer in block["layers"].as_vec().unwrap() {
        //     let layer_name = layer["class"].as_str().unwrap();
        //     model.insert_with_values(Some(&blk_node), None, &[0, 1], &[&layer_name, &0]);
        // }
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
