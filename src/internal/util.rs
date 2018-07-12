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

use std::char;
use std::i16;
use std::io::{self, Error, ErrorKind, Read};

// ========================================================================= //

const MAX_HEADER_VALUE: i32 = 0xFFFF;

// ========================================================================= //

fn read_char_escape<R: Read>(mut reader: R, quote: u8)
                             -> io::Result<Option<char>> {
    let mut buffer = vec![0u8];
    try!(reader.read_exact(&mut buffer));
    let byte = buffer[0];
    if byte == quote {
        Ok(None)
    } else if byte == b'\\' {
        try!(reader.read_exact(&mut buffer));
        let esc = buffer[0];
        if esc == b'\\' {
            Ok(Some('\\'))
        } else if esc == b'\'' {
            Ok(Some('\''))
        } else if esc == b'"' {
            Ok(Some('"'))
        } else if esc == b'n' {
            Ok(Some('\n'))
        } else if esc == b'r' {
            Ok(Some('\r'))
        } else if esc == b't' {
            Ok(Some('\t'))
        } else if esc == b'u' {
            try!(read_exactly(reader.by_ref(), b"{"));
            let value = try!(read_hex_u32(reader.by_ref(), b'}'));
            char::from_u32(value).ok_or_else(|| {
                let msg = format!("invalid unicode value: {}", value);
                Error::new(ErrorKind::InvalidData, msg)
            }).map(Some)
        } else {
            let msg = format!("invalid char escape: {}", esc);
            Err(Error::new(ErrorKind::InvalidData, msg))
        }
    } else if byte < b' ' || byte > b'~' {
        let msg = format!("invalid char literal byte: {}", byte);
        Err(Error::new(ErrorKind::InvalidData, msg))
    } else {
        Ok(Some(char::from_u32(byte as u32).unwrap()))
    }
}

pub(crate) fn read_exactly<R: Read>(mut reader: R, expected: &[u8])
                                    -> io::Result<()> {
    let mut actual = vec![0u8; expected.len()];
    try!(reader.read_exact(&mut actual));
    if &actual as &[u8] != expected {
        let msg = format!("expected '{}', found '{}'",
                          String::from_utf8_lossy(expected),
                          String::from_utf8_lossy(&actual));
        Err(Error::new(ErrorKind::InvalidData, msg))
    } else {
        Ok(())
    }
}

pub(crate) fn read_header_int<R: Read>(reader: R, terminator: u8)
                                       -> io::Result<i32> {
    let mut negative = false;
    let mut any_digits = false;
    let mut value: i32 = 0;
    for next in reader.bytes() {
        let byte = try!(next);
        if byte == terminator {
            if !any_digits {
                let msg = "missing integer field in header";
                return Err(Error::new(ErrorKind::InvalidData, msg));
            }
            break;
        } else if byte == b'-' {
            if negative || any_digits {
                let msg = "misplaced minus sign in header field";
                return Err(Error::new(ErrorKind::InvalidData, msg));
            }
            negative = true;
        } else if byte < b'0' || byte > b'9' {
            let msg = format!("invalid byte in header field: '{}'",
                              String::from_utf8_lossy(&[byte]));
            return Err(Error::new(ErrorKind::InvalidData, msg));
        } else {
            value = value * 10 + (byte - b'0') as i32;
            if value > MAX_HEADER_VALUE {
                let msg = "header value is too large";
                return Err(Error::new(ErrorKind::InvalidData, msg));
            }
            any_digits = true;
        }
    }
    if negative {
        value = -value;
    }
    Ok(value)
}

pub(crate) fn read_header_uint<R: Read>(reader: R, terminator: u8)
                                        -> io::Result<u32> {
    let value = try!(read_header_int(reader, terminator));
    if value < 0 {
        let msg = format!("value must be nonnegative (was {})", value);
        return Err(Error::new(ErrorKind::InvalidData, msg));
    }
    Ok(value as u32)
}

pub(crate) fn read_hex_digits<R: Read>(reader: R, terminator: u8)
                                       -> io::Result<Vec<u8>> {
    let mut digits = Vec::<u8>::new();
    for next in reader.bytes() {
        let byte = next?;
        if byte == terminator {
            break;
        }
        let digit = if byte >= b'0' && byte <= b'9' {
            byte - b'0'
        } else if byte >= b'a' && byte <= b'f' {
            byte - b'a' + 0xa
        } else if byte >= b'A' && byte <= b'F' {
            byte - b'A' + 0xA
        } else {
            let msg = format!("invalid hex digit: '{}'",
                              String::from_utf8_lossy(&[byte]));
            return Err(Error::new(ErrorKind::InvalidData, msg));
        };
        digits.push(digit as u8);
    }
    Ok(digits)
}

pub(crate) fn read_hex_u32<R: Read>(reader: R, terminator: u8)
                                    -> io::Result<u32> {
    let digits = read_hex_digits(reader, terminator)?;
    if digits.is_empty() {
        let msg = "missing hex literal";
        return Err(Error::new(ErrorKind::InvalidData, msg));
    }
    if digits.len() > 8 {
        let msg = "hex literal is too large";
        return Err(Error::new(ErrorKind::InvalidData, msg));
    }
    let mut value: u32 = 0;
    for digit in digits.into_iter() {
        value = value * 0x10 + digit as u32;
    }
    Ok(value)
}

pub(crate) fn read_list_of_i16s<R: Read>(mut reader: R)
                                         -> io::Result<Vec<i16>> {
    read_exactly(reader.by_ref(), b"[")?;
    let mut values = Vec::<i16>::new();
    let mut done = false;
    while !done {
        let mut negative = false;
        let mut any_digits = false;
        let mut value: i32 = 0;
        for byte in reader.by_ref().bytes() {
            let byte = byte?;
            if byte == b']' || byte == b',' {
                if !any_digits && (byte == b',' || !values.is_empty()) {
                    let msg = "missing integer in list";
                    return Err(Error::new(ErrorKind::InvalidData, msg));
                }
                if byte == b']' {
                    done = true;
                }
                break;
            } else if byte == b'-' {
                if negative || any_digits {
                    let msg = "misplaced minus sign in integer list";
                    return Err(Error::new(ErrorKind::InvalidData, msg));
                }
                negative = true;
            } else if byte < b'0' || byte > b'9' {
                let msg = format!("invalid byte in list integer: '{}'",
                                  String::from_utf8_lossy(&[byte]));
                return Err(Error::new(ErrorKind::InvalidData, msg));
            } else {
                value = value * 10 + (byte - b'0') as i32;
                any_digits = true;
                if value > 0x8000 {
                    break;
                }
            }
        }
        if any_digits {
            if negative {
                value = -value;
            }
            if value > (i16::MAX as i32) || value < (i16::MIN as i32) {
                let msg = "list integer value is out of range";
                return Err(Error::new(ErrorKind::InvalidData, msg));
            }
            values.push(value as i16);
            if !done {
                read_exactly(reader.by_ref(), b" ")?;
            }
        } else {
            debug_assert!(values.is_empty());
            debug_assert!(done);
        }
    }
    Ok(values)
}

pub(crate) fn read_quoted_char<R: Read>(mut reader: R) -> io::Result<char> {
    read_exactly(reader.by_ref(), b"\'")?;
    if let Some(chr) = read_char_escape(reader.by_ref(), b'\'')? {
        read_exactly(reader.by_ref(), b"\'")?;
        Ok(chr)
    } else {
        Err(Error::new(ErrorKind::InvalidData, "empty char literal"))
    }
}

pub(crate) fn read_quoted_string<R: Read>(mut reader: R)
                                          -> io::Result<String> {
    read_exactly(reader.by_ref(), b"\"")?;
    let mut string = String::new();
    while let Some(chr) = read_char_escape(reader.by_ref(), b'"')? {
        string.push(chr);
    }
    Ok(string)
}

// ========================================================================= //
