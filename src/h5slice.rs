extern crate image;
extern crate flate2;

use std::{mem, slice};
use std::io::prelude::*;
use std::net::TcpStream;
use flate2::read::GzDecoder;
use std::ops::Deref;
use image::Pixel;

// fn as_rgba<Image, Pixel>(im_input: Image, size: (u32, u32)) -> image::RgbaImage
//     where Image: image::GenericImage<Pixel=Pixel>,
//         Pixel: image::Pixel
// {
//     image::ImageBuffer::from_fn(size.0, size.1,
//         |x, y| { im_input.get_pixel(x, y).to_rgba() })
// }

pub fn get_one() -> Option<image::RgbaImage> {
    // TODO handle connection error gracefully.
    let mut stream = TcpStream::connect("localhost:8000").expect("Cannot connect to stream.");
    let mut buffer_in = Vec::with_capacity(8<<10);
    let mut buffer_out = Vec::with_capacity(4<<20);
    let _ = stream.write("/home/alex/datasets/ucm-sample.h5\t/source/images\t13\tf4\n".as_bytes());
    if let Ok(n) = stream.read_to_end(&mut buffer_in) {
        println!("Read {} bytes from network.", n);
        let mut decoder = GzDecoder::new(&buffer_in[..]);
        match decoder.read_to_end(&mut buffer_out) {
            Ok(n) => {
                println!("Decompressed into {} bytes.", n);
                let im_raw: Vec<u8> = unsafe {
                    slice::from_raw_parts(buffer_out.as_ptr() as *const f32, n/mem::size_of::<f32>())
                }.into_iter().map(|x| {(x+100.0) as u8}).collect();

                let im_rgb: image::RgbImage = image::ImageBuffer::from_raw(224, 224, im_raw).expect("Error when constructing RgbImage");
                let im_dynamic = image::DynamicImage::ImageRgb8(im_rgb);
                return Some(im_dynamic.to_rgba());
                // let im_rgba: image::RgbaImage = image::ImageBuffer::from_fn(224, 224,
                //     |x, y| { im_rgb.get_pixel(x, y).to_rgba() });
                // return Some(im_rgba);
            },
            Err(err) => { println!("Error: {}", err); }
        }
    }
    return None;
}
