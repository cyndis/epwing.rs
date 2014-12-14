use std::io::{IoResult, Reader};

use jis0208;
use unicode_hfwidth;

pub trait ReaderJisExt {
    fn read_jis_string(&mut self, len: uint) -> IoResult<Vec<u8>>;
    fn convert_jis_string(&mut self, len: uint) -> IoResult<Option<String>>;
}

impl<T: Reader> ReaderJisExt for T {
    fn read_jis_string(&mut self, len: uint) -> IoResult<Vec<u8>> {
        assert_eq!(len % 2, 0);

        let mut data = Vec::with_capacity(len);

        for _ in range(0, len / 2) {
            let cp = try!(self.read_le_u16());
            if cp == 0x00 {
                continue;
            }

            data.push(((cp     ) & 0xff) as u8);
            data.push(((cp >> 8) & 0xff) as u8);
        }

        Ok(data)
    }

    fn convert_jis_string(&mut self, len: uint) -> IoResult<Option<String>> {
        assert_eq!(len % 2, 0);

        let mut string = String::with_capacity(len / 2);
        let mut done = false;
        let mut err = false;

        for _ in range(0, len / 2) {
            let cp = try!(self.read_le_u16());
            if done { continue }

            if cp == 0x00 {
                done = true;
                continue;
            }

            match jis0208::decode_codepoint(cp) {
                Some(cp) => string.push(cp),
                None     => { err = true; done = true; }
            }
        }

        if err {
            Ok(None)
        } else {
            Ok(Some(string))
        }
    }
}

pub trait CharWidthExt {
    fn to_standard_width(self) -> Self;
    fn to_fullwidth(self) -> Self;
}

impl CharWidthExt for char {
    fn to_standard_width(self) -> char {
        match self {
            // U+3000 IDEOGRAPHIC SPACE
            '\u{3000}' => ' ',
            // Rest
            _        => unicode_hfwidth::to_standard_width(self).unwrap_or(self)
        }
    }

    fn to_fullwidth(self) -> char {
        match self {
            ' ' => '\u{3000}',
            _   => unicode_hfwidth::to_fullwidth(self).unwrap_or(self)
        }
    }
}

pub trait ToJisString for Sized? {
    fn to_jis_string(&self) -> Vec<u8>;
}

impl ToJisString for str {
    fn to_jis_string(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(self.char_len() * 2);
        for ch in self.chars() {
            for cp in jis0208::encode_codepoint(ch).into_iter() {
                data.push(((cp & 0xff00) >> 8) as u8);
                data.push((cp & 0xff) as u8);
            }
        }
        data
    }
}

pub trait ToUnicodeString for Sized? {
    fn to_unicode_string(&self) -> String;
}

impl ToUnicodeString for [u8] {
    fn to_unicode_string(&self) -> String {
        let mut data = String::new();
        for bs in self.chunks(2) {
            let cp = (bs[0] as u16 << 8) | (bs[1] as u16);
            for ch in jis0208::decode_codepoint(cp).into_iter() {
                data.push(ch);
            }
        }
        data
    }
}

#[test]
fn test_conversion_roundtrip() {
    let a = "ｅｎｖｉｒｏｎｍｅｎｔａｌ　ｓｔｒｅｓｓ";

    let b = a.to_jis_string();
    assert_eq!(b[], b"#e#n#v#i#r#o#n#m#e#n#t#a#l!!#s#t#r#e#s#s");

    let c = b.to_unicode_string();
    assert_eq!(c[], a);
}
