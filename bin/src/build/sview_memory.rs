use std::fs;
use std::path::PathBuf;
use sview_fmindex::{
    FmIndexBuilder, FmIndex, blocks::{Block2, Block3},
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

    // T를 와일드카드로 취급하면 Block2, 아니면 Block3 사용
    if treat_t_as_wildcard {
        // Block2 사용 (ACG만 인덱싱)
        let builder = FmIndexBuilder::<u32, Block2<u64>>::init(
            text.len(),
            &symbols,
        )?
        .set_suffix_array_config(SuffixArrayConfig::Compressed(sasr as u32))?
        .set_lookup_table_config(LookupTableConfig::KmerSize(klts as u32))?;

        let blob_size = builder.blob_size();
        let mut blob: Vec<u8> = vec![0; blob_size];
        println!("Blob size: {} bytes", blob_size);

        builder.build(text.to_vec(), &mut blob)?;

        let output_path = data_dir.join("sview-memory-block2.blob");
        fs::write(&output_path, &blob)?;
        println!("Index saved to: {}", output_path.display());

        let _loaded_index = FmIndex::<u32, Block2<u64>>::load(&blob)?;
        println!("Index loaded successfully for verification");
    } else {
        // Block3 사용 (ACGT 모두 인덱싱)
        let builder = FmIndexBuilder::<u32, Block3<u64>>::init(
            text.len(),
            &symbols,
        )?
        .set_suffix_array_config(SuffixArrayConfig::Compressed(sasr as u32))?
        .set_lookup_table_config(LookupTableConfig::KmerSize(klts as u32))?;

        let blob_size = builder.blob_size();
        let mut blob: Vec<u8> = vec![0; blob_size];
        println!("Blob size: {} bytes", blob_size);

        builder.build(text.to_vec(), &mut blob)?;

        let output_path = data_dir.join("sview-memory-block3.blob");
        fs::write(&output_path, &blob)?;
        println!("Index saved to: {}", output_path.display());

        let _loaded_index = FmIndex::<u32, Block3<u64>>::load(&blob)?;
        println!("Index loaded successfully for verification");
    }

    Ok(())
}