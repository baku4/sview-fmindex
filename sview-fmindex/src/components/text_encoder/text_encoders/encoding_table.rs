use super::{Header, TextEncoder};

/// A table mapping symbols to their indices in the FM-index
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(zerocopy::FromBytes, zerocopy::IntoBytes, zerocopy::Immutable, zerocopy::KnownLayout)]
pub struct EncodingTable([u8; 256]);
impl TextEncoder for EncodingTable {
    fn idx_of(&self, sym: u8) -> u8 {
        unsafe { *self.0.get_unchecked(sym as usize) }
    }
}

impl EncodingTable {
    /// Treat the last symbol as wild card.
    #[inline]
    pub fn from_symbols<T: AsRef<[u8]>>(symbols: &[T]) -> Self {
        let symbol_count = symbols.len() as u32;
        let mut table = [(symbol_count - 1) as u8; 256]; // wild card's index is symbol_count
        symbols.iter().enumerate().for_each(|(idx, sym)| {
            sym.as_ref().iter().for_each(|x| table[*x as usize] = idx as u8);
        });
        Self(table)
    }
    /// Add one additional wildcard
    #[inline]
    pub fn from_symbols_with_wildcard<T: AsRef<[u8]>>(symbols: &[T]) -> Self {
        let symbol_count = symbols.len() as u32 + 1;
        let mut table = [(symbol_count - 1) as u8; 256]; // wild card's index is symbol_count
        symbols.iter().enumerate().for_each(|(idx, sym)| {
            sym.as_ref().iter().for_each(|x| table[*x as usize] = idx as u8);
        });
        Self(table)
    }
    pub fn symbol_count(&self) -> u32 {
        *self.0.iter().max().unwrap() as u32 + 1
    }
}

impl Header for EncodingTable {}