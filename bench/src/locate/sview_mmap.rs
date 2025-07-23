use std::path::PathBuf;
use std::io::{BufRead, Write};
use std::fs;
use memmap2::Mmap;
use sview_fmindex::{FmIndex, Block, blocks::{Block2, Block3}};
use crate::locate::{create_pattern_reader, create_result_writer, write_locations_to_file};

fn locate_and_write_results<B: Block>(
    data_dir: &PathBuf,
    blob_stem: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let blob_path = data_dir.join(format!("{}.blob", blob_stem));
    
    // Blob 로딩 시간 측정
    let load_start_time = std::time::Instant::now();
    let file = fs::File::open(&blob_path)?;
    let mmap = unsafe { Mmap::map(&file)? };
    
    #[cfg(unix)]
    {
        use memmap2::Advice;
        // mmap advice: 환경변수 3개로 선택 적용 (우선순위: RANDOM > SEQUENTIAL > DONTDUMP)
        if std::env::var("MMAP_ADVICE_RANDOM").is_ok() {
            println!("Applying MADV_RANDOM advice to mmap");
            mmap.advise(Advice::Random)?;
        } else if std::env::var("MMAP_ADVICE_SEQUENTIAL").is_ok() {
            println!("Applying MADV_SEQUENTIAL advice to mmap");
            mmap.advise(Advice::Sequential)?;
        } else if std::env::var("MMAP_ADVICE_DONTDUMP").is_ok() {
            #[cfg(target_os = "linux")]
            {
                println!("Applying MADV_DONTDUMP advice to mmap");
                mmap.advise(Advice::DontDump)?;
            }
            #[cfg(not(target_os = "linux"))]
            {
                println!("MADV_DONTDUMP advice is only supported on Linux. Skipping.");
            }
        }
    }
    
    let fm_index = FmIndex::<u32, B>::load(&mmap)?;
    let load_time = load_start_time.elapsed().as_nanos();
    
    let result_path = data_dir.join(format!("{}-results.txt", blob_stem));

    // Locate 처리 시간 측정
    let locate_start_time = std::time::Instant::now();
    let pattern_path = data_dir.join("pattern.txt");
    let reader = create_pattern_reader(&pattern_path)?;
    let mut writer = create_result_writer(&result_path)?;

    reader.lines().for_each(|line| {
        let pattern = line.unwrap();
        let locations = fm_index.locate_pattern(pattern.as_bytes());
        write_locations_to_file(&mut writer, &locations).unwrap();
    });

    writer.flush()?;
    let locate_time = locate_start_time.elapsed().as_nanos();
    
    println!("Blob loading time: {} ns", load_time);
    println!("Locate processing time: {} ns", locate_time);
    
    Ok(result_path)
}

pub fn locate_patterns(data_dir: &PathBuf, treat_t_as_wildcard: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("Locating patterns with sview-fmindex mmap version...");

    if treat_t_as_wildcard {
        let block2_path = data_dir.join("sview-mmap-block2.blob");
        if block2_path.exists() {
            println!("Using blob file: {}", block2_path.display());
            println!("Treat T as wildcard: true (Block2)");
            let result_path = locate_and_write_results::<Block2<u64>>(data_dir, "sview-mmap-block2")?;
            println!("Results saved to: {}", result_path.display());
        } else {
            return Err("Block2 blob file (treat_t_as_wildcard=true) not found".into());
        }
    } else {
        let block3_path = data_dir.join("sview-mmap-block3.blob");
        if block3_path.exists() {
            println!("Using blob file: {}", block3_path.display());
            println!("Treat T as wildcard: false (Block3)");
            let result_path = locate_and_write_results::<Block3<u64>>(data_dir, "sview-mmap-block3")?;
            println!("Results saved to: {}", result_path.display());
        } else {
            return Err("Block3 blob file (treat_t_as_wildcard=false) not found".into());
        }
    }

    Ok(())
} 