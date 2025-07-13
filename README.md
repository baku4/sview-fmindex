# sview-fmindex

***Data is in single blob, and fm-index is slice view.***

`sview-fmindex` is a Rust library for building FM-indexes into pre-allocated blobs and using them with minimal copying through slice views.

- `-fmindex`: FM-index is a compressed text index that provides two main operations:
  1) **Count** the number of occurrences in text
  2) **Locate** the positions in text
- `sview-`: In this library, data is stored in one contiguous blob, and the FM-index structure is created as a slice view into this blob.

## Architecture

```
           builder
          ┌────────┐
          │ header │
          └────────┘
               │
               |-(build with text)
 blob          ▼
┌────────┬──────────────────────┐
│ header │     body (LARGE)     │
└────────┴──────────────────────┘
               │
               │-(load index)
      fm-index ▼
      ┌────────┬────────┐
      │ header │  view  │
      └────────┴────────┘
```

## Usage

### Basic Example

```rust
use sview_fmindex::{FmIndexBuilder, FmIndex};
use sview_fmindex::blocks::Block2; // Block2 can index 3 types of symbols

// (1) Prepare Data
let text = b"CTCCGTACACCTGTTTCGTATCGGAXXYYZZ".to_vec();
let symbols: &[&[u8]] = &[
    b"Aa", b"Cc", b"Gg"
];

// (2) Build index
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
let count = fm_index.count_pattern(pattern);
assert_eq!(count, 2);
//   - locate
let mut locations = fm_index.locate_pattern(pattern);
locations.sort();  // The locations may not be in order.
assert_eq!(locations, vec![5,18]);
// All unindexed characters are treated as the same character.
// In the text, X, Y, and Z can match any other unindexed character
let mut locations = fm_index.locate_pattern(b"UNDEF");
locations.sort();
// Using the b"XXXXX", b"YYYYY", or b"!@#$%" gives the same result.
assert_eq!(locations, vec![25,26]);
```
