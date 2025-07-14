// Core types
mod text_length;
pub use text_length::Position;
// Header & View to compose FmIndex
mod components;
pub use components::{Block, blocks};
// Builder for FmIndex
mod builder;
pub use builder::{FmIndexBuilder, BuildError, build_config};

/// FM-index
///
/// FM-index is a data structure to locate all occurrences of a pattern in a text.
#[derive(Clone, PartialEq, Eq)]
pub struct FmIndex<'a, P: Position, B: Block> {
    // headers
    magic_number: components::MagicNumber,
    encoding_table: components::EncodingTable,
    count_array_header: components::CountArrayHeader,
    suffix_array_header: components::SuffixArrayHeader,
    bwm_header: components::BwmHeader,
    // views
    count_array_view: components::CountArrayView<'a, P>,
    suffix_array_view: components::SuffixArrayView<'a, P>,
    bwm_view: components::BwmView<'a, P, B>,
}

// Methods of FmIndex
//  - Load from blob
mod load_from_blob;
pub use load_from_blob::LoadError;
//  - Count & locate pattern
mod locate;
//  - Get debug info
mod debug;

#[cfg(test)]
mod tests;
