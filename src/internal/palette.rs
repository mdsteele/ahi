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
