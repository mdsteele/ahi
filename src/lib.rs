//! A library for encoding/decoding ASCII Hex Image (.ahi) files.
//!
//! # The AHI format
//!
//! ASCII Hex Image (AHI) is a file format for storing collections of small,
//! 16-color images as an ASCII text file.  It is intended for storing sprites
//! for games or other graphical applications, in a way that makes changes to
//! image files result in (semi-)human-readable VCS diffs.
//!
//! Here's what a typical .ahi file looks like:
//!
//! ```text
//! ahi0 w14 h5 n2
//!
//! 0000000000000FFF0000
//! FFFFFFFFFFFFFF11FF00
//! F11111111111111111FF
//! FFFFFFFFFFFFFF11FF00
//! 0000000000000FFF0000
//!
//! 0000FFF0000000000000
//! 00FF11FFFFFFFFFFFFFF
//! FF11111111111111111F
//! 00FF11FFFFFFFFFFFFFF
//! 0000FFF0000000000000
//! ```
//!
//! The start of the .ahi file is the _header line_, which has the form
//! `ahi<version> w<width> h<height> n<num_images>`, where each of the four
//! fields is a hexadecimal number.  So, the above file is AHI version 0
//! (currently the only valid version), and contains two 20x5-pixel images (all
//! the images in a single file must have the same dimensions).
//!
//! After the header line comes the images, which are separated from the header
//! line and from each other by double-newlines.  Each image has one text line
//! per pixel row, with one hex digit per pixel.  Each pixel row line
//! (including the last one in the file) must be terminated by a newline.
//!
//! To map from hex digits to colors, treat each hex digit as a four-digit
//! binary number; the 1's place controls brightness, the 2's place controls
//! red, the 4's place controls green, and the 8's place controls blue.  For
//! example, `3 = 0b0011` is full-brightness red; `A = 0b1010` is
//! half-brightness magenta; `F = 0b1111` is white; and `E = 0x1110` is
//! "half-brightness white", i.e. gray.  Since "full-brightness black"
//! (`1 = 0b0001`) and "half-brightness black" (`0 = 0b0000`) would be the same
//! color, instead color `0` is special-cased to be transparent (and color `1`
//! is black).

#![warn(missing_docs)]

use std::io::{self, Error, ErrorKind, Write};

/// Represents a pixel color for an ASCII Hex Image.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Color {
    /// Completely transparent (R=0, G=0, B=0, A=0).
    Transparent,
    /// Solid black (R=0, G=0, B=0, A=255).
    Black,
    /// Half-brightness red (R=127, G=0, B=0, A=255).
    DarkRed,
    /// Full-brightness red (R=255, G=0, B=0, A=255).
    Red,
    /// Half-brightness green (R=0, G=127, B=0, A=255).
    DarkGreen,
    /// Full-brightness green (R=0, G=255, B=0, A=255).
    Green,
    /// Half-brightness yellow (R=127, G=127, B=0, A=255).
    DarkYellow,
    /// Full-brightness yellow (R=255, G=255, B=0, A=255).
    Yellow,
    /// Half-brightness blue (R=0, G=0, B=127, A=255).
    DarkBlue,
    /// Full-brightness blue (R=0, G=0, B=255, A=255).
    Blue,
    /// Half-brightness magenta (R=127, G=0, B=127, A=255).
    DarkMagenta,
    /// Full-brightness magenta (R=255, G=0, B=255, A=255).
    Magenta,
    /// Half-brightness cyan (R=0, G=127, B=127, A=255).
    DarkCyan,
    /// Full-brightness cyan (R=0, G=255, B=255, A=255).
    Cyan,
    /// Gray, i.e. half-brightness white  (R=127, G=127, B=127, A=255).
    Gray,
    /// Solid white (R=255, G=255, B=255, A=255).
    White,
}

impl Color {
    /// Returns the color's RGBA values.
    ///
    /// # Examples
    /// ```
    /// use ahi::Color;
    /// assert_eq!(Color::Transparent.rgba(), (0, 0, 0, 0));
    /// assert_eq!(Color::DarkYellow.rgba(), (127, 127, 0, 255));
    /// assert_eq!(Color::White.rgba(), (255, 255, 255, 255));
    /// ```
    pub fn rgba(self) -> (u8, u8, u8, u8) {
        match self {
            Color::Transparent => (0, 0, 0, 0),
            Color::Black => (0, 0, 0, 255),
            Color::DarkRed => (127, 0, 0, 255),
            Color::Red => (255, 0, 0, 255),
            Color::DarkGreen => (0, 127, 0, 255),
            Color::Green => (0, 255, 0, 255),
            Color::DarkYellow => (127, 127, 0, 255),
            Color::Yellow => (255, 255, 0, 255),
            Color::DarkBlue => (0, 0, 127, 255),
            Color::Blue => (0, 0, 255, 255),
            Color::DarkMagenta => (127, 0, 127, 255),
            Color::Magenta => (255, 0, 255, 255),
            Color::DarkCyan => (0, 127, 127, 255),
            Color::Cyan => (0, 255, 255, 255),
            Color::Gray => (127, 127, 127, 255),
            Color::White => (255, 255, 255, 255),
        }
    }

    fn to_byte(self) -> u8 {
        match self {
            Color::Transparent => b'0',
            Color::Black => b'1',
            Color::DarkRed => b'2',
            Color::Red => b'3',
            Color::DarkGreen => b'4',
            Color::Green => b'5',
            Color::DarkYellow => b'6',
            Color::Yellow => b'7',
            Color::DarkBlue => b'8',
            Color::Blue => b'9',
            Color::DarkMagenta => b'A',
            Color::Magenta => b'B',
            Color::DarkCyan => b'C',
            Color::Cyan => b'D',
            Color::Gray => b'E',
            Color::White => b'F',
        }
    }
}

/// Represents a single ASCII Hex Image.
#[derive(Clone)]
pub struct Image {
    width: u32,
    height: u32,
    pixels: Box<[Color]>,
}

impl Image {
    /// Constructs a new image with all pixels transparent.
    pub fn new(width: u32, height: u32) -> Image {
        let num_pixels = (width * height) as usize;
        return Image {
            width: width,
            height: height,
            pixels: vec![Color::Transparent; num_pixels].into_boxed_slice(),
        };
    }

    /// Returns the width of the image, in pixels.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Returns the height of the image, in pixels.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Returns a byte array containing RGBA-order data for the image pixels,
    /// in row-major order.
    pub fn rgba_data(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(self.pixels.len() * 4);
        for pixel in self.pixels.iter() {
            let (r, g, b, a) = pixel.rgba();
            data.push(r);
            data.push(g);
            data.push(b);
            data.push(a);
        }
        data
    }

    /// Writes a group of images to a file.  Returns an error if the images
    /// aren't all the same dimensions.
    pub fn write_all<W: Write>(mut writer: W,
                               images: &[Image])
                               -> io::Result<()> {
        let (width, height) = if images.is_empty() {
            (0, 0)
        } else {
            (images[0].width, images[0].height)
        };
        try!(write!(writer,
                    "ahi0 w{:X} h{:X} n{:X}\n",
                    width,
                    height,
                    images.len()));
        for image in images {
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

impl std::ops::Index<(u32, u32)> for Image {
    type Output = Color;
    fn index(&self, index: (u32, u32)) -> &Color {
        let (col, row) = index;
        if col >= self.width || row >= self.height {
            panic!("index out of range");
        }
        &self.pixels[(row * self.width + col) as usize]
    }
}

impl std::ops::IndexMut<(u32, u32)> for Image {
    fn index_mut(&mut self, index: (u32, u32)) -> &mut Color {
        let (col, row) = index;
        if col >= self.width || row >= self.height {
            panic!("index out of range");
        }
        &mut self.pixels[(row * self.width + col) as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_rgba_data() {
        let mut image = Image::new(2, 2);
        image[(0, 0)] = Color::DarkRed;
        image[(0, 1)] = Color::Green;
        image[(1, 1)] = Color::Cyan;
        assert_eq!(image.rgba_data(),
                   vec![127, 0, 0, 255, 0, 0, 0, 0, 0, 255, 0, 255, 0, 255,
                        255, 255]);
    }

    #[test]
    fn write_zero_images() {
        let mut output = Vec::<u8>::new();
        Image::write_all(&mut output, &[]).expect("failed to write images");
        assert_eq!(&output as &[u8], b"ahi0 w0 h0 n0\n");
    }

    #[test]
    fn write_two_images() {
        let mut image0 = Image::new(2, 2);
        image0[(0, 0)] = Color::DarkRed;
        image0[(0, 1)] = Color::Green;
        image0[(1, 1)] = Color::Cyan;
        let mut image1 = Image::new(2, 2);
        image1[(0, 0)] = Color::Gray;
        image1[(1, 1)] = Color::Gray;
        let mut output = Vec::<u8>::new();
        Image::write_all(&mut output, &[image0, image1])
            .expect("failed to write images");
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
