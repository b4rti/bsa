//! # bsa
//!
//! Here is an example of how to use this library:
//!
//! ```no_run
//! use std::error::Error;
//!
//! fn main() -> Result<(), Box<dyn Error>> {
//!     let mut bsa = bsa::open("file.bsa")?;
//!     for folder in bsa.folders() {
//!         for file in folder.files() {
//!             println!("File {:?} in folder {:?}", file.name(), folder.name());
//!             let contents = file.read_to_vec(&mut bsa)?;
//!             println!("{:?}", &contents);
//!         }
//!     }
//!     Ok(())
//! }
//! ```

mod bsa;
mod cp1252;
mod hash;

pub use crate::bsa::{open, read, Bsa, File, Folder, ReadError};
