// Core types
mod text_length;
pub use text_length::Position;
// Header & View to compose FmIndex
mod components;
pub use components::{TextEncoder, text_encoders, Block, blocks};
// Builder for FmIndex
mod builder;
pub use builder::{FmIndexBuilder, BuildError, build_config};

/// FM-index
///
/// FM-index is a data structure to locate all occurrences of a pattern in a text.
#[derive(Clone, PartialEq, Eq)]
pub struct FmIndex<'a, P: Position, B: Block, E: TextEncoder> {
    // source blob data
    source_blob: &'a [u8],
    // headers
    magic_number: components::MagicNumber,
    text_encoder: E,
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
//  - Reference to source blob data
mod reference_to_source_blob;
//  - Get debug info
// mod debug;

#[cfg(test)]
mod tests;
