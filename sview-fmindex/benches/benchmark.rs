#![allow(dead_code)]
#![allow(unused_imports)]
use criterion::{
    criterion_group, criterion_main, Criterion,
};

mod random_data;

// Bench counting bits
mod counting_bit;
use counting_bit::bench_counting_bits_of_u64;

// TODO: Sort algorithm
#[cfg(feature = "fastbwt")]
mod sorting;
#[cfg(feature = "fastbwt")]
use sorting::bench_burrow_wheeler_transform;

// Bench of locating by options
mod locate;
use locate::perf_of_locate;

// Locate with raw index
mod locate_with_raw_index;
use locate_with_raw_index::compare_locate_vs_locate_from_raw_index;

// Locate vs buffer
mod locate_vs_buffer;
use locate_vs_buffer::compare_locate_vs_buffer;

// Memory vs disk mmap
mod memory_vs_disk_mmap;
use memory_vs_disk_mmap::{
    bench_memory_vs_disk_mmap_locate_u32_block2,
    bench_memory_vs_disk_mmap_locate_u64_block2,
    bench_memory_vs_disk_mmap_locate_u64_block4,
};

criterion_group!(
    benches,
    compare_locate_vs_buffer,
);
#[cfg(feature = "fastbwt")]
criterion_group!(
    benches,
    bench_burrow_wheeler_transform,
);
criterion_main!(benches);
