<div align="center">

# sview-fmindex

[![crates.io](https://img.shields.io/crates/v/sview-fmindex.svg?style=flat-square)](https://crates.io/crates/sview-fmindex)
[![CI](https://img.shields.io/github/actions/workflow/status/baku4/sview-fmindex/ci.yml?style=flat-square)](https://github.com/baku4/sview-fmindex/actions/workflows/ci.yml)
[![license](https://img.shields.io/github/license/baku4/sview-fmindex?style=flat-square)](https://github.com/baku4/sview-fmindex/blob/main/LICENSE)

***Data is in single blob, and FM-index is slice view.***

</div>

`sview-fmindex` is a Rust library for building FM-index into pre-allocated blob and using it with minimal copying through slice view.

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
use sview_fmindex::blocks::Block2; // Block2 can index 4 symbols
use sview_fmindex::text_encoders::EncodingTable;

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
```

## Benchmarks

### 1 Gbp nucleotide text / 20 bp patterns (Cold Start)

Benchmarks comparing `mmap` vs full in-memory loading with **page cache cleared** before each test.

#### Cold = 1% (99% repeated patterns)
| Patterns | Blob | Elapsed (s) | Index Load (%) | Max RSS |
|----------|------|-------------|----------------|---------|
| 10 | in-memory | 3.39 | 95 | 2.63 GiB |
| | **mmap** | **0.30** | 6 | 5.4 MB |
| 1,000 | in-memory | 3.42 | 95 | 2.63 GiB |
| | **mmap** | **2.21** | 1 | 29 MB |
| 100,000 | **in-memory** | **3.55** | 91 | 2.63 GiB |
| | mmap | 28.52 | 0 | 656 MB |

#### Cold = 10% (90% repeated patterns)
| Patterns | Blob | Elapsed (s) | Index Load (%) | Max RSS |
|----------|------|-------------|----------------|---------|
| 10 | in-memory | 3.37 | 96 | 2.63 GiB |
| | **mmap** | **0.18** | 7 | 5.4 MB |
| 1,000 | **in-memory** | **3.35** | 96 | 2.63 GiB |
| | mmap | 8.02 | 0 | 197 MB |
| 100,000 | **in-memory** | **3.56** | 91 | 2.63 GiB |
| | mmap | 67.53 | 0 | 1.15 GiB |

#### Cold = 100% (all unique patterns)
| Patterns | Blob | Elapsed (s) | Index Load (%) | Max RSS |
|----------|------|-------------|----------------|---------|
| 10 | in-memory | 3.40 | 96 | 2.63 GiB |
| | **mmap** | **1.85** | 0 | 29 MB |
| 1,000 | **in-memory** | **3.35** | 96 | 2.63 GiB |
| | mmap | 28.17 | 0 | 657 MB |
| 100,000 | **in-memory** | **3.66** | 88 | 2.63 GiB |
| | mmap | 131.15 | 0 | 2.39 GiB |

#### When to use `mmap`
- Few queries
- Many repeated patterns
- Memory constrained

#### When to use full in-memory
- Many unique queries
- Batch processing (page faults in mmap cause significant overhead)

<details>
<summary>Test setup</summary>

- **Data**
  - **Text:** 1 Gbp random nucleotide (seed: 42)
  - **Patterns:** 20 bp, extracted from text
  - **Cold ratio:** percentage of unique patterns (rest are repeated)
  - **Index:** Position: `u32`, Block: `Block3<u64>`, SA sampling ratio: 2, kLTS: 3
- **Environment**
  - **CPU:** Intel Xeon E5-2680 v4 @ 2.40 GHz
  - **Memory:** 256 GiB
  - **OS:** Ubuntu 20.04.2 LTS (5.4.0-171-generic)
  - **Page cache:** Cleared (`echo 3 > /proc/sys/vm/drop_caches`) before each test
- **Reproduce:** `cd bench && sudo ./run_benchmark.sh`
- **Note:** Performance is nearly identical to [`lt-fm-index`](https://github.com/baku4/lt-fm-index) when using in-memory loading.
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
