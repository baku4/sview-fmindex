use crate::{
    FmIndex,
    // traits
    Position, Block, TextEncoder,
};

impl<'a, P: Position, B: Block, E: TextEncoder> FmIndex<'a, P, B, E> {
    /// Returns the original blob from which this FmIndex was loaded.
    pub fn blob(&self) -> &'a [u8] {
        self.source_blob
    }
}