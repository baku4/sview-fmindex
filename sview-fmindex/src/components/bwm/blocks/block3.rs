use crate::Position;
use super::{Aligned, Block, Vector};

#[repr(C)]
#[derive(zerocopy::FromBytes, zerocopy::IntoBytes, zerocopy::Immutable)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Block3<V: Vector>([V; 3]);

impl<V: Vector> Aligned for Block3<V> {
    const ALIGN_SIZE: usize = V::ALIGN_SIZE;
}

impl<V: Vector> Block for Block3<V> {
    const BLOCK_LEN: u32 = V::BLOCK_LEN;
    const MAX_SYMBOL: u32 = 8;

    #[inline]
    fn vectorize<P: Position>(text_chunk: &[u8], rank_pre_counts: &mut [P]) -> Self {
        let mut bwt_vectors = [V::ZERO; 3];
        text_chunk.iter().for_each(|symidx_with_sentinel| {
            let symidx = symidx_with_sentinel - 1; // sentinel's idx is 0
            rank_pre_counts[symidx as usize] += P::ONE;
            bwt_vectors[0] <<= V::ONE;
            if symidx & 0b001 != 0 {
                bwt_vectors[0] += V::ONE;
            }
            bwt_vectors[1] <<= V::ONE;
            if symidx & 0b010 != 0 {
                bwt_vectors[1] += V::ONE;
            }
            bwt_vectors[2] <<= V::ONE;
            if symidx & 0b100 != 0 {
                bwt_vectors[2] += V::ONE;
            }
        });
        Self(bwt_vectors)
    }
    fn shift_last_offset(&mut self, offset: u32) {
        self.0.iter_mut().for_each(|bits| *bits <<= offset);
    }
    #[inline]
    fn get_remain_count_of(&self, rem: u32, symidx: u8) -> u32 {
        let mut count_bits = match symidx {
            0 => !self.0[2] & !self.0[1] & !self.0[0], // 000
            1 => !self.0[2] & !self.0[1] & self.0[0],  // 001
            2 => !self.0[2] & self.0[1] & !self.0[0],  // 010
            3 => !self.0[2] & self.0[1] & self.0[0],   // 011
            4 => self.0[2] & !self.0[1] & !self.0[0],  // 100
            5 => self.0[2] & !self.0[1] & self.0[0],   // 101
            6 => self.0[2] & self.0[1] & !self.0[0],   // 110
            _ => self.0[2] & self.0[1] & self.0[0],    // 111
        };
        count_bits >>= V::BLOCK_LEN - rem;
        count_bits.count_ones()
    }
    #[inline]
    fn get_symidx_of(&self, rem: u32) -> u8 {
        let mov = V::BLOCK_LEN - rem - 1;
        let v1 = (self.0[0] >> V::from_u32(mov)).as_u8() & 1;
        let v2 = (self.0[1] >> V::from_u32(mov)).as_u8() & 1;
        let v3 = (self.0[2] >> V::from_u32(mov)).as_u8() & 1;
        v1 + (v2 << 1) + (v3 << 2)
    }
}
