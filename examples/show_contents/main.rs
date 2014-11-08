extern crate epwing;

use epwing::ToPlaintext;

pub fn main() {
    let book_path = match std::os::args().get(1) {
        Some(path) => Path::new(path),
        None => panic!("No path given")
    };

    let book = epwing::Book::open(book_path).unwrap();

    println!("Subbooks:");
    for (i, subbook) in book.subbooks().iter().enumerate() {
        println!("  {}: {}", i+1, subbook.title);
    }

    println!("");

    let spine = book.subbooks().head().unwrap();
    println!("Title page ({}) for {}:", spine.index_page, spine.title);

    let mut subbook = book.open_subbook(spine).unwrap();
    let title_text = subbook.read_text(spine.index_page as u32, 0).unwrap();
    println!("{}", title_text.to_plaintext());
}
