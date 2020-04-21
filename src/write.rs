//! Functions for serializing  datatypes used by the protocol
use errors::Result;

use std::io::Write;

use byteorder::{BigEndian, WriteBytesExt};

/* While many of the functions here may seem redundant, keeping them is
 * convenient and consistent. */

/// Write a boolean to the Writer
pub fn write_bool<W: Write>(val: &bool, writer: &mut W) -> Result<()> {
    if *val {
        Ok(writer.write_all(&[1])?)
    } else {
        Ok(writer.write_all(&[0])?)
    }
}

/// Write a single unsigned byte to the Writer
pub fn write_u8<W: Write>(val: &u8, writer: &mut W) -> Result<()> {
    Ok(writer.write_all(&[*val])?)
}

/// Write a single i8 to the Writer
pub fn write_i8<W: Write>(val: &i8, writer: &mut W) -> Result<()> {
    Ok(writer.write_i8(*val)?)
}

/// Write a single u16 to the Writer
pub fn write_u16<W: Write>(val: &u16, writer: &mut W) -> Result<()> {
    Ok(writer.write_u16::<BigEndian>(*val)?)
}

/// Write a single i16 to the Writer
pub fn write_i16<W: Write>(val: &i16, writer: &mut W) -> Result<()> {
    Ok(writer.write_i16::<BigEndian>(*val)?)
}

/// Write a single unsigned 32-bit int to  the Writer
pub fn write_u32<W: Write>(val: &u32, writer: &mut W) -> Result<()> {
    Ok(writer.write_u32::<BigEndian>(*val)?)
}

/// Write a single signed 32-bit int to the Writer
pub fn write_i32<W: Write>(val: &i32, writer: &mut W) -> Result<()> {
    Ok(writer.write_i32::<BigEndian>(*val)?)
}

/// Write a single unsigned 64-bit int to the Writer
pub fn write_u64<W: Write>(val: &u64, writer: &mut W) -> Result<()> {
    Ok(writer.write_u64::<BigEndian>(*val)?)
}

/// Write a single unsigned 128-bit int to the Writer
pub fn write_u128<W: Write>(val: &u128, writer: &mut W) -> Result<()> {
    Ok(writer.write_u128::<BigEndian>(*val)?)
}

/// Write a single i64 to the Writer
pub fn write_i64<W: Write>(val: &i64, writer: &mut W) -> Result<()> {
    Ok(writer.write_i64::<BigEndian>(*val)?)
}

/// Write a single i32 to the Writer, as a varint
pub fn write_varint<W: Write>(val: &i32, writer: &mut W) -> Result<()> {
    let msb: u8 = 0b10000000;
    let mask: i32 = 0b01111111;

    let mut val = *val;
    for _ in 0..5 {
        let tmp = (val & mask) as u8;
        val &= !mask;
        val = val.rotate_right(7);

        if val != 0 {
            writer.write_all(&[tmp | msb])?;
        } else {
            writer.write_all(&[tmp])?;
            return Ok(());
        }
    }

    panic!("Internal error in write_varint, loop ended");
}

/// Write a single i64 to the Writer, as a Minecraft-style varlong
pub fn write_varlong<W: Write>(val: &i64, writer: &mut W) -> Result<()> {
    let msb: u8 = 0b10000000;
    let mask: i64 = 0b01111111;

    let mut val = *val;
    for _ in 0..10 {
        let tmp = (val & mask) as u8;
        val &= !mask;
        val = val.rotate_right(7);

        if val != 0 {
            writer.write_all(&[tmp | msb])?;
        } else {
            writer.write_all(&[tmp])?;
            return Ok(());
        }
    }

    panic!("Internal error in write_varlong, loop ended");
}

/// Write a single f32 to the Writer
pub fn write_f32<W: Write>(val: &f32, writer: &mut W) -> Result<()> {
    Ok(writer.write_f32::<BigEndian>(*val)?)
}

/// Write a single f64 to the Writer
pub fn write_f64<W: Write>(val: &f64, writer: &mut W) -> Result<()> {
    Ok(writer.write_f64::<BigEndian>(*val)?)
}

/// Write a String to the Writer, ensuring to properly length-prefix it
#[allow(non_snake_case)]
pub fn write_String<W: Write>(val: &str, writer: &mut W) -> Result<()> {
    let string = val.as_bytes();
    let length = val.len() as i32;

    write_varint(&length, writer)?;

    Ok(writer.write_all(string)?)
}

/// Write a length-prefixed bytearray, where the length is given as a varint
pub fn write_prefixed_bytearray<W: Write>(val: &[u8],
                                          writer: &mut W)
                                          -> Result<()> {
    write_varint(&(val.len() as i32), writer)?;
    Ok(writer.write_all(val)?)
}

/// Write a length-prefixed varint array, where the length is a varint
pub fn write_prefixed_varintarray<W: Write>(val: &[i32],
                                            writer: &mut W)
                                            -> Result<()> {
    write_varint(&(val.len() as i32), writer)?;
    for x in val {
        write_varint(x, writer)?;
    }
    Ok(())
}

/// Write a bytearray without any length prefix
pub fn write_bytearray<W: Write>(val: &Vec<u8>, writer: &mut W) -> Result<()> {
    Ok(writer.write_all(val)?)
}

/// Alias for write_bytearray
pub fn write_bytearray_to_end<W: Write>(val: &Vec<u8>,
                                        writer: &mut W)
                                        -> Result<()> {
    write_bytearray(val, writer)
}

/// Write a uuid (u128) in hexadecimal string format, without dashes
pub fn write_uuid_str<W: Write>(val: &u128, writer: &mut W) -> Result<()> {
    /* The string length */
    writer.write_all(&[32])?;
    Ok(write!(writer, "{:032x}", val)?)
}

/// Write a uuid (u128) in hexadecimal string format with dashes
pub fn write_uuid_str_dashes<W: Write>(val: &u128,
                                       writer: &mut W)
                                       -> Result<()> {
    /* A uuid in dashes is represented by 5 groups that are 8-4-4-4-12
     * hexadecimal digits each, meaning 32-16-16-16-48 bits each.
     * Number each of these groups a-b-c-d-e */
    let mask_4: u128 = 0xffff;
    let mask_8: u128 = 0xffffffff;
    let mask_12: u128 = 0xffffffffffff;

    let a = (val >> 96) & mask_8;
    let b = (val >> 80) & mask_4;
    let c = (val >> 64) & mask_4;
    let d = (val >> 48) & mask_4;
    let e = val & mask_12;
    writer.write_all(&[36])?;
    Ok(write!(writer, "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}", a, b, c, d, e)?)
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
                                -> Result<()> {
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

    let val = ((x & 0x3ffffff) << 38) | ((z & 0x3ffffff) << 12) | (y & 0xfff);
    write_u64(&val, writer)
}
