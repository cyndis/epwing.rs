use std;
use std::io::{Reader, Seek, SeekSet, IoResult};
use jis0208;
use unicode_hfwidth;

use Error;
use Result;

#[deriving(Show)]
struct IndexLocation {
    page: u32,
    length: u32
}

#[deriving(Show)]
struct Indices {
    menu: Option<IndexLocation>,
    copyright: Option<IndexLocation>,
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

impl<IO: Reader+Seek> Subbook<IO> {
    pub fn from_io(mut io: IO) -> Result<Subbook<IO>> {
        let indices = try!(Indices::read_from(&mut io));

        Ok(Subbook {
            io: io,
            indices: indices
        })
    }

    pub fn read_text(&mut self, page: u32, offset: u16) -> Result<Text> {
        try!(self.io.seek( (page * 0x800 + offset as u32) as i64, SeekSet ));
        read_text(&mut self.io)
    }
}

impl Indices {
    fn read_from<R: Reader+Seek>(io: &mut R) -> IoResult<Indices> {
        try!(io.seek(1, SeekSet));
        let n_indices = try!(io.read_u8());

        try!(io.seek(4, SeekSet));
        let _global_avail = try!(io.read_u8());

        let mut ics = Indices {
            menu: None, copyright: None
        };

        for i in range(0, n_indices) {
            try!(io.seek((16 + i * 16) as i64, SeekSet));

            let index_id = try!(io.read_u8());
            try!(io.read_exact(1));
            let start_page = try!(io.read_be_u32());
            let page_count = try!(io.read_be_u32());
            let _avail = try!(io.read_u8());

            let loc = IndexLocation { page: start_page, length: page_count };

            match index_id {
                0x01 => ics.menu = Some(loc),
                0x02 => ics.copyright = Some(loc),
                _ => ()
            }
        }

        Ok(ics)
    }
}

#[deriving(Show)]
pub enum TextElement {
    UnicodeString(String),
    CustomCharacter(u16),
    Newline
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
                    // Newline
                    0x0a => text.push(TextElement::Newline),
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
                    // End keyword
                    0x61 => (),

                    _ => return Err(Error::InvalidFormat)
                }
            },
            _ => {
                let other = try!(io.read_u8());
                let codepoint = (byte as u16 << 8) | (other as u16);

                if let Some(mut ch) = jis0208::decode_codepoint(codepoint) {
                    if is_narrow {
                        ch = match ch as u32 {
                            // U+3000 IDEOGRAPHIC SPACE
                            0x3000 => ' ',
                            // Characters in Full/Half-width block
                            _      => unicode_hfwidth::to_standard_width(ch).unwrap_or(ch)
                        };
                    }
                    if let Some(&TextElement::UnicodeString(ref mut s)) = text.last_mut() {
                        s.push(ch);
                    } else {
                        text.push(TextElement::UnicodeString(String::from_char(1, ch)));
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
                TextElement::Newline => out.push('\n')
            }
        }

        out
    }
}
