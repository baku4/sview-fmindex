use crate::{
    FmIndex, FmIndexBuilder, Position,
    build_config::{LookupTableConfig, SuffixArrayConfig},
    Block, blocks::{Block2, Block3, Block4, Block5, Block6},
    text_encoders::EncodingTable,
};
use crate::tests::{
    random_data::{
        gen_rand_chr_list,
        gen_rand_text,
        gen_rand_pattern,
    },
};

fn assert_config_invariance<P: Position, B: Block>(
    symbols: &[&[u8]],
    text: Vec<u8>,
    pattern: &Vec<u8>,
    answer: &Vec<u64>,
    lt_config: LookupTableConfig,
    sa_config: SuffixArrayConfig,
) {
    if B::MAX_SYMBOL < symbols.len() as u32 {
        return;
    }
    
    let builder = FmIndexBuilder::<P, B, EncodingTable>::new(text.len(), symbols.len() as u32, EncodingTable::new(symbols)).unwrap()
        .set_lookup_table_config(lt_config).unwrap()
        .set_suffix_array_config(sa_config).unwrap();

    let blob_size = builder.blob_size();
    let mut blob: Vec<u8> = vec![0; blob_size];

    builder.build(text, &mut blob).unwrap();

    let fm_index = FmIndex::<P, B, EncodingTable>::load(&blob).unwrap();

    let mut result: Vec<u64> = fm_index.locate(pattern).into_iter().map(|x| x.as_u64()).collect();
    result.sort();
    assert_eq!(&result, answer);
}

#[test]
fn test_config_invariance() {
    // Test range
    let lookup_table_configs = [
        LookupTableConfig::None,
        LookupTableConfig::KmerSize(2),
        LookupTableConfig::KmerSize(3),
        LookupTableConfig::KmerSize(4),
    ];
    let suffix_array_configs = [
        SuffixArrayConfig::Uncompressed,
        SuffixArrayConfig::Compressed(2),
        SuffixArrayConfig::Compressed(3),
        SuffixArrayConfig::Compressed(4),
    ];

    let symbol_count_range = [4, 6, 8, 10];
    let num_tests_for_symbol_count = 3;

    // Test each symbol count
    for symbol_count in symbol_count_range {
        println!(" - Symbol count: {}", symbol_count);

        // Test each configuration
        for test_idx in 0..num_tests_for_symbol_count {
            println!("   - Test: {}/{}", test_idx, num_tests_for_symbol_count);
            // Generate random data
            let chr_list = gen_rand_chr_list(symbol_count);
            let symbols = chr_list.chunks(1).map(|c| c).collect::<Vec<_>>();
            let text = gen_rand_text(&chr_list, 1000, 1000);
            let pattern = gen_rand_pattern(&text, 10, 10);
        
            // Get Answer using default configuration
            let base_builder = FmIndexBuilder::<u32, Block4<u32>, EncodingTable>::new(text.len(), symbols.len() as u32, EncodingTable::new(&symbols)).unwrap();
            let blob_size = base_builder.blob_size();
            let mut blob: Vec<u8> = vec![0; blob_size];
            base_builder.build(text.clone(), &mut blob).unwrap();
            let base_fm_index = FmIndex::<u32, Block4<u32>, EncodingTable>::load(&blob).unwrap();
            let mut answer: Vec<u64> = base_fm_index.locate(&pattern).into_iter().map(|x| x.as_u64()).collect();
            answer.sort();
        
            macro_rules! test_type_of {
                ( $p: ty, $b: ident, $v: ty ) => {
                    for &lt_config in &lookup_table_configs {
                        for &sa_config in &suffix_array_configs {
                            if lt_config == LookupTableConfig::default() && sa_config == SuffixArrayConfig::default() {
                                continue;
                            }
                            assert_config_invariance::<$p, $b::<$v>>(
                                &symbols,
                                text.clone(),
                                &pattern,
                                &answer,
                                lt_config,
                                sa_config,
                            );
                        }
                    }
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