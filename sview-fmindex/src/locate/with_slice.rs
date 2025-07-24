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
}
