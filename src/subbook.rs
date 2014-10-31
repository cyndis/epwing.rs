use std;
use std::io::{Reader, Seek, SeekSet, IoResult};

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

#[deriving(Show)]
pub enum Error {
    IoError(std::io::IoError),
    InvalidEncoding
}

pub type Result<T> = std::result::Result<T, Error>;

impl<IO: Reader+Seek> Subbook<IO> {
    pub fn from_io(mut io: IO) -> Result<Subbook<IO>> {
        let indices = try!(Indices::read_from(&mut io).map_err(IoError));

        Ok(Subbook {
            io: io,
            indices: indices
        })
    }
}

impl Indices {
    fn read_from<R: Reader+Seek>(io: &mut R) -> IoResult<Indices> {
        try!(io.seek(1, SeekSet));
        let n_indices = try!(io.read_u8());

        try!(io.seek(4, SeekSet));
        let global_avail = try!(io.read_u8());

        let mut ics = Indices {
            menu: None, copyright: None
        };

        for i in range(0, n_indices) {
            try!(io.seek((16 + i * 16) as i64, SeekSet));

            let index_id = try!(io.read_u8());
            try!(io.read_exact(1));
            let start_page = try!(io.read_be_u32());
            let page_count = try!(io.read_be_u32());
            let avail = try!(io.read_u8());

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
