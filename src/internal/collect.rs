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

use internal::image::Image;
use internal::palette::Palette;
use internal::util::{
    read_exactly, read_header_uint, read_hex_u32, read_list_of_i16s,
    read_quoted_string,
};
use std::io::{self, Error, ErrorKind, Read, Write};

// ========================================================================= //

// TODO: Support BHI format, which is a binary encoding of an AHI file, with
// compressed image data.
//
// Header:
// +------------+---------+-------+--------------+------------+
// | 4 bytes    | u16     | u16   | u16          | u16        |
// +------------+---------+-------+--------------+------------+
// | "[ESC]bhi" | version | flags | num_palettes | num_images |
// +------------+---------+-------+--------------+------------+
//
// Global size (if flag 1 is unset):
// +-------+--------+
// | u16   | u16    |
// +-------+--------+
// | width | height |
// +-------+--------+
//
// Palette (repeated num_palettes times):
// +-----------+
// | u32  x 16 |
// +-----------+
// | RGBA x 16 |
// +-----------+
//
// Image (repeated num_images times):
// Tag (if flag 2 is set):
// +--------+--------------+
// | u16    | u8 x length  |
// +--------+--------------+
// | length | utf-8        |
// +--------+--------------+
// Metadata (if flag 4 is set):
// +--------+--------------+
// | u16    | i16 x length |
// +--------+--------------+
// | length | metadata     |
// +--------+--------------+
// Image size (if flag 1 is set):
// +-------+--------+
// | u16   | u16    |
// +-------+--------+
// | width | height |
// +-------+--------+
// Image data:
// +-------------------------------+
// | u8 x ceil(width * height / 2) |
// +-------------------------------+
// | image data                    |
// +-------------------------------+

const FLAG_INDIVIDUAL_DIMENSIONS: u32 = 1;
const FLAG_STRING_TAGS: u32 = 2;
const FLAG_METADATA_INTS: u32 = 4;

// ========================================================================= //

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
        Collection { palettes: Vec::new(), images: Vec::new() }
    }

    /// Reads a collection from an AHI file.
    pub fn read<R: Read>(mut reader: R) -> io::Result<Collection> {
        try!(read_exactly(reader.by_ref(), b"ahi"));
        let version = try!(read_header_uint(reader.by_ref(), b' '));
        if version != 0 && version != 1 {
            let msg = format!("unsupported AHI version: {}", version);
            return Err(Error::new(ErrorKind::InvalidData, msg));
        }
        let flags = if version == 1 {
            read_exactly(reader.by_ref(), b"f")?;
            read_hex_u32(reader.by_ref(), b' ')?
        } else {
            0
        };
        let num_palettes = if version == 1 {
            read_exactly(reader.by_ref(), b"p")?;
            read_header_uint(reader.by_ref(), b' ')? as usize
        } else {
            0
        };
        let (num_images, global_width, global_height) = if version == 0 {
            try!(read_exactly(reader.by_ref(), b"w"));
            let width = try!(read_header_uint(reader.by_ref(), b' '));
            try!(read_exactly(reader.by_ref(), b"h"));
            let height = try!(read_header_uint(reader.by_ref(), b' '));
            try!(read_exactly(reader.by_ref(), b"n"));
            let num_images = try!(read_header_uint(reader.by_ref(), b'\n'));
            (num_images as usize, width, height)
        } else if flags & FLAG_INDIVIDUAL_DIMENSIONS != 0 {
            read_exactly(reader.by_ref(), b"i")?;
            let num_images = read_header_uint(reader.by_ref(), b'\n')?;
            (num_images as usize, 0, 0)
        } else {
            read_exactly(reader.by_ref(), b"i")?;
            let num_images = read_header_uint(reader.by_ref(), b' ')?;
            read_exactly(reader.by_ref(), b"w")?;
            let width = read_header_uint(reader.by_ref(), b' ')?;
            read_exactly(reader.by_ref(), b"h")?;
            let height = read_header_uint(reader.by_ref(), b'\n')?;
            (num_images as usize, width, height)
        };

        let mut palettes = Vec::with_capacity(num_palettes);
        if num_palettes > 0 {
            read_exactly(reader.by_ref(), b"\n")?;
        }
        for _ in 0..num_palettes {
            palettes.push(Palette::read(reader.by_ref())?);
        }

        let mut images = Vec::with_capacity(num_images);
        for _ in 0..num_images {
            read_exactly(reader.by_ref(), b"\n")?;
            let tag = if flags & FLAG_STRING_TAGS != 0 {
                let tag = read_quoted_string(reader.by_ref())?;
                read_exactly(reader.by_ref(), b"\n")?;
                tag
            } else {
                String::new()
            };
            let metadata = if flags & FLAG_METADATA_INTS != 0 {
                let metadata = read_list_of_i16s(reader.by_ref())?;
                read_exactly(reader.by_ref(), b"\n")?;
                metadata
            } else {
                Vec::new()
            };
            let (width, height) = if flags & FLAG_INDIVIDUAL_DIMENSIONS != 0 {
                try!(read_exactly(reader.by_ref(), b"w"));
                let width = try!(read_header_uint(reader.by_ref(), b' '));
                try!(read_exactly(reader.by_ref(), b"h"));
                let height = try!(read_header_uint(reader.by_ref(), b'\n'));
                (width, height)
            } else {
                (global_width, global_height)
            };
            let mut image = Image::read(reader.by_ref(), width, height)?;
            image.set_tag(tag);
            image.set_metadata(metadata);
            images.push(image);
        }

        Ok(Collection { palettes, images })
    }

    /// Writes a collection to an AHI file, automatically choosing the lowest
    /// format version possible.
    pub fn write<W: Write>(&self, mut writer: W) -> io::Result<()> {
        let global_size = if self.images.is_empty() {
            Some((0, 0))
        } else {
            let mut size =
                Some((self.images[0].width(), self.images[0].height()));
            for image in self.images.iter() {
                if Some((image.width(), image.height())) != size {
                    size = None;
                    break;
                }
            }
            size
        };
        let mut has_string_tags = false;
        for image in self.images.iter() {
            if !image.tag().is_empty() {
                has_string_tags = true;
                break;
            }
        }
        let mut has_metadata = false;
        for image in self.images.iter() {
            if !image.metadata().is_empty() {
                has_metadata = true;
                break;
            }
        }
        let version = if self.palettes.is_empty()
            && global_size.is_some()
            && !has_string_tags
            && !has_metadata
        {
            0
        } else {
            1
        };
        if version == 0 {
            let (width, height) = global_size.unwrap();
            try!(write!(
                writer,
                "ahi0 w{} h{} n{}\n",
                width,
                height,
                self.images.len()
            ));
        } else {
            let mut flags = 0;
            if global_size.is_none() {
                flags |= FLAG_INDIVIDUAL_DIMENSIONS;
            }
            if has_string_tags {
                flags |= FLAG_STRING_TAGS;
            }
            if has_metadata {
                flags |= FLAG_METADATA_INTS;
            }
            write!(
                writer,
                "ahi1 f{:X} p{} i{}",
                flags,
                self.palettes.len(),
                self.images.len()
            )?;
            if let Some((width, height)) = global_size {
                write!(writer, " w{} h{}", width, height)?;
            }
            write!(writer, "\n")?;
        }
        if !self.palettes.is_empty() {
            write!(writer, "\n")?;
            for palette in self.palettes.iter() {
                palette.write(writer.by_ref())?;
            }
        }
        for image in self.images.iter() {
            try!(write!(writer, "\n"));
            if has_string_tags {
                let mut escaped = String::new();
                for chr in image.tag().chars() {
                    escaped
                        .push_str(&chr.escape_default().collect::<String>());
                }
                write!(writer, "\"{}\"\n", escaped)?;
            }
            if has_metadata {
                write!(writer, "[")?;
                let mut first = true;
                for &value in image.metadata().iter() {
                    if first {
                        first = false;
                    } else {
                        write!(writer, ", ")?;
                    }
                    write!(writer, "{}", value)?;
                }
                write!(writer, "]\n")?;
            }
            if global_size.is_none() {
                write!(writer, "w{} h{}\n", image.width(), image.height())?;
            }
            image.write(writer.by_ref())?;
        }
        Ok(())
    }
}

// ========================================================================= //

#[cfg(test)]
mod tests {
    use super::*;
    use internal::color::Color;

    #[test]
    fn read_empty_v0_collection() {
        let input: &[u8] = b"ahi0 w0 h0 n0\n";
        let collection = Collection::read(input).unwrap();
        assert_eq!(collection.images.len(), 0);
    }

    #[test]
    fn read_empty_v1_collection() {
        let input: &[u8] = b"ahi1 f0 p0 i0 w0 h0\n";
        let collection = Collection::read(input).unwrap();
        assert_eq!(collection.images.len(), 0);
    }

    #[test]
    fn read_v0_collection_with_two_images() {
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
    fn read_v1_collection_with_two_palettes_and_one_image() {
        let input: &[u8] = b"ahi1 f0 p2 i1 w2 h2\n\
                             \n\
                             0;1;2;3;4;5;6;7;8;9;A;B;C;D;E;F\n\
                             ;0;F00;F70;FF0;0F0;0FF;00F;70F;3;5;8;B;D;F0F;F\n\
                             \n\
                             E0\n\
                             0E\n";
        let collection = Collection::read(input).unwrap();
        assert_eq!(collection.palettes.len(), 2);
        assert_eq!(
            collection.palettes[0][Color::Ce],
            (0xee, 0xee, 0xee, 0xff)
        );
        assert_eq!(collection.palettes[1][Color::Ce], (0xff, 0, 0xff, 0xff));
        assert_eq!(collection.images.len(), 1);
        assert_eq!(collection.images[0][(0, 0)], Color::Ce);
    }

    #[test]
    fn read_v1_collection_with_two_different_sized_images() {
        let input: &[u8] = b"ahi1 f1 p0 i2\n\
                             \n\
                             w4 h2\n\
                             2021\n\
                             5D57\n\
                             \n\
                             w2 h3\n\
                             E0\n\
                             0E\n\
                             F1\n";
        let collection = Collection::read(input).unwrap();
        assert_eq!(collection.images.len(), 2);
        assert_eq!(collection.images[0].width(), 4);
        assert_eq!(collection.images[0].height(), 2);
        assert_eq!(collection.images[1].width(), 2);
        assert_eq!(collection.images[1].height(), 3);
    }

    #[test]
    fn read_v1_collection_with_some_images_empty() {
        let input: &[u8] = b"ahi1 f1 p0 i5\n\
              \n\
              w4 h1\n\
              0000\n\
              \n\
              w4 h0\n\
              \n\
              w3 h1\n\
              000\n\
              \n\
              w0 h4\n\
              \n\
              w2 h1\n\
              00\n";
        let collection = Collection::read(input).unwrap();
        assert_eq!(collection.images.len(), 5);
        assert_eq!(collection.images[0].width(), 4);
        assert_eq!(collection.images[0].height(), 1);
        assert_eq!(collection.images[1].width(), 4);
        assert_eq!(collection.images[1].height(), 0);
        assert_eq!(collection.images[2].width(), 3);
        assert_eq!(collection.images[2].height(), 1);
        assert_eq!(collection.images[3].width(), 0);
        assert_eq!(collection.images[3].height(), 4);
        assert_eq!(collection.images[4].width(), 2);
        assert_eq!(collection.images[4].height(), 1);
    }

    #[test]
    fn read_v1_collection_with_string_tags() {
        let input: &[u8] = b"ahi1 f2 p0 i2 w2 h1\n\
              \n\
              \"foo\"\n\
              00\n\
              \n\
              \"Snowman\\u{2603}\"\n\
              00\n";
        let collection = Collection::read(input).unwrap();
        assert_eq!(collection.images.len(), 2);
        assert_eq!(collection.images[0].tag(), "foo");
        assert_eq!(collection.images[1].tag(), "Snowman\u{2603}");
    }

    #[test]
    fn read_v1_collection_with_metadata() {
        let input: &[u8] = b"ahi1 f4 p0 i2 w2 h1\n\
              \n\
              [1, -2, 3]\n\
              00\n\
              \n\
              [4, 5]\n\
              00\n";
        let collection = Collection::read(input).unwrap();
        assert_eq!(collection.images.len(), 2);
        assert_eq!(collection.images[0].metadata(), &[1, -2, 3]);
        assert_eq!(collection.images[1].metadata(), &[4, 5]);
    }

    #[test]
    fn read_v1_collection_with_all_flags() {
        let input: &[u8] = b"ahi1 f7 p0 i2\n\
              \n\
              \"\"\n\
              [1, -2, 3]\n\
              w3 h1\n\
              000\n\
              \n\
              \"foobar\"\n\
              []\n\
              w1 h2\n\
              0\n\
              0\n";
        let collection = Collection::read(input).unwrap();
        assert_eq!(collection.images.len(), 2);
        assert_eq!(collection.images[0].tag(), "");
        assert_eq!(collection.images[0].metadata(), &[1, -2, 3]);
        assert_eq!(collection.images[0].width(), 3);
        assert_eq!(collection.images[0].height(), 1);
        assert_eq!(collection.images[1].tag(), "foobar");
        assert_eq!(collection.images[1].metadata(), &[]);
        assert_eq!(collection.images[1].width(), 1);
        assert_eq!(collection.images[1].height(), 2);
    }

    #[test]
    fn write_empty_collection() {
        let mut output = Vec::<u8>::new();
        Collection::new().write(&mut output).unwrap();
        assert_eq!(&output as &[u8], b"ahi0 w0 h0 n0\n");
    }

    #[test]
    fn write_collection_with_two_palettes() {
        let mut collection = Collection::new();
        collection.palettes.push(Palette::default().clone());
        collection.palettes.push(Palette::new([(0, 0, 0, 255); 16]));
        let mut output = Vec::<u8>::new();
        collection.write(&mut output).unwrap();
        let expected: &[u8] = b"ahi1 f0 p2 i0 w0 h0\n\
              \n\
              ;0;7F0000;F00;007F00;0F0;7F7F00;FF0;00007F;00F;\
              7F007F;F0F;007F7F;0FF;7F;F\n\
              0;0;0;0;0;0;0;0;0;0;0;0;0;0;0;0\n";
        assert_eq!(&output as &[u8], expected);
    }

    #[test]
    fn write_collection_with_same_sized_images() {
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
        assert_eq!(
            &output as &[u8],
            b"ahi0 w2 h2 n2\n\
                     \n\
                     20\n\
                     5D\n\
                     \n\
                     E0\n\
                     0E\n"
        );
    }

    #[test]
    fn write_collection_with_different_sized_images() {
        let mut collection = Collection::new();
        collection.images.push(Image::new(4, 2));
        collection.images.push(Image::new(1, 3));
        let mut output = Vec::<u8>::new();
        collection.write(&mut output).unwrap();
        let expected: &[u8] = b"ahi1 f1 p0 i2\n\
              \n\
              w4 h2\n\
              0000\n\
              0000\n\
              \n\
              w1 h3\n\
              0\n\
              0\n\
              0\n";
        assert_eq!(&output as &[u8], expected);
    }

    #[test]
    fn write_collection_with_some_images_empty() {
        let mut collection = Collection::new();
        collection.images.push(Image::new(4, 1));
        collection.images.push(Image::new(4, 0));
        collection.images.push(Image::new(3, 1));
        collection.images.push(Image::new(0, 4));
        collection.images.push(Image::new(2, 1));
        let mut output = Vec::<u8>::new();
        collection.write(&mut output).unwrap();
        let expected: &[u8] = b"ahi1 f1 p0 i5\n\
              \n\
              w4 h1\n\
              0000\n\
              \n\
              w4 h0\n\
              \n\
              w3 h1\n\
              000\n\
              \n\
              w0 h4\n\
              \n\
              w2 h1\n\
              00\n";
        assert_eq!(&output as &[u8], expected);
    }

    #[test]
    fn write_collection_with_with_string_tags() {
        let mut collection = Collection::new();
        collection.images.push(Image::new(2, 1));
        collection.images[0].set_tag("foo");
        collection.images.push(Image::new(2, 1));
        collection.images[1].set_tag("Snowman\u{2603}");
        let mut output = Vec::<u8>::new();
        collection.write(&mut output).unwrap();
        let expected: &[u8] = b"ahi1 f2 p0 i2 w2 h1\n\
              \n\
              \"foo\"\n\
              00\n\
              \n\
              \"Snowman\\u{2603}\"\n\
              00\n";
        assert_eq!(&output as &[u8], expected);
    }

    #[test]
    fn write_collection_with_with_metadata_ints() {
        let mut collection = Collection::new();
        collection.images.push(Image::new(2, 1));
        collection.images[0].set_metadata(vec![1, -2, 3]);
        collection.images.push(Image::new(2, 1));
        collection.images[1].set_metadata(vec![4, 5]);
        let mut output = Vec::<u8>::new();
        collection.write(&mut output).unwrap();
        let expected: &[u8] = b"ahi1 f4 p0 i2 w2 h1\n\
              \n\
              [1, -2, 3]\n\
              00\n\
              \n\
              [4, 5]\n\
              00\n";
        assert_eq!(&output as &[u8], expected);
    }

    #[test]
    fn write_collection_with_all_flags() {
        let mut collection = Collection::new();
        collection.images.push(Image::new(3, 1));
        collection.images[0].set_metadata(vec![1, -2, 3]);
        collection.images.push(Image::new(1, 2));
        collection.images[1].set_tag("foobar");
        let mut output = Vec::<u8>::new();
        collection.write(&mut output).unwrap();
        let expected: &[u8] = b"ahi1 f7 p0 i2\n\
              \n\
              \"\"\n\
              [1, -2, 3]\n\
              w3 h1\n\
              000\n\
              \n\
              \"foobar\"\n\
              []\n\
              w1 h2\n\
              0\n\
              0\n";
        assert_eq!(&output as &[u8], expected);
    }
}

// ========================================================================= //
