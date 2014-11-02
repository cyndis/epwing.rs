# EPWING.rs - Dictionary parsing library

EPWING.rs is a Rust library for reading electronic dictionaries stored in the EPWING format. Only a small subset of EPWING features is currently supported.

## Features

Note: features have only been tested on one EPWING book, and may not work with your file.

- Partial reading of CATALOGS files
- Reading text sections in HONMON files
- Conversion of JIS X 0208 text into UTF-8. JIS X 0208 decoding is also available as a separate crate.

## Currently not supported

- Non-JIS X 0208-encoded text
- Images, sound and video
- Most text formatting commands
- Reading indexes, search
- Compressed files
- Fonts

## Will not be supported

- Old EB format books

