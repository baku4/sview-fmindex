use crate::Position;
use super::{Aligned,Header, View};

pub mod blocks;

#[repr(C)]
#[derive(zerocopy::FromBytes, zerocopy::IntoBytes, zerocopy::Immutable, zerocopy::KnownLayout)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BwmHeader {
    // Given
    pub symbol_count: u32,
    _padding: u32,
    // Derivatives
    pub rank_checkpoints_len: u64,
    pub blocks_len: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BwmView<'a, P: Position, B: Block> {
    // From header
    symbol_with_sentinel_count: P,
    // From blob
    sentinel_index: P,
    rank_checkpoints: &'a [P],
    blocks: &'a [B],
}

/// Block to compress the Burrows-Wheeler transformed text
/// 
/// Define the maximum symbol count that can be indexed by the block.
pub trait Block: zerocopy::FromBytes + zerocopy::IntoBytes + zerocopy::Immutable + Aligned {
    const BLOCK_LEN: u32; // Length of block
    const MAX_SYMBOL: u32; // Maximum symbol count that can be indexed by the block

    // Build
    fn vectorize<P: Position>(bwt_text: &[u8], rank_pre_counts: &mut [P]) -> Self;
    fn shift_last_offset(&mut self, offset: u32);
    // Locate
    fn get_remain_count_of(&self, rem: u32, chridx: u8) -> u32;
    fn get_symidx_of(&self, rem: u32) -> u8;
}

impl BwmHeader {
    fn sentinel_index_raw_size<P: Position>(&self) -> usize {
        std::mem::size_of::<P>()
    }
    fn sentinel_index_aligned_size<P: Position, A: Aligned>(&self) -> usize {
        A::aligned_size(self.sentinel_index_raw_size::<P>())
    }
    fn rank_checkpoints_raw_size<P: Position>(&self) -> usize {
        self.rank_checkpoints_len as usize * std::mem::size_of::<P>()
    }
    fn rank_checkpoints_aligned_size<P: Position, A: Aligned>(&self) -> usize {
        A::aligned_size(self.rank_checkpoints_raw_size::<P>())
    }
    fn blocks_raw_size<B: Block>(&self) -> usize {
        self.blocks_len as usize * std::mem::size_of::<B>()
    }
    fn blocks_aligned_size<B: Block, A: Aligned>(&self) -> usize {
        A::aligned_size(self.blocks_raw_size::<B>())
    }
}

impl Header for BwmHeader {}

// ================================================
// Build
// ================================================
impl BwmHeader {
    #[inline]
    pub fn new<P: Position, B: Block>(
        text_len: u64,
        symbol_count: u32,
    ) -> Self {
        let block_len = B::BLOCK_LEN;

        // Add one more block always for save rank checkpoints
        let block_count = (text_len / block_len as u64) + 1;

        let rank_checkpoints_len = block_count * (symbol_count as u64);
        let blocks_len = block_count as u64;

        Self {
            symbol_count,
            _padding: 0,
            rank_checkpoints_len,
            blocks_len,
        }
    }

    pub fn encode_bwm_body<P: Position, B: Block>(
        &self,
        bwt_text: Vec<u8>, // burrow-wheeler transformed text
        sentinel_index: P, // Sentinel index in bwt_text
        blob: &mut [u8],
    ) {
        let last_offset = {
            let rem = bwt_text.len() % B::BLOCK_LEN as usize;
            if rem == 0 {
                0
            } else {
                B::BLOCK_LEN - (rem as u32)
            }
        };

        // Write sentinel_index
        let sentinel_index_blob = &mut blob[..self.sentinel_index_raw_size::<P>()];
        sentinel_index_blob.copy_from_slice(&sentinel_index.as_bytes());

        // Divide blob into rank_checkpoints and blocks
        let sentinel_index_aligned_size = self.sentinel_index_aligned_size::<P, B>();
        let rank_checkpoints_raw_size = self.rank_checkpoints_raw_size::<P>();
        let rank_checkpoints_aligned_size = self.rank_checkpoints_aligned_size::<P, B>();
        let blocks_raw_size = self.blocks_raw_size::<B>();

        let (rank_checkpoints_blob, blocks_blob) = {
            let (left, right) = blob[sentinel_index_aligned_size..].split_at_mut(rank_checkpoints_aligned_size);
            let left: &mut [P] = zerocopy::FromBytes::mut_from_bytes(&mut left[..rank_checkpoints_raw_size]).unwrap();
            let right: &mut [B] = zerocopy::FromBytes::mut_from_bytes(&mut right[..blocks_raw_size]).unwrap();
            (left, right)
        };

        let mut rank_pre_counts = vec![P::ZERO; self.symbol_count as usize];
        let mut rank_checkpoints_start_index = 0;

        bwt_text.chunks(B::BLOCK_LEN as usize).enumerate().for_each(|(block_idx, text_chunk)| {
            rank_checkpoints_blob[
                rank_checkpoints_start_index..rank_checkpoints_start_index+(self.symbol_count as usize)
            ].copy_from_slice(&rank_pre_counts);
            rank_checkpoints_start_index += self.symbol_count as usize;

            let block = B::vectorize(text_chunk, &mut rank_pre_counts);
            blocks_blob[block_idx] = block;
        });

        if last_offset == 0 {
            rank_checkpoints_blob[rank_checkpoints_start_index..].copy_from_slice(&rank_pre_counts);
            blocks_blob[self.blocks_len as usize - 1].as_mut_bytes().fill(0);
        } else {
            let last_block = blocks_blob.last_mut().unwrap();
            last_block.shift_last_offset(last_offset);
        }
    }
}

// ================================================
// Load
// ================================================
impl<'a, P: Position, B: Block> View<'a> for BwmView<'a, P, B> {
    type Header = BwmHeader;
    
    fn aligned_body_size<A: Aligned>(header: &Self::Header) -> usize {
        header.sentinel_index_aligned_size::<P, A>()
        + header.rank_checkpoints_aligned_size::<P, A>()
        + header.blocks_aligned_size::<B, A>()
    }
    fn load_from_body<A: Aligned>(header: &Self::Header, body_blob: &'a [u8]) -> Self {
        let symbol_with_sentinel_count = P::from_u32(header.symbol_count);

        // Sentinel index
        let mut body_start_index = 0;
        let mut body_end_index = header.sentinel_index_raw_size::<P>();
        let mut next_body_start_index = header.sentinel_index_aligned_size::<P, A>();
        let sentinel_index = zerocopy::FromBytes::read_from_bytes(
            &body_blob[body_start_index..body_end_index]
        ).unwrap();

        // Rank checkpoints
        body_start_index = next_body_start_index;
        body_end_index = body_start_index + header.rank_checkpoints_raw_size::<P>();
        next_body_start_index = body_start_index + header.rank_checkpoints_aligned_size::<P, A>();
        let rank_checkpoints: &[P] = zerocopy::FromBytes::ref_from_bytes(
            &body_blob[body_start_index..body_end_index]
        ).unwrap();

        // Blocks
        body_start_index = next_body_start_index;
        body_end_index = body_start_index + header.blocks_raw_size::<B>();
        let blocks: &[B] = zerocopy::FromBytes::ref_from_bytes(
            &body_blob[body_start_index..body_end_index]
        ).unwrap();

        BwmView {
            symbol_with_sentinel_count,
            sentinel_index,
            rank_checkpoints,
            blocks,
        }
    }
}

// ================================================
// Locate
// ================================================
impl<'a, P: Position, B: Block> BwmView<'a, P, B> {
    #[inline]
    pub fn get_next_rank(
        &self,
        mut pos: P,
        symidx: u8,
    ) -> P {
        if pos < self.sentinel_index {
            pos += P::ONE;
        }
        let (quot, rem) = pos.div_rem_with_u32(B::BLOCK_LEN);

        let rank_idx = quot.as_usize() * self.symbol_with_sentinel_count.as_usize() + symidx as usize;
        let rank_precount = self.rank_checkpoints[rank_idx];
        if rem == 0 {
            rank_precount
        } else {
            let rem_count = self.blocks[quot.as_usize()].get_remain_count_of(rem, symidx);
            rank_precount + P::from_u32(rem_count)
        }
    }
    #[inline]
    pub fn get_pre_rank_and_symidx(&self, mut pos: P) -> Option<(P, u8)> {
        if pos == self.sentinel_index - P::ONE {
            return None;
        } else if pos < self.sentinel_index {
            pos += P::ONE;
        }
        let (quot, rem) = pos.div_rem_with_u32(B::BLOCK_LEN);
        
        let block = &self.blocks[quot.as_usize()];
        let symidx = block.get_symidx_of(rem);

        let rank_idx = quot.as_usize() * self.symbol_with_sentinel_count.as_usize() + symidx as usize;
        let rank_precount = self.rank_checkpoints[rank_idx];
        if rem == 0 {
            Some((rank_precount, symidx))
        } else {
            let rem_count = block.get_remain_count_of(rem, symidx);
            Some((rank_precount + P::from_u32(rem_count), symidx))
        }
    }
}
