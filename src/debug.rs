use super::{FmIndex, Position, Block};

impl<'a, P: Position, B: Block> FmIndex<'a, P, B> {
    pub fn get_encoding_table(&self) -> &[u8] {
        &self.encoding_table.get_encoding_table()
    }
}

// TODO: impl Debug trait for FmIndex