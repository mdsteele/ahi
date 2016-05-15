//! A library for encoding/decoding ASCII Hex Image (.ahi) files.

#![warn(missing_docs)]

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
}
