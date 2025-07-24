use super::{FmIndex, Position, Block, TextEncoder};

impl<'a, P: Position, B: Block, E: TextEncoder> FmIndex<'a, P, B, E> {
    /// Count the number of occurrences with the reverse iterator of pattern
    pub fn count_rev_iter<I: Iterator<Item = u8>>(&self, pattern_rev_iter: I) -> P {
        let pos_range = self.get_pos_range_from_rev_iter(pattern_rev_iter);
        pos_range.1 - pos_range.0
    }
    /// Locate all occurrences with the reverse iterator of pattern
    pub fn locate_rev_iter<I: Iterator<Item = u8>>(&self, pattern_rev_iter: I) -> Vec<P> {
        let pos_range = self.get_pos_range_from_rev_iter(pattern_rev_iter);
        self.get_locations(pos_range)
    }
    /// Locate all occurrences with the reverse iterator of pattern & write to buffer
    pub fn locate_rev_iter_to_buffer<I: Iterator<Item = u8>>(&self, pattern_rev_iter: I, buffer: &mut Vec<P>) {
        let pos_range = self.get_pos_range_from_rev_iter(pattern_rev_iter);
        self.write_locations_to_buffer(pos_range, buffer);
    }

    fn get_pos_range_from_rev_iter<I: Iterator<Item = u8>>(
        &self,
        mut pattern_rev_iter: I,
    ) -> (P, P) {
        let mut pos_range = self.count_array_view.get_initial_pos_range_and_idx_of_pattern_rev_iter(
            &mut pattern_rev_iter,
            &self.text_encoder,
        );
        // LF mapping
        while pos_range.0 < pos_range.1  {
            match pattern_rev_iter.next() {
                Some(next_sym) => {
                    pos_range = self.next_pos_range(pos_range, next_sym);
                },
                None => break,
            };
        }
        pos_range
    }
}
