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

//! A library for encoding/decoding ASCII Hex Image (.ahi) and ASCII Hex Font
//! (.ahf) files.
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
//! ahi0 w20 h5 n2
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
//! fields is a decimal number.  So, the above file is AHI version 0 (currently
//! the only valid version), and contains two 20x5-pixel images (all the
//! images in a single file must have the same dimensions).
//!
//! After the header line comes the images, which are separated from the header
//! line and from each other by double-newlines.  Each image has one text line
//! per pixel row, with one hex digit per pixel.  Each pixel row line
//! (including the last one in the file) must be terminated by a newline.
//!
//! To map from hex digits to colors in the default palette, treat each hex
//! digit as a four-digit binary number; the 1's place controls brightness, the
//! 2's place controls red, the 4's place controls green, and the 8's place
//! controls blue.  For example, `3 = 0b0011` is full-brightness red; `A =
//! 0b1010` is half-brightness magenta; `F = 0b1111` is white; and `E = 0x1110`
//! is "half-brightness white", i.e. gray.  Since "full-brightness black" (`1 =
//! 0b0001`) and "half-brightness black" (`0 = 0b0000`) would be the same
//! color, instead color `0` is special-cased to be transparent (and color `1`
//! is black).
//!
//! # The AHF format
//!
//! ASCII Hex Font (AHF) is a variation on the AHI file format, meant for
//! storing 16-color bitmap fonts as an ASCII text file.
//!
//! Here's what a typical .ahf file looks like:
//!
//! ```text
//! ahf0 h6 b5 n2
//!
//! def w4 s5
//! 1111
//! 1001
//! 1001
//! 1001
//! 1001
//! 1111
//!
//! 'A' w5 s6
//! 01110
//! 10001
//! 11111
//! 10001
//! 10001
//! 00000
//!
//! 'g' w4 s5
//! 0000
//! 0111
//! 1001
//! 0111
//! 0001
//! 0110
//! ```
//!
//! The start of the .ahf file is the _header line_, which has the form
//! `ahf<version> h<height> b<baseline> n<num_glyphs>`, where each of the four
//! fields is a decimal number.  So, the above file is AHF version 0 (currently
//! the only valid version), and contains two 6-pixel high glyphs in addition
//! to the default glyph, with a baseline height of 5 pixels from the top.
//!
//! After the header line comes the glyphs, which are separated from the header
//! line and from each other by double-newlines.  Each glyph has a _subheader
//! line_, followed by one text line per pixel row, with one hex digit per
//! pixel.  Each pixel row line (including the last one in the file) must be
//! terminated by a newline.
//!

//! Each glyph subheader line has the form `<char> w<width> l<left> r<right>`,
//! where the `<char>` field is either `def` for the font's default glyph
//! (which must be present, and must come first), or a single-quoted Rust
//! character literal (e.g. `'g'` or `'\n'` or `'\u{2603}'`).  The `<left>` and
//! `<right>` fields give the number of pixels between the left edge of the
//! glyph's image and the virtual left/right edge of the glyph itself when
//! printing a string.  Color mapping of pixels works the same as for AHI
//! files.

#![warn(missing_docs)]

mod internal;

pub use crate::internal::collect::Collection;
pub use crate::internal::color::Color;
pub use crate::internal::image::Image;
pub use crate::internal::palette::Palette;
use crate::internal::util::{
    read_exactly, read_header_int, read_header_uint, read_quoted_char,
};
use std::collections::{btree_map, BTreeMap};
use std::io::{self, Error, ErrorKind, Read, Write};
use std::ops::Deref;
use std::rc::Rc;

// ========================================================================= //

// TODO: Once v1 format Image metadata is implemented, change ahf0-reading API
// to just return a Collection with the appropriate image metadata.

/// An image for a single character in a font, along with spacing information.
#[derive(Clone)]
pub struct Glyph {
    image: Image,
    left: i32,
    right: i32,
}

impl Glyph {
    /// Creates a new glyph with the given image and left/right edges.
    pub fn new(image: Image, left: i32, right: i32) -> Glyph {
        Glyph { image, left, right }
    }

    /// Returns the image for this glyph.
    pub fn image(&self) -> &Image {
        &self.image
    }

    /// Returns a mutable reference to the image for this glyph.
    pub fn image_mut(&mut self) -> &mut Image {
        &mut self.image
    }

    /// Returns the left edge of this glyph, in pixels.  This is the distance
    /// (possibly negative) to the right of the left edge of this glyph's image
    /// at which this glyph should start.  Normally this is zero, but can be
    /// positive if the glyph needs extra space on the left, or negative if the
    /// image should extend to the left (e.g. the tail on a lowercase j).
    pub fn left_edge(&self) -> i32 {
        self.left
    }

    /// Sets the left edge for this glyph.
    pub fn set_left_edge(&mut self, left: i32) {
        self.left = left;
    }

    /// Returns the right edge of this glyph, in pixels.  This is the distance
    /// (possibly negative) to the right of the left edge of this glyph's image
    /// at which the next glyph in the text should start, which may be slightly
    /// different than the width of this glyph's image (e.g. if this is an
    /// italic glyph whose image should extend a bit over that of the next
    /// glyph).
    pub fn right_edge(&self) -> i32 {
        self.right
    }

    /// Sets the right edge for this glyph.
    pub fn set_right_edge(&mut self, right: i32) {
        self.right = right;
    }
}

// ========================================================================= //

/// Represents a mapping from characters to glyphs.
#[derive(Clone)]
pub struct Font {
    glyphs: BTreeMap<char, Rc<Glyph>>,
    default_glyph: Rc<Glyph>,
    baseline: i32,
}

impl Font {
    /// Creates a new, empty font whose glyphs have the given height, in
    /// pixels.  The initial default glyph will be a zero-width space, and the
    /// initial baseline will be equal to the height.
    pub fn with_glyph_height(height: u32) -> Font {
        Font {
            glyphs: BTreeMap::new(),
            default_glyph: Rc::new(Glyph::new(Image::new(0, height), 0, 0)),
            baseline: height as i32,
        }
    }

    /// Returns the image height of the glyphs in this font, in pixels.
    pub fn glyph_height(&self) -> u32 {
        self.default_glyph.image.height()
    }

    /// Returns the baseline height for this font, measured in pixels down from
    /// the top of the glyph.  The baseline is the line on which e.g. most
    /// numerals and capital letters sit, and below which e.g. the tail on a
    /// lowercase 'g' or 'y' extends.
    pub fn baseline(&self) -> i32 {
        self.baseline
    }

    /// Sets the baseline value for this font, measured in pixels down from the
    /// top of the glyph.  It is customary for the baseline to be in the range
    /// (0, `height`], but note that this is not actually required.
    pub fn set_baseline(&mut self, baseline: i32) {
        self.baseline = baseline;
    }

    /// Gets the glyph for the given character, if any.  If you instead want to
    /// get the default glyph for characters that have no glyph, use the index
    /// operator.
    pub fn get_char_glyph(&self, chr: char) -> Option<&Glyph> {
        match self.glyphs.get(&chr) {
            Some(glyph) => Some(glyph.deref()),
            None => None,
        }
    }

    /// Gets a mutable reference to the glyph for the given character, if any.
    pub fn get_char_glyph_mut(&mut self, chr: char) -> Option<&mut Glyph> {
        match self.glyphs.get_mut(&chr) {
            Some(glyph) => Some(Rc::make_mut(glyph)),
            None => None,
        }
    }

    /// Sets the glyph for the given character.  Panics if the new glyph's
    /// height is not equal to the font's glyph height.
    pub fn set_char_glyph(&mut self, chr: char, glyph: Glyph) {
        assert_eq!(glyph.image().height(), self.glyph_height());
        self.glyphs.insert(chr, Rc::new(glyph));
    }

    /// Removes the glyph for the given character from the font.  After calling
    /// this, the font's default glyph will be used for this character.
    pub fn remove_char_glyph(&mut self, chr: char) {
        self.glyphs.remove(&chr);
    }

    /// Gets the default glyph for this font, which is used for characters that
    /// don't have a glyph.
    pub fn default_glyph(&self) -> &Glyph {
        &self.default_glyph
    }

    /// Gets a mutable reference to the default glyph for this font.
    pub fn default_glyph_mut(&mut self) -> &mut Glyph {
        Rc::make_mut(&mut self.default_glyph)
    }

    /// Sets the default glyph for this font.  Panics if the new glyph's height
    /// is not equal to the font's glyph height.
    pub fn set_default_glyph(&mut self, glyph: Glyph) {
        assert_eq!(glyph.image().height(), self.glyph_height());
        self.default_glyph = Rc::new(glyph);
    }

    /// Returns an iterator over the characters that have glyphs in this font.
    pub fn chars(&self) -> Chars {
        Chars { iter: self.glyphs.keys() }
    }

    /// Reads a font from an AHF file.
    pub fn read<R: Read>(mut reader: R) -> io::Result<Font> {
        read_exactly(reader.by_ref(), b"ahf")?;
        let version = read_header_uint(reader.by_ref(), b' ')?;
        if version != 0 {
            let msg = format!("unsupported AHF version: {}", version);
            return Err(Error::new(ErrorKind::InvalidData, msg));
        }
        read_exactly(reader.by_ref(), b"h")?;
        let height = read_header_uint(reader.by_ref(), b' ')?;
        read_exactly(reader.by_ref(), b"b")?;
        let baseline = read_header_int(reader.by_ref(), b' ')?;
        read_exactly(reader.by_ref(), b"n")?;
        let num_glyphs = read_header_uint(reader.by_ref(), b'\n')?;

        read_exactly(reader.by_ref(), b"\ndef ")?;
        let default_glyph = Font::read_glyph(reader.by_ref(), height)?;

        let mut glyphs = BTreeMap::new();
        for _ in 0..num_glyphs {
            read_exactly(reader.by_ref(), b"\n")?;
            let chr = read_quoted_char(reader.by_ref())?;
            read_exactly(reader.by_ref(), b" ")?;
            let glyph = Font::read_glyph(reader.by_ref(), height)?;
            glyphs.insert(chr, Rc::new(glyph));
        }
        Ok(Font { glyphs, default_glyph: Rc::new(default_glyph), baseline })
    }

    fn read_glyph<R: Read>(mut reader: R, height: u32) -> io::Result<Glyph> {
        read_exactly(reader.by_ref(), b"w")?;
        let width = read_header_uint(reader.by_ref(), b' ')?;
        read_exactly(reader.by_ref(), b"l")?;
        let left = read_header_int(reader.by_ref(), b' ')?;
        read_exactly(reader.by_ref(), b"r")?;
        let right = read_header_int(reader.by_ref(), b'\n')?;
        let mut row_buffer = vec![0u8; width as usize];
        let mut pixels = Vec::with_capacity((width * height) as usize);
        for _ in 0..height {
            reader.read_exact(&mut row_buffer)?;
            for &byte in &row_buffer {
                pixels.push(Color::from_byte(byte)?);
            }
            read_exactly(reader.by_ref(), b"\n")?;
        }
        let image = Image {
            tag: String::new(),
            metadata: Vec::new(),
            width,
            height,
            pixels: pixels.into_boxed_slice(),
        };
        Ok(Glyph { image, left, right })
    }

    /// Writes the font to an AHF file.
    pub fn write<W: Write>(&self, mut writer: W) -> io::Result<()> {
        let height = self.glyph_height();
        write!(
            writer,
            "ahf0 h{} b{} n{}\n",
            height,
            self.baseline(),
            self.glyphs.len()
        )?;
        write!(writer, "\ndef ")?;
        Font::write_glyph(writer.by_ref(), &self.default_glyph)?;
        for (chr, glyph) in self.glyphs.iter() {
            let escaped: String = chr.escape_default().collect();
            write!(writer, "\n'{}' ", escaped)?;
            Font::write_glyph(writer.by_ref(), glyph)?;
        }
        Ok(())
    }

    fn write_glyph<W: Write>(mut writer: W, glyph: &Glyph) -> io::Result<()> {
        let image = glyph.image();
        let width = image.width();
        let height = image.height();
        write!(
            writer,
            "w{} l{} r{}\n",
            width,
            glyph.left_edge(),
            glyph.right_edge()
        )?;
        for row in 0..height {
            for col in 0..width {
                let color = image[(col, row)];
                writer.write_all(&[color.to_byte()])?;
            }
            write!(writer, "\n")?;
        }
        Ok(())
    }
}

impl std::ops::Index<char> for Font {
    type Output = Glyph;
    fn index(&self, index: char) -> &Glyph {
        self.glyphs.get(&index).unwrap_or(&self.default_glyph)
    }
}

impl std::ops::IndexMut<char> for Font {
    fn index_mut(&mut self, index: char) -> &mut Glyph {
        Rc::make_mut(match self.glyphs.get_mut(&index) {
            Some(glyph) => glyph,
            None => &mut self.default_glyph,
        })
    }
}

// ========================================================================= //

/// An iterator over a the characters that have glyphs in a font.
pub struct Chars<'a> {
    iter: btree_map::Keys<'a, char, Rc<Glyph>>,
}

impl<'a> Iterator for Chars<'a> {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        self.iter.next().map(|&chr| chr)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a> ExactSizeIterator for Chars<'a> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

// ========================================================================= //

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_font() {
        let input: &[u8] = b"ahf0 h3 b2 n2\n\
            \n\
            def w3 l0 r4\n\
            101\n\
            010\n\
            101\n\
            \n\
            '|' w1 l0 r2\n\
            1\n\
            1\n\
            1\n\
            \n\
            '\\u{2603}' w2 l0 r4\n\
            11\n\
            11\n\
            00\n";
        let font = Font::read(input).expect("failed to read font");
        assert_eq!(font.glyph_height(), 3);
        assert_eq!(font.baseline(), 2);
        assert_eq!(font.default_glyph().image().width(), 3);
        assert_eq!(font.default_glyph().left_edge(), 0);
        assert_eq!(font.default_glyph().right_edge(), 4);
        assert_eq!(font['|'].image().width(), 1);
        assert_eq!(font.chars().len(), 2);
    }

    #[test]
    fn write_font() {
        let mut font = Font::with_glyph_height(3);
        font.set_baseline(2);

        let mut img_default = Image::new(3, 3);
        img_default[(0, 0)] = Color::C1;
        img_default[(2, 0)] = Color::C1;
        img_default[(1, 1)] = Color::C1;
        img_default[(0, 2)] = Color::C1;
        img_default[(2, 2)] = Color::C1;
        font.set_default_glyph(Glyph::new(img_default, 0, 4));

        let mut img_snowman = Image::new(2, 3);
        img_snowman[(0, 0)] = Color::C1;
        img_snowman[(1, 0)] = Color::C1;
        img_snowman[(0, 1)] = Color::C1;
        img_snowman[(1, 1)] = Color::C1;
        font.set_char_glyph('\u{2603}', Glyph::new(img_snowman, 0, 4));

        let mut img_vbar = Image::new(1, 3);
        img_vbar[(0, 0)] = Color::C1;
        img_vbar[(0, 1)] = Color::C1;
        img_vbar[(0, 2)] = Color::C1;
        font.set_char_glyph('|', Glyph::new(img_vbar, 0, 2));

        let mut output = Vec::<u8>::new();
        font.write(&mut output).expect("failed to write font");
        let mut expected = Vec::<u8>::new();
        expected.extend_from_slice(
            b"ahf0 h3 b2 n2\n\
            \n\
            def w3 l0 r4\n\
            101\n\
            010\n\
            101\n\
            \n\
            '|' w1 l0 r2\n\
            1\n\
            1\n\
            1\n\
            \n\
            '\\u{2603}' w2 l0 r4\n\
            11\n\
            11\n\
            00\n",
        );
        assert_eq!(output, expected);
    }
}

// ========================================================================= //
