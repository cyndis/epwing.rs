use std;
use std::io::{Reader, Seek, SeekSet};

use jis0208;

use util::{ReaderJisExt, CharWidthExt, ToJisString};
use canon::{CanonicalizationRules, Canonicalization, CanonicalizeExt};

use Error;
use Result;

#[derive(Show, Copy)]
struct IndexData {
    page: u32,
    length: u32,
    canonicalization: CanonicalizationRules
}

#[derive(Show, Copy)]
struct Indices {
    menu: Option<IndexData>,
    copyright: Option<IndexData>,
    word_asis: Option<IndexData>,
}

#[derive(Show, PartialEq, Eq, Copy)]
pub enum Index {
    WordAsIs
}

pub struct Subbook<IO> {
    io: IO,
    indices: Indices
}

impl<IO> std::fmt::Show for Subbook<IO> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Subbook {{ io: ..., indices: {} }}", self.indices)
    }
}

#[derive(Show, Eq, PartialEq, Copy)]
pub struct Location {
    pub page: u32,
    pub offset: u16
}

impl Location {
    pub fn page(page: u32) -> Location {
        Location { page: page, offset: 0 }
    }
}

impl<IO: Reader+Seek> Subbook<IO> {
    pub fn from_io(mut io: IO) -> Result<Subbook<IO>> {
        let indices = try!(Indices::read_from(&mut io));

        Ok(Subbook {
            io: io,
            indices: indices
        })
    }

    pub fn read_text(&mut self, location: Location) -> Result<Text> {
        try!(self.io.seek( (location.page * 0x800 + location.offset as u32) as i64, SeekSet ));
        read_text(&mut self.io)
    }

    pub fn search(&mut self, index: Index, word: &str) -> Result<Vec<Location>> {
        let idata = try!(match index {
            Index::WordAsIs => &self.indices.word_asis,
        }.ok_or(Error::IndexNotAvailable));
        let index_page = idata.page - 1;
        let canonical = word.canonicalize(&idata.canonicalization).to_jis_string();

        try!(self.io.seek( (index_page * 0x800 ) as i64, SeekSet ));
        search_descend(&mut self.io, canonical.as_slice())
    }
}

fn search_descend<IO: Reader+Seek>(io: &mut IO, word: &[u8])
    -> Result<Vec<Location>>
{
    let page_id = try!(io.read_u8());
    let entry_len = try!(io.read_u8()) as uint;
    let entry_count = try!(io.read_be_u16());

    let is_leaf = page_id & 0x80 > 0;
    let has_groups = page_id & 0x10 > 0;
    let has_variable_arrangement = entry_len == 0;

    if is_leaf {
        /* Leaf page with links to content */

        let mut results = vec![];
        let mut matched = false;

        for _ in range(0, entry_count) {
            match (has_groups, has_variable_arrangement) {
                (true, _) => {
                    let group_id = try!(io.read_u8());

                    match group_id {
                        0x80 => {
                            /* Start of group */
                            let name_len = try!(io.read_u8()) as uint;
                            try!(io.read_be_u32());
                            let name = try!(io.read_jis_string(name_len));

                            if word == name[] {
                                matched = true;
                            } else {
                                matched = false;
                            }
                        },
                        0x00 => {
                            /* Single-entry group */
                            unimplemented!()
                        },
                        0xc0 => {
                            /* Group entry */
                            let text_page = try!(io.read_be_u32())-1;
                            let text_offs = try!(io.read_be_u16());

                            if matched {
                                results.push(Location { page: text_page, offset: text_offs });
                            }
                        },
                        _ => panic!("unexpected group_id {}", group_id)
                    }
                },
                (false, true) => {
                    let name_len = try!(io.read_u8()) as uint;
                    let name = try!(io.read_jis_string(name_len));
                    let text_page = try!(io.read_be_u32())-1;
                    let text_offs = try!(io.read_be_u16());
                    let _head_page = try!(io.read_be_u32());
                    let _head_offs = try!(io.read_be_u16());

                    if word == name[] {
                        results.push(Location { page: text_page, offset: text_offs });
                    }
                },
                (false, false) => unimplemented!()
            }
        }

        return Ok(results);
    } else {
        /* Internal node in index tree */

        for _ in range(0, entry_count) {
            let name = try!(io.read_jis_string(entry_len));
            let page = try!(io.read_be_u32()) - 1;

            if word <= name[] {
                try!(io.seek( (page * 0x800 ) as i64, SeekSet ));
                return search_descend(io, word);
            }
        }

        return Ok(vec![]);
    }
}

impl Indices {
    fn read_from<R: Reader+Seek>(io: &mut R) -> Result<Indices> {
        try!(io.seek(1, SeekSet));
        let n_indices = try!(io.read_u8());

        try!(io.seek(4, SeekSet));
        let mut global_avail = try!(io.read_u8());
        if global_avail > 0x02 { global_avail = 0x00; }

        let mut ics = Indices {
            menu: None, copyright: None, word_asis: None
        };

        for i in range(0, n_indices) {
            try!(io.seek((16 + i * 16) as i64, SeekSet));

            let index_id = try!(io.read_u8());
            try!(io.read_exact(1));
            let start_page = try!(io.read_be_u32());
            let page_count = try!(io.read_be_u32());
            let avail = try!(io.read_u8());
            let mut flags = 0u32;
            flags |= (try!(io.read_u8()) as u32) << 16;
            flags |= (try!(io.read_u8()) as u32) << 8;
            flags |= (try!(io.read_u8()) as u32) << 0;

            let space_canonicalization = if index_id == 0x72 || index_id == 0x92 {
                Canonicalization::AsIs
            } else {
                Canonicalization::Delete
            };

            macro_rules! canon(($mask:expr, $shift:expr) => (
                try!(Canonicalization::from_field(((flags & $mask) >> $shift) as u8)
                                      .ok_or(Error::InvalidFormat))
            ));

            let canonicalization =
                if (global_avail == 0x00 || avail == 0x02) || global_avail == 0x02 {
                    CanonicalizationRules {
                        katakana: canon!(0xc00000, 22),
                        lower: canon!(0x300000, 20),
                        mark: if ((flags & 0x0c0000) >> 18) == 0 {
                            Canonicalization::Delete
                        } else {
                            Canonicalization::AsIs
                        },
                        long_vowel: canon!(0x030000, 16),
                        double_consonant: canon!(0x00c000, 14),
                        contracted_sound: canon!(0x003000, 12),
                        small_vowel: canon!(0x000c00, 10),
                        voiced_consonant: canon!(0x000300, 8),
                        p_sound: canon!(0x0000c0, 6),
                        space: space_canonicalization
                    }
                } else if index_id == 0x70 || index_id == 0x90 {
                    CanonicalizationRules {
                        katakana: Canonicalization::Convert,
                        lower: Canonicalization::Convert,
                        mark: Canonicalization::Delete,
                        long_vowel: Canonicalization::Convert,
                        double_consonant: Canonicalization::Convert,
                        contracted_sound: Canonicalization::Convert,
                        small_vowel: Canonicalization::Convert,
                        voiced_consonant: Canonicalization::Convert,
                        p_sound: Canonicalization::Convert,
                        space: space_canonicalization
                    }
                } else {
                    CanonicalizationRules {
                        katakana: Canonicalization::AsIs,
                        lower: Canonicalization::Convert,
                        mark: Canonicalization::AsIs,
                        long_vowel: Canonicalization::AsIs,
                        double_consonant: Canonicalization::AsIs,
                        contracted_sound: Canonicalization::AsIs,
                        small_vowel: Canonicalization::AsIs,
                        voiced_consonant: Canonicalization::AsIs,
                        p_sound: Canonicalization::AsIs,
                        space: space_canonicalization
                    }
                };

            let loc = IndexData {
                page: start_page,
                length: page_count,
                canonicalization: canonicalization
            };

            match index_id {
                0x01 => ics.menu = Some(loc),
                0x02 => ics.copyright = Some(loc),
                0x91 => ics.word_asis = Some(loc),
                _ => ()
            }
        }

        Ok(ics)
    }
}

#[derive(Show, PartialEq, Eq)]
pub enum TextElement {
    UnicodeString(String),
    CustomCharacter(u16),
    Newline,
    Indent(u16),
    NoNewline(bool),
    BeginDecoration(u16),
    EndDecoration,
    Unsupported(&'static str)
}

pub type Text = Vec<TextElement>;

fn read_text<R: Reader>(io: &mut R) -> Result<Text> {
    let mut text = Vec::new();

    let mut is_narrow = false;
    let mut delimiter_keyword = None;

    loop {
        let byte = try!(io.read_u8());
        match byte {
            0x1f => {
                match try!(io.read_u8()) {
                    // Start text
                    0x02 => (),
                    // End text
                    0x03 => break,
                    // Start narrow text
                    0x04 => is_narrow = true,
                    // End narrow text
                    0x05 => is_narrow = false,
                    // Begin subscript
                    0x06 => text.push(TextElement::Unsupported("sub")),
                    // End subscript
                    0x07 => text.push(TextElement::Unsupported("/sub")),
                    // Indent
                    0x09 => text.push(TextElement::Indent(try!(io.read_be_u16()))),
                    // Newline
                    0x0a => text.push(TextElement::Newline),
                    // Superscript
                    0x0e => text.push(TextElement::Unsupported("sup")),
                    // End superscript
                    0x0f => text.push(TextElement::Unsupported("/sup")),
                    // Newline prohibition
                    0x10 => text.push(TextElement::NoNewline(true)),
                    // End newline prohibition
                    0x11 => text.push(TextElement::NoNewline(false)),
                    // Begin keyword
                    0x41 => {
                        let keyword = try!(io.read_be_u16());
                        if delimiter_keyword == Some(keyword) {
                            // Next entry encountered, stop.
                            break;
                        } else if delimiter_keyword.is_none() {
                            delimiter_keyword = Some(keyword);
                        }
                    },
                    // Begin reference
                    0x42 => text.push(TextElement::Unsupported("ref")),
                    // End keyword
                    0x61 => (),
                    // End reference
                    0x62 => {
                        try!(io.read_be_u32()); try!(io.read_be_u16());
                        text.push(TextElement::Unsupported("/ref"));
                    }
                    0xe0 => text.push(TextElement::BeginDecoration(try!(io.read_be_u16()))),
                    0xe1 => text.push(TextElement::EndDecoration),

                    x => { println!("0x{:x}", x); return Err(Error::InvalidFormat) }
                }
            },
            _ => {
                let other = try!(io.read_u8());
                let codepoint = ((byte as u16) << 8) | (other as u16);

                if let Some(mut ch) = jis0208::decode_codepoint(codepoint) {
                    if is_narrow {
                        ch = ch.to_standard_width();
                    }

                    if let Some(&TextElement::UnicodeString(ref mut s)) = text.last_mut() {
                        s.push(ch);
                    } else {
                        text.push(TextElement::UnicodeString(format!("{}", ch)));
                    }
                } else {
                    text.push(TextElement::CustomCharacter(codepoint));
                }
            }
        }
    }

    Ok(text)
}

pub trait ToPlaintext {
    fn to_plaintext(&self) -> String;
}

impl ToPlaintext for Text {
    fn to_plaintext(&self) -> String {
        let mut out = String::new();

        for elem in self.iter() {
            match *elem {
                TextElement::UnicodeString(ref s) => out.push_str(s.as_slice()),
                TextElement::CustomCharacter(_) => (),
                TextElement::Newline => out.push('\n'),
                TextElement::Indent(num) => {
                    for _ in range(0, num) {
                        out.push(' ');
                    }
                },
                TextElement::NoNewline(_mode) => (),
                TextElement::BeginDecoration(_deco) => (),
                TextElement::EndDecoration => (),
                TextElement::Unsupported(name) => out.push_str(format!("<{}>", name)[])
            }
        }

        out
    }
}
