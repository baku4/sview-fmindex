use super::{FmIndex, Position, Block, TextEncoder};

impl<'a, P: Position, B: Block, E: TextEncoder> FmIndex<'a, P, B, E> {
    /// Count the number of occurrences with the pattern
    pub fn count(&self, pattern: &[u8]) -> P {
        let pos_range = self.get_pos_range(pattern);
        pos_range.1 - pos_range.0
    }
    /// Locate all occurrences with the pattern
    pub fn locate(&self, pattern: &[u8]) -> Vec<P> {
        let pos_range = self.get_pos_range(pattern);
        self.get_locations(pos_range)
    }
    /// Locate all occurrences with the pattern & write to buffer
    pub fn locate_to_buffer(&self, pattern: &[u8], buffer: &mut Vec<P>) {
        let pos_range = self.get_pos_range(pattern);
        self.write_locations_to_buffer(pos_range, buffer);
    }

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

    // Get the position range of the text
    fn get_pos_range(&self, pattern: &[u8]) -> (P, P) {
        let (mut pos_range, mut idx) = self.count_array_view.get_initial_pos_range_and_idx_of_pattern(
            pattern,
            &self.text_encoder,
        );
        // LF mapping
        while pos_range.0 < pos_range.1 && idx > 0 {
            idx -= 1;
            let next_sym = pattern[idx];
            pos_range = self.next_pos_range(pos_range, next_sym);
        }
        pos_range
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

    fn next_pos_range(&self, pos_range: (P, P), sym: u8) -> (P, P) {
        let symidx = self.text_encoder.idx_of(sym);
        let precount = self.count_array_view.get_precount(symidx as usize);
        let start_rank = self.bwm_view.get_next_rank(pos_range.0, symidx);
        let end_rank = self.bwm_view.get_next_rank(pos_range.1, symidx);
        (precount + start_rank, precount + end_rank)
    }
}
