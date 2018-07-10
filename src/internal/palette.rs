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
use internal::util;
use std::io::{self, Error, ErrorKind, Read, Write};

// ========================================================================= //

/// A color palette for images.
#[derive(Clone)]
pub struct Palette {
    rgba: [(u8, u8, u8, u8); 16],
}

impl Palette {
    /// Creates a new Palette from the given RGBA data.
    pub fn new(rgba: [(u8, u8, u8, u8); 16]) -> Palette { Palette { rgba } }

    /// Returns a reference to the default palette.
    pub fn default() -> &'static Palette { &DEFAULT_PALETTE }

    /// Gets this palette's RGBA for the given color slot.
    pub fn get(&self, color: Color) -> (u8, u8, u8, u8) {
        self.rgba[color as usize]
    }

    /// Sets this palette's RGBA for the given color slot.
    pub fn set(&mut self, color: Color, rgba: (u8, u8, u8, u8)) {
        self.rgba[color as usize] = rgba;
    }

    pub(crate) fn read<R: Read>(mut reader: R) -> io::Result<Palette> {
        let mut palette = Palette::new([(0u8, 0u8, 0u8, 0u8); 16]);
        for index in 0..16 {
            let terminator = if index == 15 { b'\n' } else { b';' };
            let digits = util::read_hex_digits(reader.by_ref(), terminator)?;
            palette.rgba[index] = match digits.len() {
                0 => (0, 0, 0, 0),
                1 => {
                    let gray = digits[0] * 0x11;
                    (gray, gray, gray, 255)
                }
                2 => {
                    let gray = digits[0] * 0x10 + digits[1];
                    (gray, gray, gray, 255)
                }
                3 => {
                    (digits[0] * 0x11, digits[1] * 0x11, digits[2] * 0x11, 255)
                }
                4 => {
                    (digits[0] * 0x11, digits[1] * 0x11, digits[2] * 0x11,
                     digits[3] * 0x11)
                }
                5 => {
                    (digits[0] * 0x11, digits[1] * 0x11, digits[2] * 0x11,
                     digits[3] * 0x10 + digits[4])
                }
                6 => {
                    (digits[0] * 0x10 + digits[1],
                     digits[2] * 0x10 + digits[3],
                     digits[4] * 0x10 + digits[5],
                     255)
                }
                7 => {
                    (digits[0] * 0x10 + digits[1],
                     digits[2] * 0x10 + digits[3],
                     digits[4] * 0x10 + digits[5],
                     digits[6] * 0x11)
                }
                8 => {
                    (digits[0] * 0x10 + digits[1],
                     digits[2] * 0x10 + digits[3],
                     digits[4] * 0x10 + digits[5],
                     digits[6] * 0x10 + digits[7])
                }
                _ => {
                    let msg = "too many digits in palette color";
                    return Err(Error::new(ErrorKind::InvalidData, msg));
                }
            };
        }
        Ok(palette)
    }

    pub(crate) fn write<W: Write>(&self, mut writer: W) -> io::Result<()> {
        for (index, &(r, g, b, a)) in self.rgba.iter().enumerate() {
            if a == 0 {
                // Write nothing.
            } else if a == 0xff {
                if r == g && g == b {
                    if r % 0x11 == 0 {
                        write!(writer, "{:01X}", r / 0x11)?;
                    } else {
                        write!(writer, "{:02X}", r)?;
                    }
                } else {
                    if r % 0x11 == 0 && g % 0x11 == 0 && b % 0x11 == 0 {
                        write!(writer, "{:01X}{:01X}{:01X}",
                               r / 0x11, g / 0x11, b / 0x11)?;
                    } else {
                        write!(writer, "{:02X}{:02X}{:02X}", r, g, b)?;
                    }
                }
            } else if a % 0x11 == 0 {
                if r % 0x11 == 0 && g % 0x11 == 0 && b % 0x11 == 0 {
                    write!(writer, "{:01X}{:01X}{:01X}{:01X}",
                           r / 0x11, g / 0x11, b / 0x11, a / 0x11)?;
                } else {
                    write!(writer, "{:02X}{:02X}{:02X}{:01X}",
                           r, g, b, a / 0x11)?;
                }
            } else {
                if r % 0x11 == 0 && g % 0x11 == 0 && b % 0x11 == 0 {
                    write!(writer, "{:01X}{:01X}{:01X}{:02X}",
                           r / 0x11, g / 0x11, b / 0x11, a)?;
                } else {
                    write!(writer, "{:02X}{:02X}{:02X}{:02X}", r, g, b, a)?;
                }
            }
            if index == 15 {
                write!(writer, "\n")?;
            } else {
                write!(writer, ";")?;
            }
        }
        Ok(())
    }
}

const DEFAULT_PALETTE: Palette = Palette {
    rgba: [
        (0, 0, 0, 0),
        (0, 0, 0, 255),
        (127, 0, 0, 255),
        (255, 0, 0, 255),
        (0, 127, 0, 255),
        (0, 255, 0, 255),
        (127, 127, 0, 255),
        (255, 255, 0, 255),
        (0, 0, 127, 255),
        (0, 0, 255, 255),
        (127, 0, 127, 255),
        (255, 0, 255, 255),
        (0, 127, 127, 255),
        (0, 255, 255, 255),
        (127, 127, 127, 255),
        (255, 255, 255, 255),
    ],
};

// ========================================================================= //

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_palette() {
        let input: &[u8] =
            b"C;;7F;F00;1EBA;C2C7F;FF7F00;0;F;3F7FBF9;01234567;1;2;3;4;5\n";
        let palette = Palette::read(input).unwrap();
        assert_eq!(palette.get(Color::C0), (0xcc, 0xcc, 0xcc, 0xff));
        assert_eq!(palette.get(Color::C1), (0, 0, 0, 0));
        assert_eq!(palette.get(Color::C2), (0x7f, 0x7f, 0x7f, 0xff));
        assert_eq!(palette.get(Color::C3), (0xff, 0, 0, 0xff));
        assert_eq!(palette.get(Color::C4), (0x11, 0xee, 0xbb, 0xaa));
        assert_eq!(palette.get(Color::C5), (0xcc, 0x22, 0xcc, 0x7f));
        assert_eq!(palette.get(Color::C6), (0xff, 0x7f, 0, 0xff));
        assert_eq!(palette.get(Color::C7), (0, 0, 0, 0xff));
        assert_eq!(palette.get(Color::C8), (0xff, 0xff, 0xff, 0xff));
        assert_eq!(palette.get(Color::C9), (0x3f, 0x7f, 0xbf, 0x99));
        assert_eq!(palette.get(Color::Ca), (0x01, 0x23, 0x45, 0x67));
    }

    #[test]
    fn read_and_write_palette() {
        let input: &[u8] =
            b"C;;7F;F00;1EBA;C2C7F;FF7F00;0;F;3F7FBF9;01234567;1;2;3;4;5\n";
        let palette = Palette::read(input).unwrap();
        let mut output = Vec::<u8>::new();
        palette.write(&mut output).unwrap();
        assert_eq!(&output as &[u8], input);
    }
}

// ========================================================================= //
