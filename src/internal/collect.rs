// +--------------------------------------------------------------------------+
// | Copyright 2016 Matthew D. Steele <mdsteele@alum.mit.edu>                 |
// |                                                                          |
// | This file is part of AHI.                                                |
// |                                                                          |
// | AHI is free software: you can redistribute it and/or modify it under     |
// | the terms of the GNU General Public License as published by the Free     |
// | Software Foundation, either version 3 of the License, or (at your        |
// | option) any later version.                                               |
// |                                                                          |
// | AHI is distributed in the hope that it will be useful, but WITHOUT ANY   |
// | WARRANTY; without even the implied warranty of MERCHANTABILITY or        |
// | FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License    |
// | for details.                                                             |
// |                                                                          |
// | You should have received a copy of the GNU General Public License along  |
// | with AHI.  If not, see <http://www.gnu.org/licenses/>.                   |
// +--------------------------------------------------------------------------+

use internal::color::Color;
use internal::image::Image;
use internal::palette::Palette;
use internal::util::{read_exactly, read_header_uint};
use std::io::{self, Error, ErrorKind, Read, Write};

// ========================================================================= //

// TODO: Add support for ahi1 format:
// - Allow storing palettes as well as images
// - Allow storing a different width/height for each image
// - Allow storing string tags for each image
// - Allow storing a vector of metadata i32s for each image

// TODO: Support BHI format, which is a binary encoding of an AHI file, with
// compressed image data.

/// A collection of palettes and/or images.
pub struct Collection {
    /// The palettes in this collection.
    pub palettes: Vec<Palette>,
    /// The images in this collection.
    pub images: Vec<Image>,
}

impl Collection {
    /// Returns a new, empty collection.
    pub fn new() -> Collection {
        Collection {
            palettes: Vec::new(),
            images: Vec::new(),
        }
    }

    /// Reads a collection from an AHI file.
    pub fn read<R: Read>(mut reader: R) -> io::Result<Collection> {
        try!(read_exactly(reader.by_ref(), b"ahi"));
        let version = try!(read_header_uint(reader.by_ref(), b' '));
        if version != 0 {
            let msg = format!("unsupported AHI version: {}", version);
            return Err(Error::new(ErrorKind::InvalidData, msg));
        }
        try!(read_exactly(reader.by_ref(), b"w"));
        let width = try!(read_header_uint(reader.by_ref(), b' '));
        try!(read_exactly(reader.by_ref(), b"h"));
        let height = try!(read_header_uint(reader.by_ref(), b' '));
        try!(read_exactly(reader.by_ref(), b"n"));
        let num_images = try!(read_header_uint(reader.by_ref(), b'\n'));
        let mut images = Vec::with_capacity(num_images as usize);
        let mut row_buffer = vec![0u8; width as usize];
        for _ in 0..num_images {
            try!(read_exactly(reader.by_ref(), b"\n"));
            let mut pixels = Vec::with_capacity((width * height) as usize);
            for _ in 0..height {
                try!(reader.read_exact(&mut row_buffer));
                for &byte in &row_buffer {
                    pixels.push(try!(Color::from_byte(byte)));
                }
                try!(read_exactly(reader.by_ref(), b"\n"));
            }
            images.push(Image {
                width: width,
                height: height,
                pixels: pixels.into_boxed_slice(),
            })
        }
        Ok(Collection {
            palettes: Vec::new(),
            images: images,
        })
    }

    /// Writes a collection to an AHI file.  Returns an error if the images
    /// aren't all the same dimensions.
    pub fn write<W: Write>(&self, mut writer: W) -> io::Result<()> {
        let (width, height) = if self.images.is_empty() {
            (0, 0)
        } else {
            (self.images[0].width, self.images[0].height)
        };
        try!(write!(writer,
                    "ahi0 w{} h{} n{}\n",
                    width,
                    height,
                    self.images.len()));
        for image in self.images.iter() {
            if image.width != width || image.height != height {
                let msg = format!("images must all have the same dimensions \
                                   (found {}x{} instead of {}x{})",
                                  image.width,
                                  image.height,
                                  width,
                                  height);
                return Err(Error::new(ErrorKind::InvalidInput, msg));
            }
            try!(write!(writer, "\n"));
            for row in 0..height {
                for col in 0..width {
                    let color = image.pixels[(row * width + col) as usize];
                    try!(writer.write_all(&[color.to_byte()]));
                }
                try!(write!(writer, "\n"));
            }
        }
        Ok(())
    }
}

// ========================================================================= //

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_zero_images() {
        let input: &[u8] = b"ahi0 w0 h0 n0";
        let collection = Collection::read(input).unwrap();
        assert_eq!(collection.images.len(), 0);
    }

    #[test]
    fn read_two_images() {
        let input: &[u8] = b"ahi0 w2 h2 n2\n\
                             \n\
                             20\n\
                             5D\n\
                             \n\
                             E0\n\
                             0E\n";
        let collection = Collection::read(input).unwrap();
        assert_eq!(collection.images.len(), 2);
        assert_eq!(collection.images[0][(0, 1)], Color::C5);
        assert_eq!(collection.images[1][(0, 0)], Color::Ce);
    }

    #[test]
    fn write_zero_images() {
        let mut output = Vec::<u8>::new();
        Collection::new().write(&mut output).unwrap();
        assert_eq!(&output as &[u8], b"ahi0 w0 h0 n0\n");
    }

    #[test]
    fn write_two_images() {
        let mut collection = Collection::new();
        let mut image0 = Image::new(2, 2);
        image0[(0, 0)] = Color::C2;
        image0[(0, 1)] = Color::C5;
        image0[(1, 1)] = Color::Cd;
        collection.images.push(image0);
        let mut image1 = Image::new(2, 2);
        image1[(0, 0)] = Color::Ce;
        image1[(1, 1)] = Color::Ce;
        collection.images.push(image1);
        let mut output = Vec::<u8>::new();
        collection.write(&mut output).unwrap();
        assert_eq!(&output as &[u8],
                   b"ahi0 w2 h2 n2\n\
                     \n\
                     20\n\
                     5D\n\
                     \n\
                     E0\n\
                     0E\n");
    }
}

// ========================================================================= //
