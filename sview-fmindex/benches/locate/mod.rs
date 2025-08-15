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
    text_encoders::EncodingTable,
};
use super::random_data::{
    gen_rand_text,
    gen_rand_pattern,
};
use std::time::{Duration, Instant};

#[inline]
fn locate_multiple_patterns<P: Position, B: Block>(
    fi: &FmIndex<P, B, EncodingTable>,
    patterns: &[Vec<u8>]
) {
    patterns.iter().for_each(|pattern| {
        _ = fi.locate(pattern);
    });
}

pub fn perf_of_locate(c: &mut Criterion) {
    let mut group = c.benchmark_group("locate");

    let text_len = 1_000; // 100_000_000;
    let text = gen_rand_text(b"ACGT", text_len, text_len);

    let n_patterns = 10; // 1000;
    let pattern_length = [10, 20, 30, 40, 50];
    let patterns_by_length: Vec<Vec<Vec<u8>>> = pattern_length.iter().map(|l| {
        let patterns = (0..n_patterns).map(|_| {
            gen_rand_pattern(&text, *l, *l)
        }).collect();
        patterns
    }).collect();   

    let ss_list = [1, 2, 4, 8];
    let lk_list = [1, 2, 4, 8];

    let characters_by_index: &[&[u8]] = &[b"A", b"C", b"G"];

    for ss in ss_list {
        for lk in lk_list {
            println!("# SS: {}, LK: {}", ss, lk);
            macro_rules! TestCode {
                ( $pos: ty, $blk: ty, $tagprefix: tt ) => {
                    {
                        let tag = format!("{}_ss{}_lk{}", $tagprefix, ss, lk);
                        let start = Instant::now();
                        let text_encoder = EncodingTable::from_symbols(&characters_by_index);
                        let builder = FmIndexBuilder::<$pos, $blk, EncodingTable>::new(text.len(), text_encoder.symbol_count(), text_encoder).unwrap()
                            .set_suffix_array_config(SuffixArrayConfig::Compressed(ss)).unwrap()
                            .set_lookup_table_config(LookupTableConfig::KmerSize(lk)).unwrap();
                        let blob_size = builder.blob_size();
                        let mut blob = vec![0; blob_size];
                        builder.build(text.clone(), &mut blob).unwrap();
                        let fi = FmIndex::<$pos, $blk, EncodingTable>::load(&blob).unwrap();
                        let duration = start.elapsed();
                        println!(" - {}: built in {:?}s", tag, duration);

                        for (pattern_len, patterns) in pattern_length.iter().zip(patterns_by_length.iter()) {
                            group.bench_with_input(
                                BenchmarkId::new(&tag, pattern_len),
                                &pattern_len,
                                |b, _i| b.iter(|| {
                                    locate_multiple_patterns(
                                        black_box(&fi),
                                        black_box(patterns),
                                    );
                                }
                            ));
                        }
                    }
                };
            }
            TestCode!(u32, Block2<u32>, "LFI_u32_b2_v32");
            TestCode!(u32, Block2<u64>, "LFI_u32_b2_v64");
            TestCode!(u32, Block2<u128>, "LFI_u32_b2_v128");
            TestCode!(u32, Block3<u32>, "LFI_u32_b3_v32");
            TestCode!(u32, Block3<u64>, "LFI_u32_b3_v64");
            TestCode!(u32, Block3<u128>, "LFI_u32_b3_v128");
            TestCode!(u32, Block4<u32>, "LFI_u32_b4_v32");
            TestCode!(u32, Block4<u64>, "LFI_u32_b4_v64");
            TestCode!(u32, Block4<u128>, "LFI_u32_b4_v128");
            TestCode!(u64, Block2<u32>, "LFI_u64_b2_v32");
            TestCode!(u64, Block2<u64>, "LFI_u64_b2_v64");
            TestCode!(u64, Block2<u128>, "LFI_u64_b2_v128");
            TestCode!(u64, Block3<u32>, "LFI_u64_b3_v32");
            TestCode!(u64, Block3<u64>, "LFI_u64_b3_v64");
            TestCode!(u64, Block3<u128>, "LFI_u64_b3_v128");
            TestCode!(u64, Block4<u32>, "LFI_u64_b4_v32");
            TestCode!(u64, Block4<u64>, "LFI_u64_b4_v64");
            TestCode!(u64, Block4<u128>, "LFI_u64_b4_v128");
        }
    }

    group.finish();
}