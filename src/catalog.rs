use std::io::{Read, Seek, SeekFrom};
use byteorder::{ReadBytesExt, BigEndian};
use util::ReadExact;

use jis0208;
use Error;
use Result;

#[derive(Debug)]
pub struct Catalog {
    pub epwing_version: u16,
    pub subbooks: Vec<Subbook>,
}

#[derive(Debug)]
pub struct Subbook {
    pub title: String,
    pub directory: Vec<u8>,
    pub index_page: u16,
    pub text_file: Vec<u8>,
}

impl Catalog {
    pub fn from_stream<R: Read+Seek>(io: &mut R) -> Result<Catalog> {
        let n_subbooks = try!(io.read_u16::<BigEndian>());
        let epwing_version = try!(io.read_u16::<BigEndian>());

        try!(io.read_exact_(12));

        let mut subbooks = Vec::with_capacity(n_subbooks as usize);
        for _ in 0..n_subbooks {
            subbooks.push(try!(Subbook::from_stream(io)));
        }

        Ok(Catalog { epwing_version: epwing_version, subbooks: subbooks })
    }
}

fn trim_zero_cp<'a>(slice: &'a [u8]) -> &'a [u8] {
    let end = slice.chunks(2).position(|cp| cp[0] == 0 && cp[1] == 0);
    match end {
        Some(n) => &slice[..2*n],
        None    => slice
    }
}

impl Subbook {
    fn from_stream<R: Read+Seek>(io: &mut R) -> Result<Subbook> {
        try!(io.seek(SeekFrom::Current(2)));

        let title_jp = try!(io.read_exact_(80));
        let trimmed = trim_zero_cp(title_jp.as_slice());
        let mut title = String::new();
        for cs in trimmed.chunks(2) {
            let (a, b) = (cs[0] as u16, cs[1] as u16);
            let cp = try!(jis0208::decode_codepoint((a << 8) | b).ok_or(Error::InvalidEncoding));
            title.push(cp);
        }
        let directory = try!(io.read_exact_(8));

        try!(io.seek(SeekFrom::Current(4)));

        let index_page = try!(io.read_u16::<BigEndian>());

        Ok(Subbook {
            title: title,
            directory: directory,
            index_page: index_page,
            /* FIXME support EPWINGv2 filename section */
            text_file: b"HONMON".to_vec()
        })
    }
}
