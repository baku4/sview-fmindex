pub trait TextEncoder {
    /// The index of the given symbol.
    fn idx_of(&self, sym: u8) -> u8;
}