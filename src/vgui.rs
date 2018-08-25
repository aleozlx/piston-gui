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
pub type TexImage = RgbaImage;

const ENTRY_HEIGHT: u32 = 32;
const COLUMN_WIDTH: u32 = 315;

pub trait SpritePrototype {
    fn make_sprite<F, R>(&mut self, factory: &mut F) -> Sprite<Texture<R>>
        where F: gfx::Factory<R>, R: gfx::Resources;
}

pub trait MenuAdapter<T> {
    fn adapt(group: &T, font: VGUIFont) -> Menu;
}

pub trait Layout {
    fn view_size(&self) -> (u32, u32);
    fn item_size(&self) -> (u32, u32);
}

pub trait Paginatable {
    fn page_capacity(&self) -> usize;
}

pub struct MenuEntry {
    pub label: String,
    pub font: VGUIFont,
    pub tag: Option<String>,
    offset: usize
}

impl SpritePrototype for MenuEntry {
    fn make_sprite<F, R>(&mut self, factory: &mut F) -> Sprite<Texture<R>>
        where F: gfx::Factory<R>, R: gfx::Resources
    {
        const HEIGHT: u32 = ENTRY_HEIGHT;
        const WIDTH: u32 = COLUMN_WIDTH;
        let mut image = RgbaImage::new(WIDTH, HEIGHT);
        let scale = rusttype::Scale { x: HEIGHT as f32, y: HEIGHT as f32 };
        let scale_tag = rusttype::Scale { x: scale.x * 0.6, y: scale.y * 0.6 };
        if cfg!(debug_assertions) {
            imageproc::drawing::draw_hollow_rect_mut(&mut image, Rect::at(0, 0).of_size(WIDTH, HEIGHT), Rgba([0u8, 0u8, 255u8, 255u8]));
        }
        imageproc::drawing::draw_text_mut(&mut image, Rgba([0u8, 0u8, 255u8, 255u8]), 0, 0, scale, self.font.borrow(), &self.label);
        if let Some(tag) = &self.tag {
            imageproc::drawing::draw_text_mut(&mut image, Rgba([0u8, 0u8, 255u8, 255u8]), WIDTH - 45, 0, scale_tag, self.font.borrow(), tag);
        }
        let mut sprite = sprite_from_image(&image, factory);
        sprite.set_anchor(0.0, 0.0);
        sprite.set_position(0.0, (HEIGHT * (self.offset as u32)) as f64);
        return sprite;
    }
}

pub struct Menu {
    pub entries: Vec<MenuEntry>,
    
    cursor: usize,
    uuid_cursor: Option<uuid::Uuid>,
    pub uuid_self: Option<uuid::Uuid>
}

impl Menu {
    pub fn new(entries: &Vec<String>, font: VGUIFont) -> Menu {
        let mut menu = Menu { cursor: 0, entries: Vec::new(), uuid_cursor: None, uuid_self: None };
        for (i, val) in entries.iter().enumerate() {
            let entry = MenuEntry{ offset:i, label: val.clone(), font: Rc::clone(&font), tag: None };
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
        let mut sprite_cursor = sprite_from_image(&image, factory);
        sprite_cursor.set_anchor(0.0, 0.0);
        self.uuid_cursor = Some(sprite.add_child(sprite_cursor));

        for i in self.entries.iter_mut() {
            sprite.add_child(i.make_sprite(factory));
        }
        return sprite;
    }
}

pub fn load_font(fname: &str) -> Result<VGUIFont, rusttype::Error> {
    let assets = find_folder::Search::ParentsThenKids(2, 2).for_folder("assets").unwrap();
    let ref fname_font = assets.join(fname);
    let mut fin_font = File::open(fname_font).expect(&format!("Cannot find font: {}", fname));
    let mut buffer = Vec::new();
    fin_font.read_to_end(&mut buffer).expect("IO error while reading the font.");
    Ok(Rc::new(rusttype::FontCollection::from_bytes(buffer).unwrap().into_font()?))
}

pub struct FlowLayout {
    pub view_size: (u32, u32),
    pub item_size: (u32, u32),
    pub spacing: u32
}

impl FlowLayout {
    #[allow(dead_code)]
    pub fn new() -> FlowLayout {
        FlowLayout {
            view_size: (1920, 1080),
            item_size: (100, 100),
            spacing: 6
        }
    }

    #[allow(dead_code)]
    pub fn view_size(sz: (f64, f64)) -> FlowLayout {
        FlowLayout {
            view_size: (sz.0 as u32, sz.1 as u32),
            item_size: (100, 100),
            spacing: 6
        }
    }

    pub fn get_items_per_row(&self) -> usize {
        (self.view_size.0 / (self.item_size.0 + self.spacing)) as usize
    }

    pub fn get_items_per_col(&self) -> usize {
        (self.view_size.1 / (self.item_size.1 + self.spacing)) as usize
    }

    pub fn get_coordinate(&self, idx: usize) -> (f64, f64) {
        let items_per_row = self.get_items_per_row();
        let row = idx / items_per_row;
        let col = idx % items_per_row;
        (col as f64 * (self.item_size.1 + self.spacing) as f64,
         row as f64 * (self.item_size.0 + self.spacing) as f64)
    }
}

impl Layout for FlowLayout {
    fn view_size(&self) -> (u32, u32) {
        self.view_size
    }

    fn item_size(&self) -> (u32, u32) {
        self.item_size
    }
}

impl Paginatable for FlowLayout {
    fn page_capacity(&self) -> usize {
        self.get_items_per_row() * self.get_items_per_col()
    }
}

impl SpritePrototype for FlowLayout {
    fn make_sprite<F, R>(&mut self, factory: &mut F) -> Sprite<Texture<R>>
        where F: gfx::Factory<R>, R: gfx::Resources
    {        
        let (width, height) = self.view_size();
        let mut image = RgbaImage::new(width, height);
        let mut sprite;
        if cfg!(debug_assertions) {
            imageproc::drawing::draw_hollow_rect_mut(&mut image, Rect::at(0, 0).of_size(width, height), Rgba([0u8, 0u8, 255u8, 255u8]));
            sprite = sprite_from_image(&image, factory);
        }
        else {
            sprite = Sprite::from_texture(Rc::new(Texture::empty(factory).unwrap()));
        }
        sprite.set_anchor(0.0, 0.0);
        return sprite;
    }
}

pub struct StatusBar {
    pub label: String,
    pub font: VGUIFont,
    pub color: Rgba<u8>
}

impl StatusBar {
    pub fn update<F, R>(&mut self, new_label: String, sprite: &mut Sprite<Texture<R>>, factory: &mut F)
        where F: gfx::Factory<R>, R: gfx::Resources
    {
        self.label = new_label;
        sprite.set_texture(
            Rc::new(Texture::from_image(
            factory,
            &self.draw(),
            &TextureSettings::new()
        ).unwrap()));
    }

    fn draw(&mut self) -> TexImage {
        const HEIGHT: u32 = ENTRY_HEIGHT;
        const WIDTH: u32 = COLUMN_WIDTH * 3;
        let mut image = RgbaImage::new(WIDTH, HEIGHT);
        let scale = rusttype::Scale { x: HEIGHT as f32, y: HEIGHT as f32 };
        if cfg!(debug_assertions) {
            imageproc::drawing::draw_hollow_rect_mut(&mut image, Rect::at(0, 0).of_size(WIDTH, HEIGHT), Rgba([0u8, 0u8, 255u8, 255u8]));
        }
        imageproc::drawing::draw_text_mut(&mut image, self.color, 0, 0, scale, self.font.borrow(), &self.label);
        return image;
    }
}

impl SpritePrototype for StatusBar {
    fn make_sprite<F, R>(&mut self, factory: &mut F) -> Sprite<Texture<R>>
        where F: gfx::Factory<R>, R: gfx::Resources
    {
        let mut sprite = Sprite::from_texture(Rc::new(Texture::from_image(
            factory,
            &self.draw(),
            &TextureSettings::new()
        ).unwrap()));
        sprite.set_anchor(0.0, 0.0);
        return sprite;
    }
}

pub struct Pagnator {
    pub item_range: std::ops::Range<usize>,
    pub page_size: usize,
    pub page_current: usize
}

impl Pagnator {
    pub fn new(o: &Paginatable, n_items: usize) -> Pagnator {
        Pagnator { item_range: 0..n_items, page_size: o.page_capacity(), page_current: 0 }
    }

    pub fn get_range(&self, page_size: usize) -> Option<std::ops::Range<usize>> {
        let p = self.page_current * self.page_size;
        if self.item_range.start <= p && p < self.item_range.end {
            let end = std::cmp::min(p+self.page_size, self.item_range.end);
            Some(p..end)
        }
        else { None }
    }
}

pub fn sprite_from_image<F, R>(im: &TexImage, factory: &mut F) -> Sprite<Texture<R>>
    where F: gfx::Factory<R>, R: gfx::Resources
{
    Sprite::from_texture(Rc::new(Texture::from_image(factory, im, &TextureSettings::new()).unwrap()))
}
