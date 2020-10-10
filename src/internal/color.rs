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

use std::io::{self, Error, ErrorKind};

// ========================================================================= //

/// Represents a pixel color for an ASCII Hex Image.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum Color {
    /// The 0th color in a palette.  In the default palette, this color is
    /// completely transparent (R=0, G=0, B=0, A=0).
    C0,
    /// The 1st color in a palette.  In the default palette, this color is
    /// solid black (R=0, G=0, B=0, A=255).
    C1,
    /// The 2nd color in a palette.  In the default palette, this color is
    /// half-brightness red (R=127, G=0, B=0, A=255).
    C2,
    /// The 3rd color in a palette.  In the default palette, this color is
    /// full-brightness red (R=255, G=0, B=0, A=255).
    C3,
    /// The 4th color in a palette.  In the default palette, this color is
    /// half-brightness green (R=0, G=127, B=0, A=255).
    C4,
    /// The 5th color in a palette.  In the default palette, this color is
    /// full-brightness green (R=0, G=255, B=0, A=255).
    C5,
    /// The 6th color in a palette.  In the default palette, this color is
    /// half-brightness yellow (R=127, G=127, B=0, A=255).
    C6,
    /// The 7th color in a palette.  In the default palette, this color is
    /// full-brightness yellow (R=255, G=255, B=0, A=255).
    C7,
    /// The 8th color in a palette.  In the default palette, this color is
    /// half-brightness blue (R=0, G=0, B=127, A=255).
    C8,
    /// The 9th color in a palette.  In the default palette, this color is
    /// full-brightness blue (R=0, G=0, B=255, A=255).
    C9,
    /// The 10th color in a palette.  In the default palette, this color is
    /// half-brightness magenta (R=127, G=0, B=127, A=255).
    Ca,
    /// The 11th color in a palette.  In the default palette, this color is
    /// full-brightness magenta (R=255, G=0, B=255, A=255).
    Cb,
    /// The 12th color in a palette.  In the default palette, this color is
    /// half-brightness cyan (R=0, G=127, B=127, A=255).
    Cc,
    /// The 13th color in a palette.  In the default palette, this color is
    /// full-brightness cyan (R=0, G=255, B=255, A=255).
    Cd,
    /// The 14th color in a palette.  In the default palette, this color is
    /// gray, i.e. half-brightness white (R=127, G=127, B=127, A=255).
    Ce,
    /// The 15th color in a palette.  In the default palette, this color is
    /// solid white (R=255, G=255, B=255, A=255).
    Cf,
}

impl Color {
    pub(crate) fn to_byte(self) -> u8 {
        (b"0123456789ABCDEF")[self as usize]
    }

    pub(crate) fn from_byte(byte: u8) -> io::Result<Color> {
        match byte {
            b'0' => Ok(Color::C0),
            b'1' => Ok(Color::C1),
            b'2' => Ok(Color::C2),
            b'3' => Ok(Color::C3),
            b'4' => Ok(Color::C4),
            b'5' => Ok(Color::C5),
            b'6' => Ok(Color::C6),
            b'7' => Ok(Color::C7),
            b'8' => Ok(Color::C8),
            b'9' => Ok(Color::C9),
            b'A' => Ok(Color::Ca),
            b'B' => Ok(Color::Cb),
            b'C' => Ok(Color::Cc),
            b'D' => Ok(Color::Cd),
            b'E' => Ok(Color::Ce),
            b'F' => Ok(Color::Cf),
            _ => {
                let msg = format!(
                    "invalid pixel character: '{}'",
                    String::from_utf8_lossy(&[byte])
                );
                Err(Error::new(ErrorKind::InvalidData, msg))
            }
        }
    }
}

// ========================================================================= //

#[cfg(test)]
mod tests {
    use super::Color;

    #[test]
    fn color_byte_round_trip() {
        let colors = [
            Color::C0,
            Color::C1,
            Color::C2,
            Color::C3,
            Color::C4,
            Color::C5,
            Color::C6,
            Color::C7,
            Color::C8,
            Color::C9,
            Color::Ca,
            Color::Cb,
            Color::Cc,
            Color::Cd,
            Color::Ce,
            Color::Cf,
        ];
        for &color in colors.iter() {
            assert_eq!(Color::from_byte(color.to_byte()).unwrap(), color);
        }
    }
}

// ========================================================================= //
