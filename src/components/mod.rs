pub trait Aligned {
    const ALIGN_SIZE: usize;

    fn aligned_size(raw_size: usize) -> usize {
        let rem = raw_size % Self::ALIGN_SIZE;
        if rem == 0 { raw_size } else { raw_size + (Self::ALIGN_SIZE - rem) }
    }
}

pub trait Header: zerocopy::FromBytes + zerocopy::IntoBytes + zerocopy::Immutable + zerocopy::KnownLayout + Sized {
    fn aligned_size<A: Aligned>(&self) -> usize {
        let raw_size = self.as_bytes().len();
        A::aligned_size(raw_size)
    }
    fn write_to_blob(&self, blob: &mut [u8]) {
        self.write_to_prefix(blob).unwrap();
    }
    fn read_from_blob<'a, A: Aligned>(blob: &'a [u8]) -> (Self, &'a [u8]) {
        let (header, _) = Self::read_from_prefix(blob).unwrap();
        let remaining_bytes = &blob[header.aligned_size::<A>()..];
        (header, remaining_bytes)
    }
}

pub trait View<'a> {
    type Header;

    fn aligned_body_size<A: Aligned>(header: &Self::Header) -> usize;
    fn load_from_body<A: Aligned>(
        header: &Self::Header,
        body_blob: &'a [u8],
    ) -> Self;
}

mod magic_number;
mod encoding_table;
mod count_array;
mod suffix_array;
mod bwm;

pub use magic_number::MagicNumber;
pub use encoding_table::ChrEncodingTable;
pub use count_array::{CountArrayHeader, CountArrayView};
pub use suffix_array::{SuffixArrayHeader, SuffixArrayView};
pub use bwm::{BwmHeader, BwmView, Block, blocks};