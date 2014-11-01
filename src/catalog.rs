use std;
use std::io::Reader;
use jis0208;

#[deriving(Show)]
pub struct Catalog {
    pub epwing_version: u16,
    pub subbooks: Vec<Subbook>
}

#[deriving(Show)]
pub struct Subbook {
    pub title: String,
    pub directory: Vec<u8>,
    pub index_page: u16,
    pub text_file: Vec<u8>
}

#[deriving(Show)]
pub enum Error {
    IoError(std::io::IoError),
    InvalidEncoding
}

pub type Result<T> = std::result::Result<T, Error>;

impl Catalog {
    pub fn read_from<R: Reader>(io: &mut R) -> Result<Catalog> {
        let n_subbooks = try!(io.read_be_u16().map_err(IoError));
        let epwing_version = try!(io.read_be_u16().map_err(IoError));

        try!(io.read_exact(12).map_err(IoError));

        let mut subbooks = Vec::with_capacity(n_subbooks as uint);
        for _ in range(0, n_subbooks) {
            subbooks.push(try!(Subbook::read_from(io)));
        }

        Ok(Catalog { epwing_version: epwing_version, subbooks: subbooks })
    }
}

fn trim_zero_cp<'a>(slice: &'a [u8]) -> &'a [u8] {
    let end = slice.chunks(2).position(|cp| cp[0] == 0 && cp[1] == 0);
    match end {
        Some(n) => slice.slice_to(2*n),
        None    => slice
    }
}

impl Subbook {
    fn read_from<R: Reader>(io: &mut R) -> Result<Subbook> {
        try!(io.read_exact(2).map_err(IoError));

        let title_jp = try!(io.read_exact(80).map_err(IoError));
        let trimmed = trim_zero_cp(title_jp.as_slice());
        let title = try!(jis0208::decode_string(trimmed).ok_or(InvalidEncoding));
        let directory = try!(io.read_exact(8).map_err(IoError));

        try!(io.read_exact(4).map_err(IoError));

        let index_page = try!(io.read_be_u16().map_err(IoError));

        Ok(Subbook {
            title: title,
            directory: directory,
            index_page: index_page,
            /* FIXME support EPWINGv2 filename section */
            text_file: b"HONMON".to_vec()
        })
    }
}
