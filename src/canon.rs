use util::CharWidthExt;
use std::iter::FromIterator;

#[derive(Show, Eq, PartialEq, Copy)]
pub enum Canonicalization {
    Convert,
    AsIs,
    Delete
}
impl Canonicalization {
    pub fn from_field(field: u8) -> Option<Canonicalization> {
        match field {
            0x00 => Some(Canonicalization::Convert),
            0x01 => Some(Canonicalization::AsIs),
            0x02 => Some(Canonicalization::Delete),
            _    => None
        }
    }
}

#[derive(Show, Eq, PartialEq, Copy)]
pub struct CanonicalizationRules {
    pub katakana: Canonicalization,
    pub lower: Canonicalization,
    pub mark: Canonicalization,
    pub long_vowel: Canonicalization,
    pub double_consonant: Canonicalization,
    pub contracted_sound: Canonicalization,
    pub small_vowel: Canonicalization,
    pub voiced_consonant: Canonicalization,
    pub p_sound: Canonicalization,
    pub space: Canonicalization
}

pub trait CanonicalizeExt for Sized? {
    fn canonicalize(&self, rules: &CanonicalizationRules) -> String;
}

impl CanonicalizeExt for str {
    fn canonicalize(&self, rules: &CanonicalizationRules) -> String {
        use self::Canonicalization::*;

        FromIterator::from_iter(self.chars().filter_map(|mut ch| {
            ch = ch.to_fullwidth();

            if rules.space == Delete && ch == '\u{3000}' /* IDEOGRAPHIC SPACE */ {
                return None;
            }

            if rules.lower == Convert && ch.is_lowercase() {
                ch = ch.to_uppercase();
            }

            return Some(ch);
        }))
    }
}

#[test]
fn test_canonicalize() {
    let c = CanonicalizationRules {
        katakana: Canonicalization::Convert,
        lower: Canonicalization::Convert,
        mark: Canonicalization::Delete,
        long_vowel: Canonicalization::Convert,
        double_consonant: Canonicalization::Convert,
        contracted_sound: Canonicalization::Convert,
        small_vowel: Canonicalization::Convert,
        voiced_consonant: Canonicalization::Convert,
        p_sound: Canonicalization::Convert,
        space: Canonicalization::Delete
    };

    assert_eq!("environmental stress".canonicalize(&c)[], "ＥＮＶＩＲＯＮＭＥＮＴＡＬＳＴＲＥＳＳ")
}
