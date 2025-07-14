use super::{FmIndex, Position, Block};

// Plain text slice
mod pattern;
// Raw indices of each symbol in the pattern
mod indices;

impl<'a, P: Position, B: Block> FmIndex<'a, P, B> {
    fn get_locations(&self, pos_range: (P, P)) -> Vec<P> {
        let mut locations: Vec<P> = Vec::with_capacity((pos_range.1 - pos_range.0).as_usize());

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
        locations
    }
}