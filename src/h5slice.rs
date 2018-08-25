extern crate image;
extern crate flate2;

use std::{mem, slice};
use std::io::prelude::*;
use std::net::TcpStream;
use flate2::read::GzDecoder;
use std::string::ToString;

pub enum Dtype {
    I4, I8, F4, F8
}

pub struct H5URI {
    pub path: String,
    pub h5path: String,
    pub query: String,
    pub dtype: Dtype
}

impl ToString for Dtype {
    fn to_string(&self) -> String {
        match self {
            Dtype::I4 => String::from("i4"),
            Dtype::I8 => String::from("i8"),
            Dtype::F4 => String::from("f4"),
            Dtype::F8 => String::from("f8"),
        }
    }
}

impl ToString for H5URI {
    fn to_string(&self) -> String {
        [self.path.clone(), self.h5path.clone(), self.query.clone(), self.dtype.to_string()].join("\t")
    }
}

pub fn get_one(uri: H5URI, resolution: (u32, u32)) -> Option<image::RgbaImage> {
    let mut stream = TcpStream::connect("localhost:8000").ok()?;
    let mut buffer_in = Vec::with_capacity(8<<10);
    let mut buffer_out = Vec::with_capacity(4<<20);
    let _ = stream.write(uri.to_string().as_bytes());
    let n = stream.read_to_end(&mut buffer_in).ok()?;
    // TODO use logging instead
    if cfg!(debug_assertions) { println!("Read {} bytes from network.", n); }
    let mut decoder = GzDecoder::new(&buffer_in[..]);
    let n = decoder.read_to_end(&mut buffer_out).ok()?;
    if cfg!(debug_assertions) { println!("Decompressed into {} bytes.", n); }
    let im_rgb = image::ImageBuffer::from_raw(resolution.0, resolution.1,
        unsafe { slice::from_raw_parts(buffer_out.as_ptr() as *const f32, n/mem::size_of::<f32>()) }
            .into_iter().map(|x| {(x+100.0) as u8}).collect())?;
    // A copy is done by format conversion, so lifetime is correct.
    Some(image::DynamicImage::ImageRgb8(im_rgb).to_rgba())
}
