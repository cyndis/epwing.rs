#![feature(if_let)]

extern crate jis0208;
extern crate unicode;

pub mod catalog;
pub mod subbook;

pub fn read_catalog(path: &Path) -> catalog::Result<catalog::Catalog> {
    let mut fp = try!(std::io::File::open(path).map_err(catalog::IoError));

    catalog::Catalog::read_from(&mut fp)
}

pub fn open_subbook(path: &Path) -> subbook::Result<subbook::Subbook<std::io::File>> {
    let fp = try!(std::io::File::open(path).map_err(subbook::IoError));

    subbook::Subbook::from_io(fp)
}
