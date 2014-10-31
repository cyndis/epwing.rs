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
}
