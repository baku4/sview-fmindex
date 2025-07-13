use std::path::PathBuf;
use std::io::{BufRead, Write};
use sview_fmindex::{FmIndex, Block, blocks::{Block2, Block3}};
use crate::locate::{create_pattern_reader, create_result_writer, write_locations_to_file};

fn locate_and_write_results<B: Block>(
    data_dir: &PathBuf,
    blob_stem: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let blob_path = data_dir.join(format!("{}.blob", blob_stem));
    let blob = std::fs::read(&blob_path)?;
    let fm_index = FmIndex::<u32, B>::load(&blob)?;
    let result_path = data_dir.join(format!("{}-results.txt", blob_stem));

    let pattern_path = data_dir.join("pattern.txt");
    let reader = create_pattern_reader(&pattern_path)?;
    let mut writer = create_result_writer(&result_path)?;

    reader.lines().for_each(|line| {
        let pattern = line.unwrap();
        let locations = fm_index.locate_pattern(pattern.as_bytes());
        write_locations_to_file(&mut writer, &locations).unwrap();
    });

    writer.flush()?;
    Ok(result_path)
}

pub fn locate_patterns(data_dir: &PathBuf, treat_t_as_wildcard: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("Locating patterns with sview-fmindex memory version...");

    if treat_t_as_wildcard {
        let block2_path = data_dir.join("sview-memory-block2.blob");
        if block2_path.exists() {
            println!("Using blob file: {}", block2_path.display());
            println!("Treat T as wildcard: true (Block2)");
            let result_path = locate_and_write_results::<Block2<u64>>(data_dir, "sview-memory-block2")?;
            println!("Results saved to: {}", result_path.display());
        } else {
            return Err("Block2 blob file (treat_t_as_wildcard=true) not found".into());
        }
    } else {
        let block3_path = data_dir.join("sview-memory-block3.blob");
        if block3_path.exists() {
            println!("Using blob file: {}", block3_path.display());
            println!("Treat T as wildcard: false (Block3)");
            let result_path = locate_and_write_results::<Block3<u64>>(data_dir, "sview-memory-block3")?;
            println!("Results saved to: {}", result_path.display());
        } else {
            return Err("Block3 blob file (treat_t_as_wildcard=false) not found".into());
        }
    }

    Ok(())
} 