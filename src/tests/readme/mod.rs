#[test]
#[allow(unused_variables)]
fn example() {

use crate::{FmIndexBuilder, FmIndex};
use crate::blocks::Block2; // Block2 can index 3 types of symbols

// (1) Define characters to use
let symbols: &[&[u8]] = &[
    &[b'A', b'a'], // 'A' and 'a' are treated as the same
    &[b'C', b'c'], // 'C' and 'c' are treated as the same
    &[b'G', b'g'], // 'G' and 'g' are treated as the same
];
// Alternatively, you can use this simpler syntax:
let symbols: &[&[u8]] = &[
    b"Aa", b"Cc", b"Gg"
];

// (2) Build index
let text = b"CTCCGTACACCTGTTTCGTATCGGAXXYYZZ".to_vec();
let builder = FmIndexBuilder::<u32, Block2<u64>>::init(text.len(), symbols).unwrap();
// You have to prepare a blob to build the index.
let blob_size = builder.blob_size();
let mut blob = vec![0; blob_size];
// Build the fm-index to the blob.
builder.build(text, &mut blob).unwrap();
// Load the fm-index from the blob.
let fm_index = FmIndex::<u32, Block2<u64>>::load(&blob[..]).unwrap();

// (3) Match with pattern
let pattern = b"TA";
//   - count
let count = fm_index.count_text(pattern);
assert_eq!(count, 2);
//   - locate
let mut locations = fm_index.locate_text(pattern);
locations.sort();  // The locations may not be in order.
assert_eq!(locations, vec![5,18]);
// All unindexed characters are treated as the same character.
// In the text, X, Y, and Z can match any other unindexed character
let mut locations = fm_index.locate_text(b"UNDEF");
locations.sort();
// Using the b"XXXXX", b"YYYYY", or b"!@#$%" gives the same result.
assert_eq!(locations, vec![25,26]);

}
