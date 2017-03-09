//! Functions for serializing  datatypes used by the protocol
use u128;

use std::io;
use std::io::Write;

use byteorder::{BigEndian, WriteBytesExt};

/* While many of the functions here may seem redundant, keeping them is
 * convenient and consistent. */

/// Write a boolean to the Writer
pub fn write_bool<W: Write>(val: &bool, writer: &mut W) -> io::Result<()> {
    if *val {
        writer.write_all(&[1])
    } else {
        writer.write_all(&[0])
    }
}

/// Write a single unsigned byte to the Writer
pub fn write_u8<W: Write>(val: &u8, writer: &mut W) -> io::Result<()> {
    writer.write_all(&[*val])
}

/// Write a single i8 to the Writer
pub fn write_i8<W: Write>(val: &i8, writer: &mut W) -> io::Result<()> {
    writer.write_i8(*val)
}

/// Write a single u16 to the Writer
pub fn write_u16<W: Write>(val: &u16, writer: &mut W) -> io::Result<()> {
    writer.write_u16::<BigEndian>(*val)
}

/// Write a single i16 to the Writer
pub fn write_i16<W: Write>(val: &i16, writer: &mut W) -> io::Result<()> {
    writer.write_i16::<BigEndian>(*val)
}

/// Write a single unsigned 32-bit int to  the Writer
pub fn write_u32<W: Write>(val: &u32, writer: &mut W) -> io::Result<()> {
    writer.write_u32::<BigEndian>(*val)
}

/// Write a single signed 32-bit int to the Writer
pub fn write_i32<W: Write>(val: &i32, writer: &mut W) -> io::Result<()> {
    writer.write_i32::<BigEndian>(*val)
}

/// Write a single unsigned 64-bit int to the Writer
pub fn write_u64<W: Write>(val: &u64, writer: &mut W) -> io::Result<()> {
    writer.write_u64::<BigEndian>(*val)
}

/// Write a single i64 to the Writer
pub fn write_i64<W: Write>(val: &i64, writer: &mut W) -> io::Result<()> {
    writer.write_i64::<BigEndian>(*val)
}

/// Write a single i32 to the Writer, as a varint
pub fn write_varint<W: Write>(val: &i32, writer: &mut W) -> io::Result<()> {
    if *val == 0 {
        return writer.write_all(&[0]);
    }

    let mut vec: Vec<u8> = Vec::new();

    let mut tmp = {
        if *val > 0 {
            *val as u32
        } else {
            let mut tmp = (*val * -1) as u32;
            tmp ^= 4294967295;
            tmp + 1
        }
    };

    while tmp > 0 {
        /* By default we add the carrying bit to all bytes */
        vec.push(((tmp % 128) as u8) | (1 << 7));
        tmp = tmp >> 7;
    }

    /* and then we remove the carrying bit from the last byte */
    let i = vec.len() - 1;
    vec[i] &= !(1 << 7);

    writer.write_all(&vec)
}

/// Write a single f32 to the Writer
pub fn write_f32<W: Write>(val: &f32, writer: &mut W) -> io::Result<()> {
    writer.write_f32::<BigEndian>(*val)
}

/// Write a single f64 to the Writer
pub fn write_f64<W: Write>(val: &f64, writer: &mut W) -> io::Result<()> {
    writer.write_f64::<BigEndian>(*val)
}

/// Write a String to the Writer, ensuring to properly length-prefix it
#[allow(non_snake_case)]
pub fn write_String<W: Write>(val: &str, writer: &mut W) -> io::Result<()> {
    let string = val.as_bytes();
    let length = val.len() as i32;

    write_varint(&length, writer)?;

    writer.write_all(string)
}

/// Write a length-prefixed bytearray, where the length is given as a varint
pub fn write_prefixed_bytearray<W: Write>(val: &[u8],
                                          writer: &mut W)
                                          -> io::Result<()> {
    write_varint(&(val.len() as i32), writer)?;
    writer.write_all(val)
}

/// Write a length-prefixed varint array, where the length is a varint
pub fn write_prefixed_varintarray<W: Write>(val: &[i32],
                                            writer: &mut W)
                                            -> io::Result<()> {
    write_varint(&(val.len() as i32), writer)?;
    for x in val {
        write_varint(x, writer)?;
    }
    Ok(())
}

/// Write a bytearray without any length prefix
pub fn write_bytearray<W: Write>(val: &Vec<u8>,
                                 writer: &mut W)
                                 -> io::Result<()> {
    writer.write_all(val)
}

/// Alias for write_bytearray
pub fn write_bytearray_to_end<W: Write>(val: &Vec<u8>,
                                        writer: &mut W)
                                        -> io::Result<()> {
    write_bytearray(val, writer)
}

/// Write a uuid (u128) in raw format, i.e. as 16 bytes
pub fn write_uuid<W: Write>(val: &u128, writer: &mut W) -> io::Result<()> {
    let &u128(x, y) = val;
    write_u64(&x, writer)?;
    write_u64(&y, writer)
}

/// Write a uuid (u128) in hexadecimal string format, without dashes
pub fn write_uuid_str<W: Write>(val: &u128, writer: &mut W) -> io::Result<()> {
    let &u128(x, y) = val;
    write!(writer, "{:016x}", x)?;
    write!(writer, "{:016x}", y)
}

/// Write a uuid (u128) in hexadecimal string format with dashes
pub fn write_uuid_str_dashes<W: Write>(val: &u128,
                                       writer: &mut W)
                                       -> io::Result<()> {
    let &u128(x, y) = val;
    /* A uuid in dashes is represented by 5 groups that are 8-4-4-4-12
     * hexadecimal digits each, meaning 32-16-16-16-48 bits each.
     * Number each of these groups a-b-c-d-e */
    let a = x >> 32;
    let b = (x >> 16) & 0xffff;
    let c = x & 0xffff;
    let d = y >> 48;
    let e = y & 0xffffffffffff;
    write!(writer, "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}", a, b, c, d, e)
}

/// Write a position as described on wiki.vg, i.e. x/y/z encoded as an u64
///
/// # Panics
///
/// Panics if X, Y or Z is out of range. X and Z are given 26 bits, and Y is
/// given 12 bits, if the given values are too big to be represented with that
/// amount of memory, will panic.
pub fn write_position<W: Write>(pos: &(i32, i32, i32),
                                writer: &mut W)
                                -> io::Result<()> {
    let &(x, y, z) = pos;
    let x = if x >= 0 {
        x as u64
    } else {
        (x + (1 << 26)) as u64
    };

    let y = if y >= 0 {
        y as u64
    } else {
        (y + (1 << 12)) as u64
    };

    let z = if z >= 0 {
        z as u64
    } else {
        (z + (1 << 26)) as u64
    };

    if x & (!0x3ffffff) != 0 {
        panic!("write_position: X is out of range");
    }

    if y & (!0xfff) != 0 {
        panic!("write_position: Y is out of range");
    }

    if z & (!0x3ffffff) != 0 {
        panic!("write_position: Z is out of range");
    }

    let val = ((x & 0x3ffffff) << 38) | ((y & 0xfff) << 26) | (z & 0x3ffffff);
    write_u64(&val, writer)
}
