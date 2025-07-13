use std::fs;
use std::path::PathBuf;
use memmap2::MmapMut;
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
        
        // 빌드 실행 (mmap 슬라이스에 직접 쓰기)
        builder.build(text.to_vec(), &mut mmap)?;
        
        // mmap을 디스크에 동기화
        mmap.flush()?;
        
        println!("Index saved to: {}", output_path.display());
    } else {
        // Block3 사용 (ACGT 모두 인덱싱)
        let builder = FmIndexBuilder::<u32, Block3<u64>>::init(
            text.len(),
            &symbols,
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
        
        // 빌드 실행 (mmap 슬라이스에 직접 쓰기)
        builder.build(text.to_vec(), &mut mmap)?;
        
        // mmap을 디스크에 동기화
        mmap.flush()?;
        
        println!("Index saved to: {}", output_path.display());
    }

    Ok(())
} 