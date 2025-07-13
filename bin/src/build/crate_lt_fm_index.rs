use std::path::PathBuf;
use std::fs::File;
use lt_fm_index::{LtFmIndex, blocks::{Block2, Block3}};

use super::{SYMBOLS_ACG, SYMBOLS_ACGT};

pub fn build_index(
    text: &[u8],
    data_dir: &PathBuf,
    sasr: usize,
    klts: usize,
    treat_t_as_wildcard: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Building lt-fm-index...");
    println!("SASR: {}, KLTS: {}", sasr, klts);

    // 문자 인덱스 설정
    let symbols = if treat_t_as_wildcard {
        SYMBOLS_ACG
    } else {
        SYMBOLS_ACGT
    };

    println!("Character set: {:?}", symbols.iter().map(|c| String::from_utf8_lossy(c)).collect::<Vec<_>>());

    if treat_t_as_wildcard {
        // Block2 사용 (ACG만 인덱싱)
        // Build time 측정
        let build_start_time = std::time::Instant::now();
        let lt_fm_index = LtFmIndex::<u32, Block2<u64>>::build(
            text.to_vec(),
            &symbols,
            sasr as u32,
            klts as u32,
        )?;
        let build_time = build_start_time.elapsed();
        println!("Build time: {:.2?}", build_time);

        // Save time 측정
        let save_start_time = std::time::Instant::now();
        let output_path = data_dir.join("lt-fm-index-block2.blob");
        let mut file = File::create(&output_path)?;
        lt_fm_index.save_to(&mut file)?;
        let save_time = save_start_time.elapsed();
        println!("Save time: {:.2?}", save_time);
        println!("Index saved to: {}", output_path.display());
    } else {
        // Block3 사용 (ACGT 모두 인덱싱)
        // Build time 측정
        let build_start_time = std::time::Instant::now();
        let lt_fm_index = LtFmIndex::<u32, Block3<u64>>::build(
            text.to_vec(),
            &symbols,
            sasr as u32,
            klts as u32,
        )?;
        let build_time = build_start_time.elapsed();
        println!("Build time: {:.2?}", build_time);

        // Save time 측정
        let save_start_time = std::time::Instant::now();
        let output_path = data_dir.join("lt-fm-index-block3.blob");
        let mut file = File::create(&output_path)?;
        lt_fm_index.save_to(&mut file)?;
        let save_time = save_start_time.elapsed();
        println!("Save time: {:.2?}", save_time);
        println!("Index saved to: {}", output_path.display());
    }

    Ok(())
} 