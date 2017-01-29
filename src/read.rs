//! Functions for deserializing datatypes used by the protocol
use std::io;
use std::io::Read;
use std::string;

use byteorder::{ReadBytesExt, BigEndian};

use u128;

/// Read a single bool from the Reader
pub fn read_bool<R: Read>(reader: &mut R) -> io::Result<bool> {
    let byte = try!(read_u8(reader));
    match byte {
        0 => Ok(false),
        1 => Ok(true),
        _ => io_error!("Bool had invalid value {}", byte),
    }
}

/* Many of these functions obviously just duplicate functions from byteorder.
 * I keep them here to keep everything nicely organized, and also to make things
 * easier in the clientbound packet event macros.
 *
 * Also it ensures that if the MC modern protocol is changed, we can simply
 * change these functions without breaking anything. */

/// Read a single i8 from the Reader
pub fn read_i8<R: Read>(reader: &mut R) -> io::Result<i8> {
    reader.read_i8()
}

/// Read a single u8 from the Reader
pub fn read_u8<R: Read>(reader: &mut R) -> io::Result<u8> {
    reader.read_u8()
}

/// Read a single i16 from the Reader
pub fn read_i16<R: Read>(reader: &mut R) -> io::Result<i16> {
    reader.read_i16::<BigEndian>()
}

/// Read a single u16 from the Reader
pub fn read_u16<R: Read>(reader: &mut R) -> io::Result<u16> {
    reader.read_u16::<BigEndian>()
}

/// Read a single i32 from the Reader
pub fn read_i32<R: Read>(reader: &mut R) -> io::Result<i32> {
    reader.read_i32::<BigEndian>()
}

/// Read a single i64 from the Reader
pub fn read_i64<R: Read>(reader: &mut R) -> io::Result<i64> {
    reader.read_i64::<BigEndian>()
}

/// Read a single u64 from the Reader
pub fn read_u64<R: Read>(reader: &mut R) -> io::Result<u64> {
    reader.read_u64::<BigEndian>()
}

/// Read a single f32 from the Reader
pub fn read_f32<R: Read>(reader: &mut R) -> io::Result<f32> {
    reader.read_f32::<BigEndian>()
}

/// Read a single f64 from the Reader
pub fn read_f64<R: Read>(reader: &mut R) -> io::Result<f64> {
    reader.read_f64::<BigEndian>()
}

/// Read a length-prefixed utf-8 String from the Reader
#[allow(non_snake_case)]
pub fn read_String<R: Read>(reader: &mut R) -> io::Result<String> {
    let length = try!(read_varint(reader)) as usize;

    if length > (1 << 16) {
        return io_error!("read_string refusing to read string due to its length");
    }

    /* FIXME can we do this without a double copy? */
    let mut buf = vec![0; length];
    reader.read_exact(&mut buf).unwrap();

    Ok(string::String::from_utf8_lossy(&buf).into_owned())
}

/// Read a Minecraft-style varint, which currently fits into a i32
pub fn read_varint<R: Read>(reader: &mut R) -> io::Result<i32> {
    let mut radix: u64 = 128;
    let msb: u8 = 0b10000000; /* Only the MSB set */


    /* First we read the varint as an unsigned int */
    let mut buf = try!(reader.read_u8());
    let mut res = (buf & (!msb)) as u64;

    let mut i: usize = 0;
    while (buf & msb) != 0 {
        if i >= 5 {
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                                      "VarInt is too long"));
        }

        i += 1;
        buf = try!(reader.read_u8());
        res += ((buf & (!msb)) as u64) * radix;
        radix <<= 7;
    }

    /* Now we convert it to signed */
    if res >= 4294967296 {
        return Err(io::Error::new(io::ErrorKind::InvalidData,
                                  "Received too large varint"));
    }

    if res > 2147483647 {
        return Ok((4294967296 - res) as i32 * -1);
    } else {
        return Ok(res as i32);
    }
}

/// Read length-prefixed bytearray where the length is given as a varint
pub fn read_prefixed_bytearray<R: Read>(reader: &mut R) -> io::Result<Vec<u8>> {
    let length = read_varint(reader)?;
    let mut tmp = vec![0; length as usize];
    reader.read_exact(&mut tmp)?;
    Ok(tmp)
}

/// Read length-prefixed varint array where the length is given as a varint
pub fn read_prefixed_varintarray<R: Read>(reader: &mut R)
-> io::Result<Vec<i32>> {
    let length = read_varint(reader)?;
    let mut tmp = Vec::with_capacity(length as usize);
    for _ in 0..length {
        tmp.push(read_varint(reader)?);
    }
    Ok(tmp)
}

/// Read a uuid encoded as 16 bytes
pub fn read_uuid<R: Read>(reader: &mut R) -> io::Result<u128> {
    let a = read_u64(reader)?;
    let b = read_u64(reader)?;
    Ok(u128(a, b))
}

/// Read a uuid encoded as a string
///
/// Either with or without dashes.
pub fn read_uuid_str<R: Read>(reader: &mut R) -> io::Result<u128> {
    /* If it's without dashes, then it'll be 32 characters long, else it'll be
     * 36 characters long, so we read 32 characters first and see if it contains
     * any dahes */

    let tmp = read_String(reader)?.replace("-", "");

    let a = match u64::from_str_radix(&tmp[..16], 16) {
        Ok(x) => x,
        Err(_) => return io_error!("Invalid hex in first half of uuid_str"),
    };
    let b = match u64::from_str_radix(&tmp[16..], 16) {
        Ok(x) => x,
        Err(_) => return io_error!("Invalid hex in second half of uuid_str"),
    };
    Ok(u128(a, b))
}

/// Read a bytearray to the end of the reader
pub fn read_bytearray_to_end<R: Read>(reader: &mut R) -> io::Result<Vec<u8>> {
    let mut tmp = Vec::new();
    reader.read_to_end(&mut tmp)?;
    Ok(tmp)
}

/// Read a position as described on wiki.vg, i.e. x/y/z given as an u64
pub fn read_position<R: Read>(reader: &mut R) -> io::Result<(i32, i32, i32)> {
    let val = read_u64(reader)?;
    let mut x = (val >> 38) as i32;
    let mut y = ((val >> 26) & 0xfff) as i32;
    let mut z = (val & 0x3ffffff) as i32;
    if x >= 1 << 25 {
        x -= 1 << 26;
    }
    if y >= 1 << 11 {
        y -= 1 << 12;
    }
    if z >= 2 << 25 {
        z -= 1 << 26;
    }
    Ok((x, y, z))
}

