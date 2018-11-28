//! Functions for deserializing datatypes used by the protocol
use errors::{Result, ResultExt};

use std::io::Read;

use byteorder::{BigEndian, ReadBytesExt};

/// Read a single bool from the Reader
pub fn read_bool<R: Read>(reader: &mut R) -> Result<bool> {
    let byte = read_u8(reader)?;
    match byte {
        0 => Ok(false),
        1 => Ok(true),
        _ => bail!("Bool had invalid value {}", byte),
    }
}

/* Many of these functions obviously just duplicate functions from byteorder.
 * I keep them here to keep everything nicely organized, and also to make
 * things easier in the clientbound packet event macros.
 *
 * Also it ensures that if the MC modern protocol is changed, we can simply
 * change these functions without breaking anything. */

/// Read a single i8 from the Reader
pub fn read_i8<R: Read>(reader: &mut R) -> Result<i8> {
    Ok(reader.read_i8()?)
}

/// Read a single u8 from the Reader
pub fn read_u8<R: Read>(reader: &mut R) -> Result<u8> {
    Ok(reader.read_u8()?)
}

/// Read a single i16 from the Reader
pub fn read_i16<R: Read>(reader: &mut R) -> Result<i16> {
    Ok(reader.read_i16::<BigEndian>()?)
}

/// Read a single u16 from the Reader
pub fn read_u16<R: Read>(reader: &mut R) -> Result<u16> {
    Ok(reader.read_u16::<BigEndian>()?)
}

/// Read a single i32 from the Reader
pub fn read_i32<R: Read>(reader: &mut R) -> Result<i32> {
    Ok(reader.read_i32::<BigEndian>()?)
}

/// Read a single i64 from the Reader
pub fn read_i64<R: Read>(reader: &mut R) -> Result<i64> {
    Ok(reader.read_i64::<BigEndian>()?)
}

/// Read a single u64 from the Reader
pub fn read_u64<R: Read>(reader: &mut R) -> Result<u64> {
    Ok(reader.read_u64::<BigEndian>()?)
}

/// Read a single u128 from the Reader
pub fn read_u128<R: Read>(reader: &mut R) -> Result<u128> {
    Ok(reader.read_u128::<BigEndian>()?)
}

/// Read a single f32 from the Reader
pub fn read_f32<R: Read>(reader: &mut R) -> Result<f32> {
    Ok(reader.read_f32::<BigEndian>()?)
}

/// Read a single f64 from the Reader
pub fn read_f64<R: Read>(reader: &mut R) -> Result<f64> {
    Ok(reader.read_f64::<BigEndian>()?)
}

/// Read a length-prefixed utf-8 String from the Reader
#[allow(non_snake_case)]
pub fn read_String<R: Read>(reader: &mut R) -> Result<String> {
    let length = read_varint(reader)? as usize;

    if length > (1 << 16) {
        bail!("read_string refusing to read string due to its length");
    }

    let mut ret = String::with_capacity(length);
    let read = reader.take(length as u64).read_to_string(&mut ret)?;

    if read != length {
        bail!("read_String expected a string with length {} but was only able to read {} bytes", length, read);
    }

    Ok(ret)
}

/// Read a Minecraft-style varint, which currently fits into an i32
pub fn read_varint<R: Read>(reader: &mut R) -> Result<i32> {
    let mut result = 0;

    let msb: u8 = 0b10000000;
    let mask: u8 = !msb;

    for i in 0..5 {
        let read = reader.read_u8()?;

        result |= ((read & mask) as i32) << (7 * i);

        /* The last (5th) byte is only allowed to have the 4 LSB set */
        if i == 4 && (read & 0xf0 != 0) {
            bail!("VarInt is too long, last byte: {}", read);
        }

        if (read & msb) == 0 {
            return Ok(result);
        }
    }

    panic!("read_varint reached end of loop, which should not be possible");
}

/// Read a Minecraft-style varlong, which currently fits into an i64
pub fn read_varlong<R: Read>(reader: &mut R) -> Result<i64> {
    let mut result = 0;

    let msb: u8 = 0b10000000;
    let mask: u8 = !msb;

    for i in 0..10 {
        let read = reader.read_u8()?;

        result |= ((read & mask) as i64) << (7 * i);

        /* The last (10th) byte is only allowed to have the LSB set */
        if i == 9 && ((read & (!0x1)) != 0) {
            bail!("VarLong is too long, last byte: {}", read);
        }

        if (read & msb) == 0 {
            return Ok(result);
        }
    }

    panic!("read_varlong reached end of loop, which should not be possible");
}

/// Read length-prefixed bytearray where the length is given as a varint
pub fn read_prefixed_bytearray<R: Read>(reader: &mut R) -> Result<Vec<u8>> {
    let length = read_varint(reader)?;
    let mut tmp = vec![0; length as usize];
    reader.read_exact(&mut tmp)?;
    Ok(tmp)
}

/// Read length-prefixed varint array where the length is given as a varint
pub fn read_prefixed_varintarray<R: Read>(reader: &mut R) -> Result<Vec<i32>> {
    let length = read_varint(reader)?;
    let mut tmp = Vec::with_capacity(length as usize);
    for _ in 0..length {
        tmp.push(read_varint(reader)?);
    }
    Ok(tmp)
}

/// Read a uuid encoded as a string without dashes
pub fn read_uuid_str<R: Read>(reader: &mut R) -> Result<u128> {
    let tmp = read_String(reader)?;
    u128::from_str_radix(&tmp, 16).chain_err(|| "Invalid UUID, hex string could not be parsed")
}

/// Read a uuid encoded as a string with dashes
pub fn read_uuid_str_dashes<R: Read>(reader: &mut R) -> Result<u128> {
    let tmp = read_String(reader)?.replace("-", "");
    u128::from_str_radix(&tmp, 16).chain_err(|| "Invalid UUID, hex string could not be parsed")
}

/// Read a bytearray to the end of the reader
pub fn read_bytearray_to_end<R: Read>(reader: &mut R) -> Result<Vec<u8>> {
    let mut tmp = Vec::new();
    let _: usize = reader.read_to_end(&mut tmp)?;
    Ok(tmp)
}

/// Alias for read_bytearray_to_end
pub fn read_bytearray<R: Read>(reader: &mut R) -> Result<Vec<u8>> {
    read_bytearray_to_end(reader)
}

/// Read a position as described on wiki.vg, i.e. x/y/z given as an u64
pub fn read_position<R: Read>(reader: &mut R) -> Result<(i32, i32, i32)> {
    let val = read_u64(reader)?;
    let mut x = (val >> 38) as i32;
    let mut y = ((val >> 26) & 0xfff) as i32;
    let mut z = (val << 38 >> 38) as i32;
    if x >= 1 << 25 {
        x -= 1 << 26;
    }
    if y >= 1 << 11 {
        y -= 1 << 12;
    }
    if z >= 1 << 25 {
        z -= 1 << 26;
    }
    Ok((x, y, z))
}
