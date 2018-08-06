extern crate sprite;
extern crate find_folder;
extern crate rusttype;
extern crate gfx;
extern crate uuid;
extern crate ai_behavior;

use std;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;
use std::borrow::Borrow;
use piston_window::*;
use sprite::*;
use imageproc;
use imageproc::rect::Rect;
use image::{Rgba, RgbaImage};

pub type VGUIFont = std::rc::Rc<rusttype::Font<'static>>;

const ENTRY_HEIGHT: u32 = 32;
const COLUMN_WIDTH: u32 = 315;

pub trait SpritePrototype {
    fn make_sprite<F, R>(&mut self, factory: &mut F) -> Sprite<Texture<R>>
        where F: gfx::Factory<R>, R: gfx::Resources;
}

pub trait MenuAdapter<T> {
    fn adapt(group: &T, font: VGUIFont) -> Menu;
}

pub struct MenuEntry {
    pub label: String,
    pub font: VGUIFont,

    offset: usize
}

pub struct Menu {
    pub entries: Vec<MenuEntry>,
    
    cursor: usize,
    uuid_cursor: Option<uuid::Uuid>,
    pub uuid_self: Option<uuid::Uuid>
}

pub fn load_font(fname: &str) -> Result<VGUIFont, rusttype::Error> {
    let assets = find_folder::Search::ParentsThenKids(2, 2).for_folder("assets").unwrap();
    let ref fname_font = assets.join(fname);
    let mut fin_font = File::open(fname_font).expect(&format!("Cannot find font: {}", fname));
    let mut buffer = Vec::new();
    fin_font.read_to_end(&mut buffer).expect("IO error while reading the font.");
    Ok(Rc::new(rusttype::FontCollection::from_bytes(buffer).unwrap().into_font()?))
}

impl SpritePrototype for MenuEntry {
    fn make_sprite<F, R>(&mut self, factory: &mut F) -> Sprite<Texture<R>>
        where F: gfx::Factory<R>, R: gfx::Resources
    {
        const HEIGHT: u32 = ENTRY_HEIGHT;
        const WIDTH: u32 = COLUMN_WIDTH;
        let mut image = RgbaImage::new(WIDTH, HEIGHT);
        let scale = rusttype::Scale { x: HEIGHT as f32, y: HEIGHT as f32 };
        if cfg!(debug_assertions) {
            imageproc::drawing::draw_hollow_rect_mut(&mut image, Rect::at(0, 0).of_size(WIDTH, HEIGHT), Rgba([0u8, 0u8, 255u8, 255u8]));
        }
        imageproc::drawing::draw_text_mut(&mut image, Rgba([0u8, 0u8, 255u8, 255u8]), 0, 0, scale, self.font.borrow(), &self.label);
        let tex = Rc::new(Texture::from_image(
            factory,
            &image,
            &TextureSettings::new()
        ).unwrap());
        let mut sprite = Sprite::from_texture(tex.clone());
        sprite.set_anchor(0.0, 0.0);
        sprite.set_position(0.0, (HEIGHT * (self.offset as u32)) as f64);
        return sprite;
    }
}

impl Menu {
    pub fn new(entries: &Vec<String>, font: VGUIFont) -> Menu {
        let mut menu = Menu { cursor: 0, entries: Vec::new(), uuid_cursor: None, uuid_self: None };
        for (i, val) in entries.iter().enumerate() {
            let entry = MenuEntry{ offset:i, label: val.clone(), font: Rc::clone(&font) };
            menu.entries.push(entry);
        }
        return menu;
    }

    pub fn mv(&mut self, delta: i32) -> (uuid::Uuid, ai_behavior::Behavior<sprite::Animation>) {
        let m_delta = 
            if delta > 0 {
                std::cmp::min((self.entries.len() - self.cursor - 1) as i32, delta)
            }
            else {
                std::cmp::max(-(self.cursor as i32), delta)
            };
        self.cursor = ((self.cursor as i32) + m_delta) as usize;
        let new_y = (ENTRY_HEIGHT * (self.cursor as u32)) as f64;
        let shift = ai_behavior::Action(Ease(EaseFunction::CircularInOut, Box::new(MoveTo(0.16, 0.0, new_y))));
        (self.uuid_cursor.unwrap(), shift)
    }

    pub fn get(&self) -> Option<String> {
        if self.entries.len() > 0 {
            Some(self.entries[self.cursor].label.clone())
        }
        else { None }
    }
}

impl SpritePrototype for Menu {
    fn make_sprite<F, R>(&mut self, factory: &mut F) -> Sprite<Texture<R>>
        where F: gfx::Factory<R>, R: gfx::Resources
    {
        let tex_dummy = Rc::new(Texture::empty(factory).unwrap());
        let mut sprite = Sprite::from_texture(tex_dummy.clone());
        sprite.set_anchor(0.0, 0.0);
        
        const HEIGHT: u32 = ENTRY_HEIGHT;
        const WIDTH: u32 = COLUMN_WIDTH;
        let mut image = RgbaImage::new(WIDTH, HEIGHT);
        imageproc::drawing::draw_filled_rect_mut(&mut image, Rect::at(0, 0).of_size(WIDTH, HEIGHT), Rgba([220u8, 220u8, 250u8, 220u8]));
        let tex_cursor = Rc::new(Texture::from_image(
            factory,
            &image,
            &TextureSettings::new()
        ).unwrap());
        let mut sprite_cursor = Sprite::from_texture(tex_cursor.clone());
        sprite_cursor.set_anchor(0.0, 0.0);
        self.uuid_cursor = Some(sprite.add_child(sprite_cursor));

        for i in self.entries.iter_mut() {
            sprite.add_child(i.make_sprite(factory));
        }
        return sprite;
    }
}
