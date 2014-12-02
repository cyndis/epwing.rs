# EPWING.rs - Dictionary parsing library

EPWING.rs is a Rust library for reading electronic dictionaries stored in the EPWING format. Only a small subset of EPWING features is currently supported.

## Features

Note: features have only been tested on two EPWING books, and may not work with your file.

- Partial reading of CATALOGS files
- Reading text sections in HONMON files
- Searching using word as-is indexes
- Automatic conversion of JIS X 0208 text into UTF-8 using the jis0208 crate

## Currently not supported

- Non-JIS X 0208-encoded text
- Images, sound and video
- Some text formatting commands
- Using most index (search) types
- Full search term canonicalization
- Compressed files
- Fonts

## Will not be supported

- Old EB format books

