// Core types
mod core;
pub use core::Position; // Export to crate root
// Header & View to compose FmIndex
mod components;
pub use components::{Block, blocks}; // Export to crate root
// Builder for FmIndex
mod builder;
pub use builder::{FmIndexBuilder, BuildError, build_config}; // Export to crate root

#[derive(Clone, PartialEq, Eq)]
pub struct FmIndex<'a, P: Position, B: Block> {
    // headers
    magic_number: components::MagicNumber,
    encoding_table: components::ChrEncodingTable,
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

#[cfg(test)]
mod tests;
