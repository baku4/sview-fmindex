use zerocopy::IntoBytes;

use crate::Position;
use super::{TextEncoder, Aligned, Header, View};

#[repr(C)]
#[derive(zerocopy::FromBytes, zerocopy::IntoBytes, zerocopy::Immutable, zerocopy::KnownLayout)]
#[derive(Debug, Clone, PartialEq, Eq)]
/// A data structure for storing and querying character counts in the FM-index
pub struct CountArrayHeader {
    // Given
    pub symbol_count: u32,
    pub lookup_table_kmer_size: u32,
    // Derivatives
    pub count_array_len: u32,
    pub kmer_multiplier_len: u32,
    pub kmer_count_table_len: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CountArrayView<'a, P: Position> {
    // From header
    lookup_table_kmer_size: usize,
    // Owned
    count_array: Vec<P>,
    kmer_multiplier: Vec<usize>,
    // Reference
    kmer_count_table: &'a [P],
}

impl CountArrayHeader {
    fn count_array_raw_size<P: Position>(&self) -> usize {
        self.count_array_len as usize * std::mem::size_of::<P>()
    }
    fn count_array_aligned_size<P: Position, A: Aligned>(&self) -> usize {
        A::aligned_size(self.count_array_raw_size::<P>())
    }
    fn kmer_multiplier_raw_size(&self) -> usize {
        self.kmer_multiplier_len as usize * std::mem::size_of::<usize>()
    }
    fn kmer_multiplier_aligned_size<A: Aligned>(&self) -> usize {
        A::aligned_size(self.kmer_multiplier_raw_size())
    }
    fn kmer_count_table_raw_size<P: Position>(&self) -> usize {
        self.kmer_count_table_len as usize * std::mem::size_of::<P>()
    }
    fn kmer_count_table_aligned_size<P: Position, A: Aligned>(&self) -> usize {
        A::aligned_size(self.kmer_count_table_raw_size::<P>())
    }
}

impl Header for CountArrayHeader {}

// ================================================
// Build
// ================================================
impl CountArrayHeader {
    pub fn new(
        symbol_count: u32,
        lookup_table_kmer_size: u32,
    ) -> Self {
        // Total number of symbols in burrow-wheeler transformed text:
        // symbol_count + 1 (wildcard) + 1 (sentinel)
        let total_symbol_count_in_bwt = symbol_count + 2;

        let count_array_len = total_symbol_count_in_bwt;
        let kmer_multiplier_len = lookup_table_kmer_size;
        let kmer_count_table_len = (total_symbol_count_in_bwt).pow(lookup_table_kmer_size) as u64;
        
        Self {
            symbol_count,
            lookup_table_kmer_size,
            count_array_len,
            kmer_multiplier_len,
            kmer_count_table_len,
        }

    }
    pub fn count_and_encode_text<P: Position, A: Aligned, E: TextEncoder>(
        &self,
        text: &mut Vec<u8>,
        text_encoder: &E,
        blob: &mut [u8],
    ) {
        // 1) Init
        let total_symbol_count_in_bwt = self.count_array_len as usize;
        //  - count array
        let mut count_array = vec![P::ZERO; self.count_array_len as usize];
        //  - kmer multiplier (+ 빠른 위치 검색을 위한 sym 인덱스 계산)
        let kmer_multiplier: Vec<usize> = {
            (0..self.lookup_table_kmer_size).map(|pos| {
                (total_symbol_count_in_bwt).pow(pos)
            }).rev().collect()
        };
        let index_for_each_symbol: Vec<usize> = {
            (0..(self.symbol_count + 1) as usize).map(|symidx| { // Including wild card (symbol_count + 1)
                kmer_multiplier[0] * (symidx + 1)
            }).collect()
        };
        // - kmer count array
        let mut kmer_count_array: &mut [P] = {
            let blob_start_index = self.count_array_aligned_size::<P, A>() + self.kmer_multiplier_aligned_size::<A>();
            let blob_end_index = blob_start_index + self.kmer_count_table_raw_size::<P>();
            // 0으로 init
            let body = &mut blob[blob_start_index..blob_end_index];
            body.fill(0);

            zerocopy::FromBytes::mut_from_bytes(body).unwrap()
        };
        
        // 2) Counting
        let mut table_index: usize = 0;
        text.iter_mut().rev().for_each(|sym| {
            let symidx = text_encoder.idx_of(*sym);
            // Transform sym to symidx + 1 (sentinel will be 0 for sorting)
            *sym = symidx + 1;
            // Add count to counts
            count_array[symidx as usize + 1] += P::ONE;
            // Update table_index for kmer_count_array
            table_index /= total_symbol_count_in_bwt;
            table_index += index_for_each_symbol[symidx as usize];
            // Add count to lookup table
            kmer_count_array[table_index] += P::ONE;
        });

        accumulate_count_array(&mut count_array);
        accumulate_count_array(&mut kmer_count_array);

        // 3) Write data to blob
        blob[
            ..self.count_array_raw_size::<P>()
        ].copy_from_slice(count_array.as_bytes());
        blob[
            self.count_array_aligned_size::<P, A>()
            ..self.count_array_aligned_size::<P, A>() + self.kmer_multiplier_raw_size()
        ].copy_from_slice(kmer_multiplier.as_bytes());
    }
}

fn accumulate_count_array<P: Position>(count_array: &mut [P]) {
    let mut accumulated_count = P::ZERO;
    count_array.iter_mut().for_each(|count| {
        accumulated_count += *count;
        *count = accumulated_count;
    });
}

// ================================================
// Load
// ================================================

impl<'a, P: Position> View<'a> for CountArrayView<'a, P> {
    type Header = CountArrayHeader;

    fn aligned_body_size<A: Aligned>(header: &Self::Header) -> usize {
        header.count_array_aligned_size::<P, A>()
        + header.kmer_multiplier_aligned_size::<A>()
        + header.kmer_count_table_aligned_size::<P, A>()
    }

    fn load_from_body<A: Aligned>(
        header: &Self::Header,
        body_blob: &'a [u8],
    ) -> Self {
        let mut body_start_index = 0;
        let mut body_end_index = header.count_array_raw_size::<P>();
        let mut next_body_start_index = header.count_array_aligned_size::<P, A>();

        // Count array
        let count_array_bytes = &body_blob[body_start_index..body_end_index];
        let count_array: &[P] = zerocopy::FromBytes::ref_from_bytes(count_array_bytes).unwrap();

        // Kmer multiplier
        body_start_index = next_body_start_index;
        body_end_index = body_start_index + header.kmer_multiplier_raw_size();
        next_body_start_index = body_start_index + header.kmer_multiplier_aligned_size::<A>();
        let kmer_multiplier_bytes = &body_blob[body_start_index..body_end_index];
        let kmer_multiplier: &[usize] = zerocopy::FromBytes::ref_from_bytes(kmer_multiplier_bytes).unwrap();

        // Kmer count table
        body_start_index = next_body_start_index;
        body_end_index = body_start_index + header.kmer_count_table_raw_size::<P>();
        let kmer_count_table_bytes = &body_blob[body_start_index..body_end_index];
        let kmer_count_table: &'a [P] = zerocopy::FromBytes::ref_from_bytes(kmer_count_table_bytes).unwrap();
        
        Self {
            lookup_table_kmer_size: header.lookup_table_kmer_size as usize,
            count_array: count_array.to_vec(),
            kmer_multiplier: kmer_multiplier.to_vec(),
            kmer_count_table,
        }
    }
}

// ================================================
// Locate
// ================================================
impl<'a, P: Position> CountArrayView<'a, P> {
    pub fn get_precount(&self, symidx: usize) -> P {
        self.count_array[symidx]
    }
    
    // ================================================
    // For plain text
    //  - using pattern
    pub fn get_initial_pos_range_and_idx_of_pattern<E: TextEncoder>(
        &self,
        pattern: &[u8],
        text_encoder: &E,
    ) -> ((P, P), usize) {
        let pattern_len = pattern.len();
        if pattern_len < self.lookup_table_kmer_size {
            let start_idx = self.get_idx_of_kmer_count_table_of_pattern(pattern, text_encoder);
            let gap_btw_unsearched_kmer = self.kmer_multiplier[pattern_len - 1] - 1;
            let end_idx = start_idx + gap_btw_unsearched_kmer;

            let pos_range = (self.kmer_count_table[start_idx -1], self.kmer_count_table[end_idx]);
            (pos_range, 0)
        } else {
            let sliced_pattern = &pattern[pattern.len() - self.lookup_table_kmer_size ..];
            let start_idx = self.get_idx_of_kmer_count_table_of_pattern(sliced_pattern, text_encoder);

            let pos_range = (self.kmer_count_table[start_idx -1], self.kmer_count_table[start_idx]);
            (pos_range, pattern_len - self.lookup_table_kmer_size)
        }
    }
    fn get_idx_of_kmer_count_table_of_pattern<E: TextEncoder>(
        &self,
        sliced_pattern: &[u8],
        text_encoder: &E,
    ) -> usize {
        sliced_pattern.iter().zip(self.kmer_multiplier.iter())
            .map(|(&sym, &mul_of_pos)| {
                (text_encoder.idx_of(sym) + 1) as usize * mul_of_pos
            }).sum()
    }
    //  - use reverse iter
    pub fn get_initial_pos_range_and_idx_of_pattern_rev_iter<I: Iterator<Item = u8>, E: TextEncoder>(
        &self,
        pattern_rev_iter: &mut I,
        text_encoder: &E,
    ) -> (P, P) {
        let mut sliced_pattern_size = 0;
        let mut start_idx= 0;

        while sliced_pattern_size < self.lookup_table_kmer_size {
            match pattern_rev_iter.next() {
                Some(sym) => {
                    sliced_pattern_size += 1;
                    start_idx += (text_encoder.idx_of(sym) + 1) as usize * self.kmer_multiplier[
                        self.kmer_multiplier.len() - sliced_pattern_size as usize
                    ];
                },
                None => {
                    // The pattern length can be smaller than the k-mer size.
                    // Multiply by chr_with_pidx_count.pow(self.kmer_size - sliced_pattern_size).
                    // Here, chr_with_pidx_count = self.count_table.len().
                    start_idx *= self.count_array.len().pow((self.lookup_table_kmer_size - sliced_pattern_size) as u32);

                    let gap_btw_unsearched_kmer = self.kmer_multiplier[sliced_pattern_size as usize - 1] - 1;
                    let end_idx = start_idx + gap_btw_unsearched_kmer;

                    let pos_range = (
                        self.kmer_count_table[start_idx -1],
                        self.kmer_count_table[end_idx],
                    );
                    return pos_range
                },
            };
        }

        let pos_range = (
            self.kmer_count_table[start_idx -1],
            self.kmer_count_table[start_idx],
        );
        pos_range
    }

    // ================================================
    // For symbol indices
    pub fn get_initial_pos_range_and_idx_of_indices(
        &self,
        indices: &[u8],
    ) -> ((P, P), usize) {
        let pattern_len = indices.len();
        if pattern_len < self.lookup_table_kmer_size {
            let start_idx = self.get_idx_of_kmer_count_table_of_indices(indices,);
            let gap_btw_unsearched_kmer = self.kmer_multiplier[pattern_len - 1] - 1;
            let end_idx = start_idx + gap_btw_unsearched_kmer;

            let pos_range = (self.kmer_count_table[start_idx -1], self.kmer_count_table[end_idx]);
            (pos_range, 0)
        } else {
            let sliced_indices = &indices[indices.len() - self.lookup_table_kmer_size ..];
            let start_idx = self.get_idx_of_kmer_count_table_of_indices(sliced_indices);

            let pos_range = (self.kmer_count_table[start_idx -1], self.kmer_count_table[start_idx]);
            (pos_range, pattern_len - self.lookup_table_kmer_size)
        }
    }
    fn get_idx_of_kmer_count_table_of_indices(
        &self,
        sliced_indices: &[u8],
    ) -> usize {
        sliced_indices.iter().zip(self.kmer_multiplier.iter())
            .map(|(&idx, &mul_of_pos)| {
                (idx + 1) as usize * mul_of_pos
            }).sum()
    }
    pub fn get_initial_pos_range_and_idx_of_indices_rev_iter<I: Iterator<Item = u8>>(
        &self,
        indices_rev_iter: &mut I,
    ) -> (P, P) {
        let mut sliced_pattern_size = 0;
        let mut start_idx= 0;

        while sliced_pattern_size < self.lookup_table_kmer_size {
            match indices_rev_iter.next() {
                Some(idx) => {
                    sliced_pattern_size += 1;
                    start_idx += (idx + 1) as usize * self.kmer_multiplier[
                        self.kmer_multiplier.len() - sliced_pattern_size as usize
                    ];
                },
                None => {
                    // The pattern length can be smaller than the k-mer size.
                    // Multiply by chr_with_pidx_count.pow(self.kmer_size - sliced_pattern_size).
                    // Here, chr_with_pidx_count = self.count_table.len().
                    start_idx *= self.count_array.len().pow((self.lookup_table_kmer_size - sliced_pattern_size) as u32);

                    let gap_btw_unsearched_kmer = self.kmer_multiplier[sliced_pattern_size as usize - 1] - 1;
                    let end_idx = start_idx + gap_btw_unsearched_kmer;

                    let pos_range = (
                        self.kmer_count_table[start_idx -1],
                        self.kmer_count_table[end_idx],
                    );
                    return pos_range
                },
            };
        }

        let pos_range = (
            self.kmer_count_table[start_idx -1],
            self.kmer_count_table[start_idx],
        );
        pos_range
    }
}
