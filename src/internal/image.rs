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
use internal::palette::Palette;
use internal::util;
use std::cmp::{max, min};
use std::io::{self, Read, Write};
use std::ops::{Index, IndexMut};

// ========================================================================= //

/// Represents a single ASCII Hex Image.
#[derive(Clone)]
pub struct Image {
    pub(crate) tag: String,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) pixels: Box<[Color]>,
}

impl Image {
    /// Constructs a new image with all pixels transparent.
    pub fn new(width: u32, height: u32) -> Image {
        let num_pixels = (width * height) as usize;
        return Image {
            tag: String::new(),
            width: width,
            height: height,
            pixels: vec![Color::C0; num_pixels].into_boxed_slice(),
        };
    }

    /// Returns the string tag for this image (or empty string if it doesn't
    /// have one).
    pub fn tag(&self) -> &str { &self.tag }

    /// Sets the string tag for this image.
    pub fn set_tag(&mut self, tag: String) { self.tag = tag; }

    /// Returns the width of the image, in pixels.
    pub fn width(&self) -> u32 { self.width }

    /// Returns the height of the image, in pixels.
    pub fn height(&self) -> u32 { self.height }

    /// Returns a byte array containing RGBA-order data for the image pixels,
    /// in row-major order.
    pub fn rgba_data(&self, palette: &Palette) -> Vec<u8> {
        let mut data = Vec::with_capacity(self.pixels.len() * 4);
        for &color in self.pixels.iter() {
            let (r, g, b, a) = palette[color];
            data.push(r);
            data.push(g);
            data.push(b);
            data.push(a);
        }
        data
    }

    /// Sets all pixels in the image to transparent.
    pub fn clear(&mut self) {
        self.pixels = vec![Color::C0; self.pixels.len()]
                          .into_boxed_slice();
    }

    /// Sets all pixels in the specified rectangle to the given color.
    pub fn fill_rect(&mut self,
                     x: i32,
                     y: i32,
                     w: u32,
                     h: u32,
                     color: Color) {
        let start_row = min(max(0, y) as u32, self.height);
        let end_row = min(max(0, y + h as i32) as u32, self.height);
        let start_col = min(max(0, x) as u32, self.width);
        let end_col = min(max(0, x + w as i32) as u32, self.width);
        for row in start_row..end_row {
            for col in start_col..end_col {
                self[(col, row)] = color;
            }
        }
    }

    /// Draws pixels from `src` onto this image, placing the top-left corner of
    /// `src` at coordinates `(x, y)`.
    pub fn draw(&mut self, src: &Image, x: i32, y: i32) {
        let src_start_row = min(max(0, -y) as u32, src.height);
        let src_start_col = min(max(0, -x) as u32, src.width);
        let dest_start_row = min(max(0, y) as u32, self.height);
        let dest_start_col = min(max(0, x) as u32, self.width);
        let num_rows = min(src.height - src_start_row,
                           self.height - dest_start_row);
        let num_cols = min(src.width - src_start_col,
                           self.width - dest_start_col);
        for row in 0..num_rows {
            for col in 0..num_cols {
                let color = src[(src_start_col + col, src_start_row + row)];
                if color != Color::C0 {
                    self[(dest_start_col + col, dest_start_row + row)] = color;
                }
            }
        }
    }

    /// Returns a copy of the image that has been flipped horizontally.
    pub fn flip_horz(&self) -> Image {
        let mut pixels = Vec::with_capacity(self.pixels.len());
        for row in 0..self.height {
            let offset = row * self.width;
            for col in 0..self.width {
                let index = offset + self.width - col - 1;
                pixels.push(self.pixels[index as usize]);
            }
        }
        Image {
            tag: self.tag.clone(),
            width: self.width,
            height: self.height,
            pixels: pixels.into_boxed_slice(),
        }
    }

    /// Returns a copy of the image that has been flipped vertically.
    pub fn flip_vert(&self) -> Image {
        let mut pixels = Vec::with_capacity(self.pixels.len());
        for row in 0..self.height {
            let offset = (self.height - row - 1) * self.width;
            for col in 0..self.width {
                let index = offset + col;
                pixels.push(self.pixels[index as usize]);
            }
        }
        Image {
            tag: self.tag.clone(),
            width: self.width,
            height: self.height,
            pixels: pixels.into_boxed_slice(),
        }
    }

    /// Returns a copy of the image that has been rotated 90 degrees clockwise.
    pub fn rotate_cw(&self) -> Image {
        let mut pixels = Vec::with_capacity(self.pixels.len());
        for row in 0..self.width {
            for col in 0..self.height {
                let index = self.width * (self.height - col - 1) + row;
                pixels.push(self.pixels[index as usize]);
            }
        }
        Image {
            tag: self.tag.clone(),
            width: self.height,
            height: self.width,
            pixels: pixels.into_boxed_slice(),
        }
    }

    /// Returns a copy of the image that has been rotated 90 degrees
    /// counterclockwise.
    pub fn rotate_ccw(&self) -> Image {
        let mut pixels = Vec::with_capacity(self.pixels.len());
        for row in 0..self.width {
            for col in 0..self.height {
                let index = self.width * col + (self.width - row - 1);
                pixels.push(self.pixels[index as usize]);
            }
        }
        Image {
            tag: self.tag.clone(),
            width: self.height,
            height: self.width,
            pixels: pixels.into_boxed_slice(),
        }
    }

    /// Returns a copy of the image, cropped to the given size.  If the new
    /// width/height is less than the current value, pixels are removed from
    /// the right/bottom; if the new width/height is greater than the current
    /// value, extra transparent pixels are added to the right/bottom.
    pub fn crop(&self, new_width: u32, new_height: u32) -> Image {
        let mut new_image = Image::new(new_width, new_height);
        new_image.draw(self, 0, 0);
        new_image
    }

    pub(crate) fn read<R: Read>(mut reader: R, width: u32, height: u32)
                                -> io::Result<Image> {
        let mut pixels = Vec::with_capacity((width * height) as usize);
        let mut row_buffer = vec![0u8; width as usize];
        for _ in 0..height {
            reader.read_exact(&mut row_buffer)?;
            for &byte in &row_buffer {
                pixels.push(Color::from_byte(byte)?);
            }
            util::read_exactly(reader.by_ref(), b"\n")?;
        }
        Ok(Image {
            tag: String::new(),
            width: width,
            height: height,
            pixels: pixels.into_boxed_slice(),
        })
    }

    pub(crate) fn write<W: Write>(&self, mut writer: W) -> io::Result<()> {
        for row in 0..self.height {
            for col in 0..self.width {
                let index = row * self.width + col;
                let color = self.pixels[index as usize];
                writer.write_all(&[color.to_byte()])?;
            }
            write!(writer, "\n")?;
        }
        Ok(())
    }
}

impl Index<(u32, u32)> for Image {
    type Output = Color;
    fn index(&self, index: (u32, u32)) -> &Color {
        let (col, row) = index;
        if col >= self.width || row >= self.height {
            panic!("index out of range");
        }
        &self.pixels[(row * self.width + col) as usize]
    }
}

impl IndexMut<(u32, u32)> for Image {
    fn index_mut(&mut self, index: (u32, u32)) -> &mut Color {
        let (col, row) = index;
        if col >= self.width || row >= self.height {
            panic!("index out of range");
        }
        &mut self.pixels[(row * self.width + col) as usize]
    }
}

// ========================================================================= //

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_rgba_data() {
        let mut image = Image::new(2, 2);
        image[(0, 0)] = Color::C2;
        image[(0, 1)] = Color::C5;
        image[(1, 1)] = Color::Cd;
        assert_eq!(image.rgba_data(Palette::default()),
                   vec![127, 0, 0, 255, 0, 0, 0, 0, 0, 255, 0, 255, 0, 255,
                        255, 255]);
    }

    #[test]
    fn clear_image() {
        let mut image = Image::new(2, 2);
        image[(1, 0)] = Color::C2;
        image[(1, 1)] = Color::C5;
        image.clear();
        assert_eq!(image[(1, 0)], Color::C0);
        assert_eq!(image[(1, 1)], Color::C0);
    }

    #[test]
    fn flip_image_horz() {
        let mut image = Image::new(2, 2);
        image[(0, 1)] = Color::C3;
        image[(1, 1)] = Color::C5;
        let image = image.flip_horz();
        assert_eq!(image[(0, 1)], Color::C5);
        assert_eq!(image[(1, 1)], Color::C3);
    }

    #[test]
    fn flip_image_vert() {
        let mut image = Image::new(2, 2);
        image[(1, 0)] = Color::C3;
        image[(1, 1)] = Color::C5;
        let image = image.flip_vert();
        assert_eq!(image[(1, 0)], Color::C5);
        assert_eq!(image[(1, 1)], Color::C3);
    }

    #[test]
    fn rotate_image_cw() {
        let mut image = Image::new(4, 2);
        image[(1, 0)] = Color::C3;
        image[(1, 1)] = Color::C5;
        let image = image.rotate_cw();
        assert_eq!(2, image.width());
        assert_eq!(4, image.height());
        assert_eq!(image[(1, 1)], Color::C3);
        assert_eq!(image[(0, 1)], Color::C5);
    }

    #[test]
    fn rotate_image_ccw() {
        let mut image = Image::new(4, 2);
        image[(1, 0)] = Color::C3;
        image[(1, 1)] = Color::C5;
        let image = image.rotate_ccw();
        assert_eq!(2, image.width());
        assert_eq!(4, image.height());
        assert_eq!(image[(0, 2)], Color::C3);
        assert_eq!(image[(1, 2)], Color::C5);
    }

    #[test]
    fn fill_contained_rect() {
        let mut image = Image::new(5, 5);
        image.fill_rect(1, 1, 2, 2, Color::C3);
        let mut output = Vec::<u8>::new();
        image.write(&mut output).unwrap();
        assert_eq!(&output as &[u8],
                   b"00000\n\
                     03300\n\
                     03300\n\
                     00000\n\
                     00000\n" as &[u8]);
    }

    #[test]
    fn fill_overlapping_rect() {
        let mut image = Image::new(5, 3);
        image.fill_rect(2, 1, 7, 7, Color::C3);
        let mut output = Vec::<u8>::new();
        image.write(&mut output).unwrap();
        assert_eq!(&output as &[u8],
                   b"00000\n\
                     00333\n\
                     00333\n" as &[u8]);
    }

    #[test]
    fn draw_overlapping() {
        let input1: &[u8] = b"EEEEE\n\
                              EEEEE\n\
                              EEEEE\n";
        let mut image1 = Image::read(input1, 5, 3).unwrap();
        let input2: &[u8] = b"01110\n\
                              11011\n\
                              01110\n";
        let image2 = Image::read(input2, 5, 3).unwrap();
        image1.draw(&image2, -1, 1);
        let mut output = Vec::<u8>::new();
        image1.write(&mut output).unwrap();
        assert_eq!(&output as &[u8],
                   b"EEEEE\n\
                     111EE\n\
                     1E11E\n" as &[u8]);
    }
}

// ========================================================================= //
