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

fn get_decoding_table<P: Position, B: Block>(fi: &FmIndex<P, B>) -> Vec<u8> {
    let encoding_table = fi.get_encoding_table();
    let mut decoding_table = vec![0; 256];
    for (i, c) in encoding_table.iter().enumerate() {
        decoding_table[*c as usize] = i as u8;
    }
    decoding_table
}

#[inline]
fn locate_multiple_patterns<P: Position, B: Block>(
    lfi: &FmIndex<P, B>,
    patterns: &[Vec<u8>]
) {
    patterns.iter().for_each(|pattern| {
        _ = lfi.locate_pattern(pattern);
    });
}

#[inline]
fn locate_multiple_patterns_from_raw_index<P: Position, B: Block>(
    fi: &FmIndex<P, B>,
    patterns: &[Vec<u8>]
) {
    patterns.iter().for_each(|pattern| {
        let decoding_table = get_decoding_table(fi);
        let raw_index_rev_iter = pattern.iter().map(|&c| decoding_table[c as usize]).rev();
        _ = fi.locate_pattern_rev_iter(raw_index_rev_iter);
    });
}

pub fn compare_locate_vs_locate_from_raw_index(c: &mut Criterion) {
    let mut group = c.benchmark_group("locate_vs_locate_from_raw_index");

    let text_len = 10_000;
    let text = gen_rand_text(b"ACG", text_len, text_len);

    let n_patterns = 100;
    let pattern_length = [50]; // [10, 20, 30, 40, 50];
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
                ( $pos: ty, $blk: ty, $tagprefix: tt) => {
                    {
                        let tag = format!("{}_ss{}_lk{}", $tagprefix, ss, lk);

                        // Build FM-index
                        let start = Instant::now();
                        let builder = FmIndexBuilder::<$pos, $blk>::init(text.len(),  &characters_by_index).unwrap()
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
                                BenchmarkId::new(format!("{}_locate", tag), pattern_len),
                                &pattern_len,
                                |b, _i| b.iter(|| {
                                    locate_multiple_patterns(
                                        black_box(&fi),
                                        black_box(patterns),
                                    );
                                }
                            ));

                            group.bench_with_input(
                                BenchmarkId::new(format!("{}_locate_from_raw_index", tag), pattern_len),
                                &pattern_len,
                                |b, _i| b.iter(|| {
                                    locate_multiple_patterns_from_raw_index(
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

            TestCode!(u64, Block2<u64>, "FI_u64_b2_v64");
            TestCode!(u64, Block4<u64>, "FI_u64_b4_v64");
        }
    }

    group.finish();
}