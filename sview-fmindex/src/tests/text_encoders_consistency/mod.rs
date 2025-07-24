// Assert that the text encoders are consistent
//  - EncodingTable
//  - PassThrough

use crate::{
    FmIndex, FmIndexBuilder, TextEncoder, Position,
    build_config::{LookupTableConfig, SuffixArrayConfig},
    Block, blocks::{Block2, Block3, Block4, Block5, Block6},
    text_encoders::{EncodingTable, PassThrough},
};
use crate::tests::{
    random_data::{
        gen_rand_chr_list,
        gen_rand_text,
        gen_rand_pattern,
    },
};

// *** For testing text encoders consistency ***
fn assert_text_encoders_consistency<P: Position, B: Block>(
    chr_list: &Vec<u8>,
    text: Vec<u8>,
    patterns: &Vec<Vec<u8>>,
    ltks: u32,
    sasr: u32,
) {
    if B::MAX_SYMBOL < chr_list.len() as u32 {
        println!("          pass");
        return;
    }
    let characters_by_index = chr_list.chunks(1).map(|c| c).collect::<Vec<_>>();
    let symbol_count = characters_by_index.len() as u32;

    // Prepare text and encoders
    let text_for_encoding_table = text.clone();
    let encoding_table = EncodingTable::new(&characters_by_index);

    let text_for_pass_through = text.into_iter().map(|c| encoding_table.idx_of(c)).collect::<Vec<_>>();
    let pass_through = PassThrough;
    
    // Build with Encoding Table
    let et_builder = FmIndexBuilder::<P, B, EncodingTable>::new(
        text_for_encoding_table.len(),
        symbol_count,
        encoding_table,
    ).unwrap()
        .set_lookup_table_config(LookupTableConfig::KmerSize(ltks)).unwrap()
        .set_suffix_array_config(SuffixArrayConfig::Compressed(sasr)).unwrap();

    let et_blob_size = et_builder.blob_size();
    let mut et_blob: Vec<u8> = vec![0; et_blob_size];

    et_builder.build(text_for_encoding_table, &mut et_blob).unwrap();

    let et_fm_index = FmIndex::<P, B, EncodingTable>::load(&et_blob).unwrap();

    // Build with Pass Through
    let pt_builder = FmIndexBuilder::<P, B, PassThrough>::new(
        text_for_pass_through.len(),
        symbol_count,
        pass_through,
    ).unwrap()
        .set_lookup_table_config(LookupTableConfig::KmerSize(ltks)).unwrap()
        .set_suffix_array_config(SuffixArrayConfig::Compressed(sasr)).unwrap();

    let pt_blob_size = pt_builder.blob_size();
    let mut pt_blob: Vec<u8> = vec![0; pt_blob_size];

    pt_builder.build(text_for_pass_through, &mut pt_blob).unwrap();

    let pt_fm_index = FmIndex::<P, B, PassThrough>::load(&pt_blob).unwrap();

    // Test methods
    let encoding_table = EncodingTable::new(&characters_by_index);
    patterns.iter().for_each(|pattern| {
        // Test count methods
        let et_count = et_fm_index.count(pattern);
        let et_count_rev_iter = et_fm_index.count_rev_iter(pattern.iter().rev().cloned());
        
        // Convert pattern to indices
        let pattern_indices: Vec<u8> = pattern.iter().map(|&c| encoding_table.idx_of(c)).collect();
        let pt_count = pt_fm_index.count(&pattern_indices);
        let pt_count_rev_iter = pt_fm_index.count_rev_iter(pattern_indices.iter().rev().cloned());
        
        // Assert all count methods return the same result
        assert_eq!(et_count, et_count_rev_iter, "Count methods should return same result for pattern: {:?}", pattern);
        assert_eq!(et_count, pt_count, "Count methods should return same result for pattern: {:?}", pattern);
        assert_eq!(et_count, pt_count_rev_iter, "Count methods should return same result for pattern: {:?}", pattern);

        // Test locate methods
        let mut et_locate = et_fm_index.locate(pattern);
        et_locate.sort();
        
        let mut et_locate_rev_iter = et_fm_index.locate_rev_iter(pattern.iter().rev().cloned());
        et_locate_rev_iter.sort();
        
        let mut pt_locate = pt_fm_index.locate(&pattern_indices);
        pt_locate.sort();
        
        let mut pt_locate_rev_iter = pt_fm_index.locate_rev_iter(pattern_indices.iter().rev().cloned());
        pt_locate_rev_iter.sort();
        
        // Assert all locate methods return the same result
        assert_eq!(et_locate, et_locate_rev_iter, "Locate methods should return same result for pattern: {:?}", pattern);
        assert_eq!(et_locate, pt_locate, "Locate methods should return same result for pattern: {:?}", pattern);
        assert_eq!(et_locate, pt_locate_rev_iter, "Locate methods should return same result for pattern: {:?}", pattern);
    });
}

#[test]
fn text_encoders_are_consistent() {
    let range_chr_count = 2..4;
    let text_min_len = 100;
    let text_max_len = 300;
    let n_text = 2;
    let pattern_min_len = 1;
    let pattern_max_len = 10;
    let n_pattern = 100;
    let ltks = 3;
    let sasr = 2;

    for chr_count in range_chr_count {
        println!("- Chr count: {}", chr_count);
        for i in 0..n_text {
            println!("  - text: {}/{}", i+1, n_text);
            let chr_list = gen_rand_chr_list(chr_count);
            let text = gen_rand_text(&chr_list, text_min_len, text_max_len);

            let patterns: Vec<Vec<u8>> = (0..n_pattern).map(|_| {
                gen_rand_pattern(&text, pattern_min_len, pattern_max_len)
            }).collect();

            macro_rules! test_type_of {
                ( $p: ty, $b: ident, $v: ty ) => {
                    assert_text_encoders_consistency::<$p, $b::<$v>>(
                        &chr_list,
                        text.clone(),
                        &patterns,
                        ltks,
                        sasr,
                    )
                };
            }
            macro_rules! of_position_for_blocks {
                ( $( $p:ty ),* ) => {
                    $(
                        println!("      - Block: Block2");
                        for_vectors!($p, Block2);
                        println!("      - Block: Block3");
                        for_vectors!($p, Block3);
                        println!("      - Block: Block4");
                        for_vectors!($p, Block4);
                        println!("      - Block: Block5");
                        for_vectors!($p, Block5);
                        println!("      - Block: Block6");
                        for_vectors!($p, Block6);
                    )*
                };
            }
            macro_rules! for_vectors {
                ( $( $p: ty, $b: ident ),* ) => {
                    $(
                        println!("        - Vector: u32");
                        test_type_of!($p, $b, u32);
                        println!("        - Vector: u64");
                        test_type_of!($p, $b, u64);
                        println!("        - Vector: u128");
                        test_type_of!($p, $b, u128);
                    )*
                };
            }
            println!("    - Position: u32");
            of_position_for_blocks!(u32);
            println!("    - Position: u64");
            of_position_for_blocks!(u64);
        }
    }
}
