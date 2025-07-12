use crate::Position;
use crate::{
    FmIndex, FmIndexBuilder,
    build_config::{LookupTableConfig, SuffixArrayConfig},
    Block, blocks::{Block2, Block3, Block4, Block5, Block6},
};
use crate::tests::{
    random_data::{
        gen_rand_chr_list,
        gen_rand_text,
        gen_rand_pattern,
    },
};

// *** For testing locate method consistency ***
fn assert_locate_consistency<P: Position, B: Block>(
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
    
    let builder = FmIndexBuilder::<P, B>::init(text.len(), &characters_by_index).unwrap()
        .set_lookup_table_config(LookupTableConfig::KmerSize(ltks)).unwrap()
        .set_suffix_array_config(SuffixArrayConfig::Compressed(sasr)).unwrap();

    let blob_size = builder.blob_size();
    let mut blob: Vec<u8> = vec![0; blob_size];

    builder.build(text.clone(), &mut blob).unwrap();

    let fm_index = FmIndex::<P, B>::load(&blob).unwrap();

    // Create encoding table for converting text to indices
    let encoding_table = crate::components::EncodingTable::new(&characters_by_index);

    patterns.iter().for_each(|pattern| {
        // Test count methods
        let count_text = fm_index.count_text(pattern);
        let count_text_rev_iter = fm_index.count_text_rev_iter(pattern.iter().rev().cloned());
        
        // Convert pattern to indices
        let pattern_indices: Vec<u8> = pattern.iter().map(|&c| encoding_table.idx_of(c)).collect();
        let count_indices = fm_index.count_indices(&pattern_indices);
        let count_indices_rev_iter = fm_index.count_indices_rev_iter(pattern_indices.iter().rev().cloned());
        
        // Assert all count methods return the same result
        assert_eq!(count_text, count_text_rev_iter, "Count methods should return same result for pattern: {:?}", pattern);
        assert_eq!(count_text, count_indices, "Count methods should return same result for pattern: {:?}", pattern);
        assert_eq!(count_text, count_indices_rev_iter, "Count methods should return same result for pattern: {:?}", pattern);

        // Test locate methods
        let mut locate_text = fm_index.locate_text(pattern);
        locate_text.sort();
        
        let mut locate_text_rev_iter = fm_index.locate_text_rev_iter(pattern.iter().rev().cloned());
        locate_text_rev_iter.sort();
        
        let mut locate_indices = fm_index.locate_indices(&pattern_indices);
        locate_indices.sort();
        
        let mut locate_indices_rev_iter = fm_index.locate_indices_rev_iter(pattern_indices.iter().rev().cloned());
        locate_indices_rev_iter.sort();
        
        // Assert all locate methods return the same result
        assert_eq!(locate_text, locate_text_rev_iter, "Locate methods should return same result for pattern: {:?}", pattern);
        assert_eq!(locate_text, locate_indices, "Locate methods should return same result for pattern: {:?}", pattern);
        assert_eq!(locate_text, locate_indices_rev_iter, "Locate methods should return same result for pattern: {:?}", pattern);
    });
}

#[test]
fn locate_methods_are_consistent() {
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
                    assert_locate_consistency::<$p, $b::<$v>>(
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
