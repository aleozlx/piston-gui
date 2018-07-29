extern crate sprite;
extern crate find_folder;
extern crate rusttype;
extern crate gfx;

use std::fs::File;
use std::io::Read;
use std::rc::Rc;
use piston_window::*;
use sprite::*;
use rusttype::{Font, FontCollection, Scale};
use imageproc::drawing::draw_text_mut;
use image::{Rgba, RgbaImage};

pub struct MenuEntry<'a> {
    pub label: String,
    pub font: &'a Font<'a>
}

pub fn load_font<'a>(fname: &str) -> Result<Font<'static>, rusttype::Error> {
    let assets = find_folder::Search::ParentsThenKids(2, 2).for_folder("assets").unwrap();
    let ref fname_font = assets.join(fname);
    let mut fin_font = File::open(fname_font).expect(&format!("Cannot find font: {}", fname));
    let mut buffer = Vec::new();
    fin_font.read_to_end(&mut buffer).expect("IO error while reading the font.");
    FontCollection::from_bytes(buffer).unwrap().into_font()
}

impl<'a> MenuEntry<'a> {
    pub fn make_sprite<F, R>(&self, factory: &mut F) -> Sprite<Texture<R>>
        where F: gfx::Factory<R>, R: gfx::Resources
    {
        let mut image = RgbaImage::new(100, 100);
        let height = 32.0;
        let scale = Scale { x: height, y: height };
        draw_text_mut(&mut image, Rgba([0u8, 0u8, 255u8, 255u8]), 0, 0, scale, self.font, &self.label);
        let tex = Rc::new(Texture::from_image(
            factory,
            &image,
            &TextureSettings::new()
        ).unwrap());
        return Sprite::from_texture(tex.clone());
    }
}
