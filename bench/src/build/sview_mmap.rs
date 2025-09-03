use std::fs;
use std::path::PathBuf;
use memmap2::MmapMut;
use sview_fmindex::{
    FmIndexBuilder, blocks::{Block2, Block3},
    text_encoders::EncodingTable,
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
    println!("Building sview-fmindex mmap version...");
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
        let symbols: &[&[u8]] = &[b"Aa", b"Cc", b"Gg", b"Tt"];
        let encoding_table = EncodingTable::from_symbols(symbols);
        let symbol_count = encoding_table.symbol_count();
        // Block2 사용
        let builder = FmIndexBuilder::<u32, Block2<u64>, EncodingTable>::new(
            text.len(),
            symbol_count,
            encoding_table,
        )?
        .set_suffix_array_config(suffix_array_config)?
        .set_lookup_table_config(lookup_table_config)?;

        let blob_size = builder.blob_size();
        println!("Blob size: {} bytes", blob_size);

        // 파일 시스템에 할당
        let output_path = data_dir.join("sview-mmap-block2.blob");
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&output_path)?;
        
        // 파일 크기를 blob_size로 설정
        file.set_len(blob_size as u64)?;
        
        // mmap으로 메모리 매핑
        let mut mmap = unsafe { MmapMut::map_mut(&file)? };
        
        // Build time 측정
        let build_start_time = std::time::Instant::now();
        builder.build(text.to_vec(), &mut mmap)?;
        let build_time = build_start_time.elapsed().as_nanos();
        println!("Build time: {} ns", build_time);
        
        // mmap을 디스크에 동기화
        mmap.flush()?;
        
        println!("Index saved to: {}", output_path.display());
    } else {
        let symbols: &[&[u8]] = &[b"Aa", b"Cc", b"Gg", b"Tt", b"Nn"];
        let encoding_table = EncodingTable::from_symbols(symbols);
        let symbol_count = encoding_table.symbol_count();
        // Block3 사용
        let builder = FmIndexBuilder::<u32, Block3<u64>, EncodingTable>::new(
            text.len(),
            symbol_count,
            encoding_table,
        )?
        .set_suffix_array_config(SuffixArrayConfig::Compressed(sasr as u32))?
        .set_lookup_table_config(LookupTableConfig::KmerSize(klts as u32))?;

        let blob_size = builder.blob_size();
        println!("Blob size: {} bytes", blob_size);

        // 파일 시스템에 할당
        let output_path = data_dir.join("sview-mmap-block3.blob");
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&output_path)?;
        
        // 파일 크기를 blob_size로 설정
        file.set_len(blob_size as u64)?;
        
        // mmap으로 메모리 매핑
        let mut mmap = unsafe { MmapMut::map_mut(&file)? };
        
        // Build time 측정
        let build_start_time = std::time::Instant::now();
        builder.build(text.to_vec(), &mut mmap)?;
        let build_time = build_start_time.elapsed().as_nanos();
        println!("Build time: {} ns", build_time);
        
        // mmap을 디스크에 동기화
        mmap.flush()?;
        
        println!("Index saved to: {}", output_path.display());
    }

    Ok(())
} 