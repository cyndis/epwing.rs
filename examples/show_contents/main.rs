extern crate epwing;

pub fn main() {
    let contents_path = match std::os::args().get(1) {
        Some(path) => path.clone(),
        None => panic!("No path given")
    };

    println!("{}", contents_path);

    let path = Path::new(contents_path.as_slice());
    let mut fp = std::io::File::open(&path).unwrap();

    let contents = epwing::contents::Contents::read_from(&mut fp).unwrap();
    println!("{}", contents);
}
