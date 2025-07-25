#[test]
#[allow(unused_variables)]
fn example() {

use crate::{FmIndexBuilder, FmIndex};
use crate::blocks::Block2; // Block2 can index 4 types of symbols
use crate::text_encoders::EncodingTable;

// (1) Define symbols to use
let symbols: &[&[u8]] = &[b"Aa", b"Cc", b"Gg", b"Tt"];
let encoding_table = EncodingTable::from_symbols(symbols);
let symbol_count = encoding_table.symbol_count(); // 4

// (2) Build index
let text = b"CTCCGTACACCTGTTTCGTATCGGAXXYYZZ".to_vec();
let builder = FmIndexBuilder::<u32, Block2<u64>, EncodingTable>::new(
    text.len(),
    symbol_count,
    encoding_table,
).unwrap();
// You have to prepare a blob to build the index.
let blob_size = builder.blob_size();
let mut blob = vec![0; blob_size];
// Build the fm-index to the blob.
builder.build(text, &mut blob).unwrap();
// Load the fm-index from the blob.
let fm_index = FmIndex::<u32, Block2<u64>, EncodingTable>::load(&blob[..]).unwrap();

// (3) Match with pattern
let pattern = b"TA";
//   - count
let count = fm_index.count(pattern);
assert_eq!(count, 2);
//   - locate
let mut locations = fm_index.locate(pattern);
locations.sort();  // The locations may not be in order.
assert_eq!(locations, vec![5,18]);

// When using EncodingTable, the last symbol is treated as wild card.
// In the text, X, Y, and Z can match any other unindexed character
let mut locations = fm_index.locate(b"UNDEF");
locations.sort();
// Using the b"XXXXX", b"YYYYY", or b"!@#$%" gives the same result.
assert_eq!(locations, vec![25,26]);

}
