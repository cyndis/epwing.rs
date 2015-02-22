#![feature(old_path)]

extern crate epwing;

use std::borrow::ToOwned;
use epwing::ToPlaintext;

static BOOK_PATH: &'static str = "testbook";

fn open_book() -> epwing::Book {
    epwing::Book::open(Path::new(BOOK_PATH)).unwrap()
}

#[test]
fn catalog_test() {
    let book = open_book();

    assert_eq!(book.subbooks().len(), 1);
    assert_eq!(book.subbooks()[0].title, "ＪＭＤＩＣＴ");
}

#[test]
fn title_test() {
    use epwing::subbook::TextElement::{Unsupported, UnicodeString, Newline, Indent};

    let book = open_book();
    let spine = &book.subbooks()[0];
    let mut sbook = book.open_subbook(spine).unwrap();

    let text = sbook.read_text(epwing::subbook::Location::page(spine.index_page as u32)).unwrap();

    assert_eq!(text,
              [Indent(1), Unsupported("ref"), UnicodeString("→ About this conversion".to_owned()),
              Unsupported("/ref"), Newline, Unsupported("ref"),
              UnicodeString("→ General dictionary license statement".to_owned()),
              Unsupported("/ref"), Newline, Unsupported("ref"), UnicodeString("→ JMDict information".to_owned()),
              Unsupported("/ref"), Newline]);

}

#[test]
fn plaintext_test() {
    let book = open_book();
    let spine = &book.subbooks()[0];
    let mut sbook = book.open_subbook(spine).unwrap();

    let text = sbook.read_text(epwing::subbook::Location::page(spine.index_page as u32)).unwrap();

    assert_eq!(text.to_plaintext(), " <ref>→ About this conversion</ref>
<ref>→ General dictionary license statement</ref>
<ref>→ JMDict information</ref>\n");
}

#[test]
fn keyword_search_test() {
    let book = open_book();
    let spine = &book.subbooks()[0];
    let mut sbook = book.open_subbook(spine).unwrap();

    let result = sbook.search(epwing::subbook::Index::WordAsIs, "environmental stress").unwrap();

    assert_eq!(result.len(), 1);
    assert_eq!(result[0], epwing::subbook::Location { page: 24561, offset: 1264 });
}
