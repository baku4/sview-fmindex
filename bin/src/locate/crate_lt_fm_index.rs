use std::path::PathBuf;
use std::io::{BufRead, Write};
use lt_fm_index::{LtFmIndex, blocks::{Block2, Block3}};
use crate::locate::{create_pattern_reader, create_result_writer, write_locations_to_file};

fn locate_and_write_results<B>(
    data_dir: &PathBuf,
    blob_stem: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> 
where
    B: lt_fm_index::Block<u32>,
{
    let blob_path = data_dir.join(format!("{}.blob", blob_stem));
    
    // Blob 로딩 시간 측정
    let load_start_time = std::time::Instant::now();
    let blob = std::fs::read(&blob_path)?;
    let lt_fm_index = LtFmIndex::<u32, B>::load_from(&blob[..])?;
    let load_time = load_start_time.elapsed();
    
    let result_path = data_dir.join(format!("{}-results.txt", blob_stem));

    // Locate 처리 시간 측정
    let locate_start_time = std::time::Instant::now();
    let pattern_path = data_dir.join("pattern.txt");
    let reader = create_pattern_reader(&pattern_path)?;
    let mut writer = create_result_writer(&result_path)?;

    reader.lines().for_each(|line| {
        let pattern = line.unwrap();
        let locations = lt_fm_index.locate(pattern.as_bytes());
        // lt-fm-index는 Vec<u32>를 반환하므로 그대로 사용
        write_locations_to_file(&mut writer, &locations).unwrap();
    });

    writer.flush()?;
    let locate_time = locate_start_time.elapsed();
    
    println!("Blob loading time: {:.2?}", load_time);
    println!("Locate processing time: {:.2?}", locate_time);
    
    Ok(result_path)
}

pub fn locate_patterns(data_dir: &PathBuf, treat_t_as_wildcard: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("Locating patterns with lt-fm-index...");

    if treat_t_as_wildcard {
        let block2_path = data_dir.join("lt-fm-index-block2.blob");
        if block2_path.exists() {
            println!("Using blob file: {}", block2_path.display());
            println!("Treat T as wildcard: true (Block2)");
            let result_path = locate_and_write_results::<Block2<u64>>(data_dir, "lt-fm-index-block2")?;
            println!("Results saved to: {}", result_path.display());
        } else {
            return Err("Block2 blob file (treat_t_as_wildcard=true) not found".into());
        }
    } else {
        let block3_path = data_dir.join("lt-fm-index-block3.blob");
        if block3_path.exists() {
            println!("Using blob file: {}", block3_path.display());
            println!("Treat T as wildcard: false (Block3)");
            let result_path = locate_and_write_results::<Block3<u64>>(data_dir, "lt-fm-index-block3")?;
            println!("Results saved to: {}", result_path.display());
        } else {
            return Err("Block3 blob file (treat_t_as_wildcard=false) not found".into());
        }
    }

    Ok(())
} 