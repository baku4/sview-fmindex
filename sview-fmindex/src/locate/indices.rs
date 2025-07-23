use super::{FmIndex, Position, Block};

impl<'a, P: Position, B: Block> FmIndex<'a, P, B> {
    /// Count the number of occurrences with the indices
    pub fn count_indices(&self, indices: &[u8]) -> P {
        let pos_range = self.get_pos_range_of_indices(indices);
        pos_range.1 - pos_range.0
    }
    /// Locate all occurrences with the indices
    pub fn locate_indices(&self, indices: &[u8]) -> Vec<P> {
        let pos_range = self.get_pos_range_of_indices(indices);
        self.get_locations(pos_range)
    }
    /// Locate all occurrences with the indices & write to buffer
    pub fn locate_indices_to_buffer(&self, indices: &[u8], buffer: &mut Vec<P>) {
        let pos_range = self.get_pos_range_of_indices(indices);
        self.write_locations_to_buffer(pos_range, buffer);
    }

    /// Count the number of occurrences with the reverse iterator of indices
    pub fn count_indices_rev_iter<I: Iterator<Item = u8>>(&self, indices_rev_iter: I) -> P {
        let pos_range = self.get_pos_range_from_indices_rev_iter(indices_rev_iter);
        pos_range.1 - pos_range.0
    }
    /// Locate all occurrences with the reverse iterator of indices
    pub fn locate_indices_rev_iter<I: Iterator<Item = u8>>(&self, indices_rev_iter: I) -> Vec<P> {
        let pos_range = self.get_pos_range_from_indices_rev_iter(indices_rev_iter);
        self.get_locations(pos_range)
    }
    /// Locate all occurrences with the reverse iterator of indices & write to buffer
    pub fn locate_indices_rev_iter_to_buffer<I: Iterator<Item = u8>>(&self, indices_rev_iter: I, buffer: &mut Vec<P>) {
        let pos_range = self.get_pos_range_from_indices_rev_iter(indices_rev_iter);
        self.write_locations_to_buffer(pos_range, buffer);
    }

    // Get the position range of the indices
    fn get_pos_range_of_indices(&self, indices: &[u8]) -> (P, P) {
        let (mut pos_range, mut idx) = self.count_array_view.get_initial_pos_range_and_idx_of_indices(
            indices,
        );
        // LF mapping
        while pos_range.0 < pos_range.1 && idx > 0 {
            idx -= 1;
            let next_idx = indices[idx];
            pos_range = self.next_pos_range_of_indices(pos_range, next_idx);
        }
        pos_range
    }
    fn get_pos_range_from_indices_rev_iter<I: Iterator<Item = u8>>(
        &self,
        mut indices_rev_iter: I,
    ) -> (P, P) {
        let mut pos_range = self.count_array_view.get_initial_pos_range_and_idx_of_indices_rev_iter(
            &mut indices_rev_iter,
        );
        // LF mapping
        while pos_range.0 < pos_range.1  {
            match indices_rev_iter.next() {
                Some(next_idx) => {
                    pos_range = self.next_pos_range_of_indices(pos_range, next_idx);
                },
                None => break,
            };
        }
        pos_range
    }

    fn next_pos_range_of_indices(&self, pos_range: (P, P), symidx: u8) -> (P, P) {
        let precount = self.count_array_view.get_precount(symidx as usize);
        let start_rank = self.bwm_view.get_next_rank(pos_range.0, symidx);
        let end_rank = self.bwm_view.get_next_rank(pos_range.1, symidx);
        (precount + start_rank, precount + end_rank)
    }
}