use super::{FmIndex, Position, Block};

/// Locate pattern using plain text
impl<'a, P: Position, B: Block> FmIndex<'a, P, B> {
    pub fn count_text(&self, text: &[u8]) -> P {
        let pos_range = self.get_pos_range_of_text(text);
        pos_range.1 - pos_range.0
    }
    pub fn locate_text(&self, text: &[u8]) -> Vec<P> {
        let pos_range = self.get_pos_range_of_text(text);
        self.get_locations(pos_range)
    }
    pub fn count_text_rev_iter<I: Iterator<Item = u8>>(&self, text_rev_iter: I) -> P {
        let pos_range = self.get_pos_range_from_text_rev_iter(text_rev_iter);
        pos_range.1 - pos_range.0
    }
    pub fn locate_text_rev_iter<I: Iterator<Item = u8>>(&self, text_rev_iter: I) -> Vec<P> {
        let pos_range = self.get_pos_range_from_text_rev_iter(text_rev_iter);
        self.get_locations(pos_range)
    }

    // Get the position range of the text
    fn get_pos_range_of_text(&self, text: &[u8]) -> (P, P) {
        let (mut pos_range, mut idx) = self.count_array_view.get_initial_pos_range_and_idx_of_text(
            text,
            &self.encoding_table,
        );
        // LF mapping
        while pos_range.0 < pos_range.1 && idx > 0 {
            idx -= 1;
            let next_sym = text[idx];
            pos_range = self.next_pos_range_of_text(pos_range, next_sym);
        }
        pos_range
    }
    fn get_pos_range_from_text_rev_iter<I: Iterator<Item = u8>>(
        &self,
        mut text_rev_iter: I,
    ) -> (P, P) {
        let mut pos_range = self.count_array_view.get_initial_pos_range_and_idx_of_text_rev_iter(
            &mut text_rev_iter,
            &self.encoding_table,
        );
        // LF mapping
        while pos_range.0 < pos_range.1  {
            match text_rev_iter.next() {
                Some(next_sym) => {
                    pos_range = self.next_pos_range_of_text(pos_range, next_sym);
                },
                None => break,
            };
        }
        pos_range
    }

    fn next_pos_range_of_text(&self, pos_range: (P, P), sym: u8) -> (P, P) {
        let symidx = self.encoding_table.idx_of(sym);
        let precount = self.count_array_view.get_precount(symidx as usize);
        let start_rank = self.bwm_view.get_next_rank(pos_range.0, symidx);
        let end_rank = self.bwm_view.get_next_rank(pos_range.1, symidx);
        (precount + start_rank, precount + end_rank)
    }
}
