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
    pub(crate) fn index(self) -> usize {
        match self {
            Color::Transparent => 0,
            Color::Black => 1,
            Color::DarkRed => 2,
            Color::Red => 3,
            Color::DarkGreen => 4,
            Color::Green => 5,
            Color::DarkYellow => 6,
            Color::Yellow => 7,
            Color::DarkBlue => 8,
            Color::Blue => 9,
            Color::DarkMagenta => 10,
            Color::Magenta => 11,
            Color::DarkCyan => 12,
            Color::Cyan => 13,
            Color::Gray => 14,
            Color::White => 15,
        }
    }

    pub(crate) fn to_byte(self) -> u8 {
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

    pub(crate) fn from_byte(byte: u8) -> io::Result<Color> {
        match byte {
            b'0' => Ok(Color::Transparent),
            b'1' => Ok(Color::Black),
            b'2' => Ok(Color::DarkRed),
            b'3' => Ok(Color::Red),
            b'4' => Ok(Color::DarkGreen),
            b'5' => Ok(Color::Green),
            b'6' => Ok(Color::DarkYellow),
            b'7' => Ok(Color::Yellow),
            b'8' => Ok(Color::DarkBlue),
            b'9' => Ok(Color::Blue),
            b'A' => Ok(Color::DarkMagenta),
            b'B' => Ok(Color::Magenta),
            b'C' => Ok(Color::DarkCyan),
            b'D' => Ok(Color::Cyan),
            b'E' => Ok(Color::Gray),
            b'F' => Ok(Color::White),
            _ => {
                let msg = format!("invalid pixel character: '{}'",
                                  String::from_utf8_lossy(&[byte]));
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
            Color::Transparent,
            Color::Black,
            Color::DarkRed,
            Color::Red,
            Color::DarkGreen,
            Color::Green,
            Color::DarkYellow,
            Color::Yellow,
            Color::DarkBlue,
            Color::Blue,
            Color::DarkMagenta,
            Color::Magenta,
            Color::DarkCyan,
            Color::Cyan,
            Color::Gray,
            Color::White,
        ];
        for &color in colors.iter() {
            assert_eq!(Color::from_byte(color.to_byte()).unwrap(), color);
        }
    }
}

// ========================================================================= //
