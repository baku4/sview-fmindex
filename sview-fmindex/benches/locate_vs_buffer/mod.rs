use criterion::{
    black_box, Criterion, BenchmarkId,
    PlotConfiguration, AxisScale,
};
use sview_fmindex::{
    FmIndex, blocks::{
        Block2, Block3, Block4,
    }, Position, Block,
    build_config::{LookupTableConfig, SuffixArrayConfig},
    FmIndexBuilder,
};
use super::random_data::{
    gen_rand_text,
    gen_rand_pattern,
};
use std::time::{Duration, Instant};

#[inline]
fn locate_multiple_patterns<P: Position, B: Block>(
    fi: &FmIndex<P, B>,
    patterns: &[Vec<u8>]
) {
    patterns.iter().for_each(|pattern| {
        _ = fi.locate_pattern(pattern);
    });
}

#[inline]
fn locate_multiple_patterns_to_buffer<P: Position, B: Block>(
    fi: &FmIndex<P, B>,
    patterns: &[Vec<u8>]
) {
    let mut buffer = Vec::new();
    patterns.iter().for_each(|pattern| {
        buffer.clear();
        fi.locate_pattern_to_buffer(pattern, &mut buffer);
    });
}

pub fn compare_locate_vs_buffer(c: &mut Criterion) {
    let mut group = c.benchmark_group("locate_vs_buffer");

    let text_len = 100_000;
    let text = gen_rand_text(b"ACGT", text_len, text_len);

    let n_patterns = 1_000;
    let pattern_length = [4, 6, 8, 10];
    let patterns_by_length: Vec<Vec<Vec<u8>>> = pattern_length.iter().map(|l| {
        let patterns = (0..n_patterns).map(|_| {
            gen_rand_pattern(&text, *l, *l)
        }).collect();
        patterns
    }).collect();   

    let ss_list = [4];
    let lk_list = [4];

    let characters_by_index: &[&[u8]] = &[b"A", b"C", b"G"];

    for ss in ss_list {
        for lk in lk_list {
            println!("# SS: {}, LK: {}", ss, lk);
            macro_rules! TestCode {
                ( $pos: ty, $blk: ty, $tagprefix: tt ) => {
                    {
                        let tag = format!("{}_ss{}_lk{}", $tagprefix, ss, lk);
                        let start = Instant::now();
                        let builder = FmIndexBuilder::<$pos, $blk>::new(text.len(), &characters_by_index).unwrap()
                            .set_suffix_array_config(SuffixArrayConfig::Compressed(ss)).unwrap()
                            .set_lookup_table_config(LookupTableConfig::KmerSize(lk)).unwrap();
                        let blob_size = builder.blob_size();
                        let mut blob = vec![0; blob_size];
                        builder.build(text.clone(), &mut blob).unwrap();
                        let fi = FmIndex::<$pos, $blk>::load(&blob).unwrap();
                        let duration = start.elapsed();
                        println!(" - {}: built in {:?}s", tag, duration);

                        for (pattern_len, patterns) in pattern_length.iter().zip(patterns_by_length.iter()) {
                            group.bench_with_input(
                                BenchmarkId::new(format!("{}_locate_pattern", tag), pattern_len),
                                &pattern_len,
                                |b, _i| b.iter(|| {
                                    locate_multiple_patterns(
                                        black_box(&fi),
                                        black_box(patterns),
                                    );
                                }
                            ));

                            group.bench_with_input(
                                BenchmarkId::new(format!("{}_locate_pattern_to_buffer", tag), pattern_len),
                                &pattern_len,
                                |b, _i| b.iter(|| {
                                    locate_multiple_patterns_to_buffer(
                                        black_box(&fi),
                                        black_box(patterns),
                                    );
                                }
                            ));
                        }
                    }
                };
            }
            TestCode!(u32, Block2<u64>, "FI_u32_b2_v64");
            TestCode!(u32, Block4<u64>, "FI_u32_b4_v64");
        }
    }

    group.finish();
} 