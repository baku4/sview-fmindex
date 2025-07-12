use super::Header;

/// A table mapping symbols to their indices in the FM-index
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(zerocopy::FromBytes, zerocopy::IntoBytes, zerocopy::Immutable, zerocopy::KnownLayout)]
pub struct EncodingTable([u8; 256]);

impl EncodingTable {
    #[inline]
    pub fn new<T>(symbols: &[T]) -> Self
    where
        T: AsRef<[u8]>,
    {
        let symbol_count = symbols.len() as u32;
        let mut table = [symbol_count as u8; 256]; // wild card's index is symbol_count
        symbols.iter().enumerate().for_each(|(idx, sym)| {
            sym.as_ref().iter().for_each(|x| table[*x as usize] = idx as u8);
        });
        Self(table)
    }
    #[inline(always)]
    pub fn idx_of(&self, sym: u8) -> u8 {
        unsafe { *self.0.get_unchecked(sym as usize) }
    }

    pub fn get_encoding_table(&self) -> &[u8] {
        &self.0
    }
}

impl Header for EncodingTable {}