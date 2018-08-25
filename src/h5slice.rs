extern crate image;
extern crate flate2;

use std::{mem, slice};
use std::io::prelude::*;
use std::net::TcpStream;
use flate2::read::GzDecoder;
use std::string::ToString;
use std::hash::Hash;
use std::cmp::Eq;
use std::collections::HashMap;

pub type TexImage = image::RgbaImage;

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum Dtype {
    I4, I8, F4, F8
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

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum Query {
    One(usize),
    Range(usize, usize),
    Batch(usize, usize)
}

impl ToString for Query {
    fn to_string(&self) -> String {
        match self {
            Query::One(idx) => idx.to_string(),
            Query::Range(a, b) => format!("{}:{}", a, b),
            Query::Batch(idx, len) => format!("{}:{}", idx, idx+len)
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct H5URI {
    pub path: String,
    pub h5path: String,
    pub query: Query,
    pub dtype: Dtype
}

impl ToString for H5URI {
    fn to_string(&self) -> String {
        [self.path.clone(), self.h5path.clone(), self.query.to_string(), self.dtype.to_string()].join("\t")
    }
}

pub struct H5Cache {
    buffer: HashMap<H5URI, TexImage>
}

impl H5Cache {
    pub fn new() -> H5Cache {
        H5Cache { buffer: HashMap::with_capacity(60) }
    }

    pub fn request_one(&mut self, uri: &H5URI, resolution: &(u32, u32)) -> Option<&'_ mut TexImage> {
        if self.buffer.contains_key(uri) {
            Some(self.buffer.get_mut(uri).unwrap())
        }
        else {
            self.fetch_one(uri, resolution)
        }
    }

    fn fetch_one(&mut self, uri: &H5URI, resolution: &(u32, u32)) -> Option<&'_ mut TexImage> {
        fn __fetch_one(uri: &H5URI, resolution: &(u32, u32)) -> Option<TexImage> {
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
        
        self.buffer.insert(uri.clone(), __fetch_one(uri, resolution)?);
        Some(self.buffer.get_mut(uri).unwrap())
    }
}
