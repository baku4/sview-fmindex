use super::{FmIndex, Position, Block, TextEncoder};

mod with_slice;
mod with_rev_iter;

impl<'a, P: Position, B: Block, E: TextEncoder> FmIndex<'a, P, B, E> {
    fn get_locations(&self, pos_range: (P, P)) -> Vec<P> {
        let mut locations: Vec<P> = Vec::with_capacity((pos_range.1 - pos_range.0).as_usize());

        self.write_locations_to_buffer(pos_range, &mut locations);
        locations
    }
    #[inline]
    fn write_locations_to_buffer(
        &self,
        pos_range: (P, P),
        locations: &mut Vec<P>,
    ) {
        'each_pos: for mut pos in P::as_vec_in_range(&pos_range.0, &pos_range.1) {
            let mut offset: P = P::ZERO;
            while pos % self.suffix_array_view.sampling_ratio() != P::ZERO { 
                match self.bwm_view.get_pre_rank_and_symidx(pos) {
                    Some((rank, symidx)) => {
                        let precount = self.count_array_view.get_precount(symidx as usize);
                        pos = precount + rank;
                    },
                    None => { // if position == pidx
                        locations.push(offset);
                        continue 'each_pos;
                    }
                }
                offset += P::ONE;
            }
            let location = self.suffix_array_view.get_location_of(pos) + offset;
            locations.push(location);
        }
    }
    #[inline]
    fn next_pos_range(&self, pos_range: (P, P), sym: u8) -> (P, P) {
        let symidx = self.text_encoder.idx_of(sym);
        let precount = self.count_array_view.get_precount(symidx as usize);
        let start_rank = self.bwm_view.get_next_rank(pos_range.0, symidx);
        let end_rank = self.bwm_view.get_next_rank(pos_range.1, symidx);
        (precount + start_rank, precount + end_rank)
    }
}
