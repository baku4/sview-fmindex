# sview-fmindex

[![crates.io](https://img.shields.io/crates/v/sview-fmindex.svg?style=flat-square)](https://crates.io/crates/sview-fmindex)
[![license](https://img.shields.io/github/license/baku4/sview-fmindex?style=flat-square)](https://github.com/baku4/sview-fmindex/blob/main/LICENSE)

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

## Benchmarks

### 1 Gbp nucleotide text · 20 bp pattern

| Load strategy        | Avg RSS      | Peak RSS     | Blob load (ms) | Locate per pattern (ns) |
| -------------------- | ------------ | ------------ | -------------- | ----------------------- |
| Full in‑memory       | **2.82 GiB** | **4.50 GiB** | 2,388          | 1,369 ns                |
| `mmap` (no `Advice`) | **0.48 GiB** | **0.52 GiB** | 0.04           | 1,365 ns                |

<details>
<summary>Test setup</summary>

- **Data**
  - **Text:** 1 Gbp random nucleotide
  - **Patterns:** 1 000 000 × 20 bp
  - **Index:** Position: `u32`, Block: `Block2<u64>`, Uncompressed
- **Hardware**
  - **CPU** Intel Xeon E5‑2680 v4 @ 2.40 GHz
  - **Memory** 256 GiB
  - **OS (Kernel)** Ubuntu 20.04.2 LTS (5.4.0‑171‑generic)
  - **Page size** 4 KiB
</details>

## References

**Base repo**: This repository was forked from [`baku4/lt-fm-index`](https://github.com/baku4/lt-fm-index) (v0.7.0, commit `1327896`).

**FM-index implementation:**
- Ferragina, P., & Manzini, G. (2004). An Alphabet-Friendly FM-Index. *String Processing and Information Retrieval*, 150-160.
- Wang, Y., et al. (2018). Accelerating FM-index Search for Genomic Data Processing. *Proceedings of the 47th International Conference on Parallel Processing*.

***K-mer* Lookup table implementation:**
- Anderson, T., & Wheeler, T. J. (2021). An optimized FM-index library for nucleotide and amino acid search. *Algorithms for Molecular Biology*, 16(1), 25.

**Burrows-Wheeler Transform:**
- [libdivsufsort](https://github.com/y-256/libdivsufsort) by Yuta Mori.
- [Rust-Bio](https://github.com/rust-bio/rust-bio): A fast and safe bioinformatics library, introduced in Köster, J. (2016), "Rust-Bio: a fast and safe bioinformatics library," *Bioinformatics*, 32(3), 444-446.
