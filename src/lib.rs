mod bsa;
mod cp1252;

pub use crate::bsa::{Bsa, ReadError};

#[cfg(test)]
mod tests {
    #[test]
    fn make_new_bsa() {
    }
}
