//! Tests the serialization of the various datatypes, i.e. the files read.rs
//! and write.rs
use read::*;
use write::*;

use std::io::Cursor;

/// Given a value, the same correctly encoded, and a reader and write function
/// for that type of value, test that the reader and writer create values that
/// correctly match the supplied ones.
macro_rules! read_and_write {
    ($val:expr, $binary:expr, $reader:path, $writer:path) =>
    {
        let binary = $binary;
        let mut cursor = Cursor::new(binary);
        assert_eq!($reader(&mut cursor).unwrap(), $val);
        let mut tmp = Vec::new();
        $writer(&$val, &mut tmp).unwrap();
        assert_eq!(&tmp, binary);
    };
}

#[test]
fn byteorder() {
    read_and_write!(true, &[1], read_bool, write_bool);
    read_and_write!(-1, &[255], read_i8, write_i8);
    read_and_write!(42, &[42], read_u8, write_u8);
    read_and_write!(-5, &[255, 251], read_i16, write_i16);
    read_and_write!(65535, &[255, 255], read_u16, write_u16);
    read_and_write!(-9, &[255, 255, 255, 247], read_i32, write_i32);
    read_and_write!(-2,
                    &[255, 255, 255, 255, 255, 255, 255, 254],
                    read_i64,
                    write_i64);
    read_and_write!(8898902191272547,
                    &[0x00, 0x1f, 0x9d, 0x81, 0x20, 0x00, 0x72, 0x63],
                    read_u64,
                    write_u64);
    read_and_write!(7546.57470703125,
                    &[0x45, 0xeb, 0xd4, 0x99],
                    read_f32,
                    write_f32);
    read_and_write!(7546.5746871564779212349094450473785400390625,
                    &[0x40, 0xbd, 0x7a, 0x93, 0x1e, 0xb2, 0x8e, 0x81],
                    read_f64,
                    write_f64);
}

#[test]
fn string() {
    read_and_write!("ozelot".to_string(),
                    &[6, b'o', b'z', b'e', b'l', b'o', b't'],
                    read_String,
                    write_String);
    read_and_write!("オゼロット".to_string(),
                    &[15, 0xe3, 0x82, 0xaa, 0xe3, 0x82, 0xbc, 0xe3, 0x83,
                      0xad, 0xe3, 0x83, 0x83, 0xe3, 0x83, 0x88],
                    read_String,
                    write_String);
}

#[test]
fn varint() {
    read_and_write!(300, &[(1 << 7) | 44, 2], read_varint, write_varint);
    read_and_write!(-1, &[255, 255, 255, 255, 15], read_varint, write_varint);
}

#[test]
fn position() {
    read_and_write!((0, 63, 0),
                    &[0, 0, 0, 0, 0xfc, 0, 0, 0],
                    read_position,
                    write_position);
    read_and_write!((32374, 72, 29283),
                    &[0x00, 0x1f, 0x9d, 0x81, 0x20, 0x00, 0x72, 0x63],
                    read_position,
                    write_position);
    read_and_write!((-32374, -72, 29283),
                    &[0xff, 0xe0, 0x62, 0xbe, 0xe0, 0x00, 0x72, 0x63],
                    read_position,
                    write_position);
    read_and_write!((-109, 64, -120),
                    &[0xff, 0xff, 0xe4, 0xc1, 0x03, 0xff, 0xff, 0x88],
                    read_position,
                    write_position);
}
