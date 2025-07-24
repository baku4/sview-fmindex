use super::Header;

pub trait TextEncoder: Header {
    /// The index of the given symbol.
    fn idx_of(&self, sym: u8) -> u8;
}

pub mod text_encoders;
