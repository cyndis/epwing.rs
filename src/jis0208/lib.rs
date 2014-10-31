#![feature(phase, slicing_syntax)]

mod map;

pub fn decode_codepoint(codepoint: u16) -> Option<char> {
    map::decode(codepoint)
}

pub fn decode_codepoints(codepoints: &[u16]) -> Option<String> {
    let mut string = String::with_capacity(codepoints.len());

    for ch in codepoints.iter().map(|&cp| map::decode(cp)) {
        match ch {
            Some(ch) => string.push(ch),
            None     => return None
        }
    }

    Some(string)
}

pub fn decode_string(string: &[u8]) -> Option<String> {
    let cps: Vec<u16> = string.chunks(2).map(|chunk| (chunk[0] as u16 << 8) | chunk[1] as u16)
                              .collect();
    decode_codepoints(cps[])
}

#[test]
fn test_decode() {
    assert_eq!(decode_codepoint(0x2341), std::char::from_u32(0xff21));
    assert_eq!(decode_codepoint(0x3000), None);
}

#[test]
fn test_decode_string() {
    assert_eq!(Some("あい".to_string()), decode_string(b"\x24\x22\x24\x24"));
}
