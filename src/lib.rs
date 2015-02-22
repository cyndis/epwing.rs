#![feature(core, unicode, fs, io, old_path)]

extern crate jis0208;
extern crate unicode_hfwidth;
extern crate byteorder;

use std::error::FromError;
use std::io::Error as IoError;

use catalog::Catalog;
use subbook::Subbook;

pub use subbook::ToPlaintext as ToPlaintext;

pub mod catalog;
pub mod subbook;

mod util;
mod canon;

#[derive(Debug)]
pub enum Error {
    Io(IoError),
    InvalidEncoding,
    InvalidFormat,
    IndexNotAvailable
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        use std::error::Error;

        write!(fmt, "EPWING error: {}", self.description())
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Io(_) => "input/output error",
            Error::InvalidEncoding => "encountered non-JIS X 0208 character",
            Error::InvalidFormat => "file is malformed",
            Error::IndexNotAvailable => "requested index is not available",
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            Error::Io(ref e) => Some(e as &std::error::Error),
            _ => None
        }
    }
}

impl FromError<IoError> for Error {
    fn from_error(err: IoError) -> Error {
        Error::Io(err)
    }
}

impl FromError<byteorder::Error> for Error {
    fn from_error(err: byteorder::Error) -> Error {
        match err {
            byteorder::Error::UnexpectedEOF => Error::InvalidFormat,
            byteorder::Error::Io(e) => Error::Io(e)
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct Book {
    path: Path,
    catalog: Catalog
}

impl Book {
    pub fn open(path: Path) -> Result<Book> {
        let mut catalog_fp = try!(std::fs::File::open(&path.join("CATALOGS")));
        let catalog = try!(Catalog::from_stream(&mut catalog_fp));

        Ok(Book {
            catalog: catalog,
            path: path
        })
    }

    pub fn subbooks(&self) -> &[catalog::Subbook] {
        self.catalog.subbooks.as_slice()
    }

    pub fn open_subbook(&self, subbook: &catalog::Subbook) -> Result<Subbook> {
        let last_nonws_i = try!(subbook.directory.iter().rposition(|&ch| ch != ' ' as u8)
                                                        .ok_or(Error::InvalidFormat));
        let dir_path = &subbook.directory[..last_nonws_i+1];

        let path = self.path.join_many(&[dir_path, b"DATA", subbook.text_file.as_slice()]);
        let fp = try!(std::fs::File::open(&path));

        subbook::Subbook::from_io(fp)
    }
}

