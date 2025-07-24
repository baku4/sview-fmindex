use super::{Header, TextEncoder};

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(zerocopy::FromBytes, zerocopy::IntoBytes, zerocopy::Immutable, zerocopy::KnownLayout)]
pub struct PassThrough;

impl TextEncoder for PassThrough {
    #[inline(always)]
    fn idx_of(&self, sym: u8) -> u8 {
        sym
    }
}

impl Header for PassThrough {}
