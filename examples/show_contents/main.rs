extern crate epwing;

pub fn main() {
    let catalog_path = match std::os::args().get(1) {
        Some(path) => path.clone(),
        None => panic!("No path given")
    };

    println!("{}", catalog_path);

    let path = Path::new(catalog_path.as_slice());
    let mut fp = std::io::File::open(&path).unwrap();

    let catalog = epwing::catalog::Catalog::read_from(&mut fp).unwrap();
    println!("{}", catalog);

    let dir_name = std::str::from_utf8(catalog.subbooks[0].directory.as_slice()).unwrap();
    let file_name = std::str::from_utf8(catalog.subbooks[0].text_file.as_slice()).unwrap();
    let subbook_path = path.dir_path().join_many([dir_name.trim_right(), "DATA", file_name]);

    let mut subbook = epwing::open_subbook(&subbook_path).unwrap();
    println!("{}", subbook);

    let page = subbook.read_text(2, 0).unwrap();
    println!("{}", page);

    println!("{}", page.to_plaintext());
}
