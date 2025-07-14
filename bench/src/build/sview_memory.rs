use std::fs;
use std::path::PathBuf;
use sview_fmindex::{
    FmIndexBuilder, blocks::{Block2, Block3},
    build_config::{LookupTableConfig, SuffixArrayConfig}
};

use super::{SYMBOLS_ACG, SYMBOLS_ACGT};

pub fn build_index(
    text: &[u8],
    data_dir: &PathBuf,
    sasr: usize,
    klts: usize,
    treat_t_as_wildcard: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Building sview-fmindex memory version...");
    println!("SASR: {}, KLTS: {}", sasr, klts);

    // 문자 인덱스 설정
    let symbols = if treat_t_as_wildcard {
        SYMBOLS_ACG
    } else {
        SYMBOLS_ACGT
    };

    println!("Character set: {:?}", 
        symbols.iter()
            .map(|c| String::from_utf8_lossy(c))
            .collect::<Vec<_>>()
    );

    // Config 설정
    let suffix_array_config = if sasr == 1 {
        SuffixArrayConfig::Uncompressed
    } else {
        SuffixArrayConfig::Compressed(sasr as u32)
    };
    let lookup_table_config = if klts == 1 {
        LookupTableConfig::None
    } else {
        LookupTableConfig::KmerSize(klts as u32)
    };

    // T를 와일드카드로 취급하면 Block2, 아니면 Block3 사용
    if treat_t_as_wildcard {
        // Block2 사용 (ACG만 인덱싱)
        let builder = FmIndexBuilder::<u32, Block2<u64>>::init(
            text.len(),
            &symbols,
        )?
        .set_suffix_array_config(suffix_array_config)?
        .set_lookup_table_config(lookup_table_config)?;

        let blob_size = builder.blob_size();
        let mut blob: Vec<u8> = vec![0; blob_size];
        println!("Blob size: {} bytes", blob_size);

        // Build time 측정
        let build_start_time = std::time::Instant::now();
        builder.build(text.to_vec(), &mut blob)?;
        let build_time = build_start_time.elapsed().as_nanos();
        println!("Build time: {} ns", build_time);

        // Save time 측정
        let save_start_time = std::time::Instant::now();
        let output_path = data_dir.join("sview-memory-block2.blob");
        fs::write(&output_path, &blob)?;
        let save_time = save_start_time.elapsed().as_nanos();
        println!("Save time: {} ns", save_time);
        println!("Index saved to: {}", output_path.display());
    } else {
        // Block3 사용 (ACGT 모두 인덱싱)
        let builder = FmIndexBuilder::<u32, Block3<u64>>::init(
            text.len(),
            &symbols,
        )?
        .set_suffix_array_config(suffix_array_config)?
        .set_lookup_table_config(lookup_table_config)?;

        let blob_size = builder.blob_size();
        let mut blob: Vec<u8> = vec![0; blob_size];
        println!("Blob size: {} bytes", blob_size);

        // Build time 측정
        let build_start_time = std::time::Instant::now();
        builder.build(text.to_vec(), &mut blob)?;
        let build_time = build_start_time.elapsed().as_nanos();
        println!("Build time: {} ns", build_time);

        // Save time 측정
        let save_start_time = std::time::Instant::now();
        let output_path = data_dir.join("sview-memory-block3.blob");
        fs::write(&output_path, &blob)?;
        let save_time = save_start_time.elapsed().as_nanos();
        println!("Save time: {} ns", save_time);
        println!("Index saved to: {}", output_path.display());
    }

    Ok(())
}