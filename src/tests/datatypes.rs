//! Tests the serialization of the various datatypes, i.e. the files read.rs
//! and write.rs
use read::*;
use write::*;

use std::io::Cursor;
use std::i32;

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
    /* Test some special values */
    read_and_write!(0, &[0], read_varint, write_varint);
    read_and_write!(-1, &[255, 255, 255, 255, 15], read_varint, write_varint);
    read_and_write!(i32::MAX, &[255, 255, 255, 255, 7], read_varint, write_varint);
    read_and_write!(i32::MIN, &[128, 128, 128, 128, 8], read_varint, write_varint);

    /* Test some random numbers */
    read_and_write!(16, &[16], read_varint, write_varint);
    read_and_write!(128, &[0x80, 0x01], read_varint, write_varint);
    read_and_write!(255, &[0xff, 0x01], read_varint, write_varint);
    read_and_write!(300, &[(1 << 7) | 44, 2], read_varint, write_varint);
    read_and_write!(2649887, &[0x9f, 0xde, 0xa1, 0x01], read_varint, write_varint);
}

#[test]
fn varint_too_large() {
    let mut cursor = Cursor::new([128, 128, 128, 128, 16]);
    assert!(read_varint(&mut cursor).is_err());
    let mut cursor = Cursor::new([128, 128, 128, 128, 128]);
    assert!(read_varint(&mut cursor).is_err());
}

#[test]
fn varlong() {
    read_and_write!(0, &[0], read_varlong, write_varlong);
    read_and_write!(1, &[1], read_varlong, write_varlong);
    read_and_write!(2, &[2], read_varlong, write_varlong);
    read_and_write!(127, &[127], read_varlong, write_varlong);
    read_and_write!(128, &[128, 1], read_varlong, write_varlong);
    read_and_write!(255, &[255, 1], read_varlong, write_varlong);
    read_and_write!(300, &[(1 << 7) | 44, 2], read_varlong, write_varlong);
    read_and_write!(2147483647, &[255, 255, 255, 255, 7], read_varlong, write_varlong);
    read_and_write!(9223372036854775807, &[255, 255, 255, 255, 255, 255, 255, 255, 127], read_varlong, write_varlong);
    read_and_write!(-1, &[255, 255, 255, 255, 255, 255, 255, 255, 255, 1], read_varlong, write_varlong);
    read_and_write!(-2147483648, &[128, 128, 128, 128, 248, 255, 255, 255, 255, 1], read_varlong, write_varlong);
    read_and_write!(-9223372036854775808, &[128, 128, 128, 128, 128, 128, 128, 128, 128, 1], read_varlong, write_varlong);
}

#[test]
fn varlong_too_large() {
    let mut cursor = Cursor::new([128, 128, 128, 128, 128, 128, 128, 128, 128, 2]);
    assert!(read_varlong(&mut cursor).is_err());
    let mut cursor = Cursor::new([128, 128, 128, 128, 128, 128, 128, 128, 128, 128]);
    assert!(read_varlong(&mut cursor).is_err());
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

#[test]
fn uuid_str_without_dashes() {
        let mut binary = vec![32];
        for _ in 0..32 {
            binary.push(48); /* Ascii 0 */
        }
        let mut cursor = Cursor::new(&binary);
        assert_eq!(read_uuid_str(&mut cursor).unwrap(), 0);
        let mut tmp = Vec::new();
        write_uuid_str(&0, &mut tmp).unwrap();
        assert_eq!(&tmp, &binary);

        let mut binary = vec![32];
        for i in 0..10 {
            binary.push(48 + i); /* Ascii 0 - 10 */
        }
        for i in 0..6 {
            binary.push(97 + i); /* Ascii a - f */
        }
        for i in 0..10 {
            binary.push(48 + i); /* Ascii 0 - 10 */
        }
        for i in 0..6 {
            binary.push(97 + i); /* Ascii a - f */
        }
        let mut cursor = Cursor::new(&binary);
        assert_eq!(read_uuid_str(&mut cursor).unwrap(), 1512366075204170929049582354406559215);
        let mut tmp = Vec::new();
        write_uuid_str(&1512366075204170929049582354406559215, &mut tmp).unwrap();
        assert_eq!(&tmp, &binary);
}

#[test]
fn uuid_str_with_dashes() {
        let mut binary = vec![36];
        for _ in 0..8 {
            binary.push(48); /* Ascii 0 */
        }
        binary.push(45); /* Ascii - */
        for _ in 0..4 {
            binary.push(48); /* Ascii 0 */
        }
        binary.push(45); /* Ascii - */
        for _ in 0..4 {
            binary.push(48); /* Ascii 0 */
        }
        binary.push(45); /* Ascii - */
        for _ in 0..4 {
            binary.push(48); /* Ascii 0 */
        }
        binary.push(45); /* Ascii - */
        for _ in 0..12 {
            binary.push(48); /* Ascii 0 */
        }
        let mut cursor = Cursor::new(&binary);
        assert_eq!(read_uuid_str(&mut cursor).unwrap(), 0);
        let mut tmp = Vec::new();
        write_uuid_str_dashes(&0, &mut tmp).unwrap();
        assert_eq!(&tmp, &binary);

        let mut binary = vec![36];
        for i in 0..8 {
            binary.push(48 + i); /* Ascii 0 .. */
        }
        binary.push(45); /* Ascii - */
        for i in 0..4 {
            binary.push(97 + i); /* Ascii a .. */
        }
        binary.push(45); /* Ascii - */
        for i in 0..4 {
            binary.push(48 + i); /* Ascii 0 .. */
        }
        binary.push(45); /* Ascii - */
        for i in 0..4 {
            binary.push(99 + i); /* Ascii c .. */
        }
        binary.push(45); /* Ascii - */
        for _ in 0..12 {
            binary.push(57); /* Ascii 9 */
        }
        let mut cursor = Cursor::new(&binary);
        assert_eq!(read_uuid_str(&mut cursor).unwrap(), 1512366085766797629701178291595614617);
        let mut tmp = Vec::new();
        write_uuid_str_dashes(&1512366085766797629701178291595614617, &mut tmp).unwrap();
        assert_eq!(&tmp, &binary);
}
