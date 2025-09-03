use std::fs;
use std::path::PathBuf;

pub mod sview_memory;
pub mod sview_mmap;
pub mod crate_lt_fm_index;

#[derive(Debug, Clone, Copy)]
pub enum Algorithm {
    LtFmIndex,
    SviewMemory,
    SviewMmap,
}

impl std::str::FromStr for Algorithm {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "lt-fm-index" => Ok(Algorithm::LtFmIndex),
            "sview-memory" => Ok(Algorithm::SviewMemory),
            "sview-mmap" => Ok(Algorithm::SviewMmap),
            _ => Err(format!("Unknown algorithm: {}", s)),
        }
    }
}

const SYMBOLS_ACG: &[&[u8]] = &[b"Aa", b"Cc", b"Gg"];
const SYMBOLS_ACGT: &[&[u8]] = &[b"Aa", b"Cc", b"Gg", b"Tt"];
const SYMBOLS_ACGTN: &[&[u8]] = &[b"Aa", b"Cc", b"Gg", b"Tt", b"Nn"];

pub fn build_indices(
    algorithm: Algorithm,
    data_dir: PathBuf,
    sasr: usize,
    klts: usize,
    treat_t_as_wildcard: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let total_start_time = std::time::Instant::now();
    
    println!("Building FM-index...");
    println!("Algorithm: {:?}", algorithm);
    println!("Data directory: {}", data_dir.display());
    println!("SASR: {}", sasr);
    println!("KLTS: {}", klts);
    println!("Treat T as wildcard: {}", treat_t_as_wildcard);
    println!();

    // 데이터 파일 읽기
    let text_path = data_dir.join("text.txt");

    if !text_path.exists() {
        return Err(format!("Text file not found: {}", text_path.display()).into());
    }

    let text = fs::read(&text_path)?;
    println!("Loaded text: {} bytes", text.len());

    // 알고리즘별로 인덱스 빌드
    match algorithm {
        Algorithm::LtFmIndex => {
            crate_lt_fm_index::build_index(&text, &data_dir, sasr, klts, treat_t_as_wildcard)?;
        }
        Algorithm::SviewMemory => {
            sview_memory::build_index(&text, &data_dir, sasr, klts, treat_t_as_wildcard)?;
        }
        Algorithm::SviewMmap => {
            sview_mmap::build_index(&text, &data_dir, sasr, klts, treat_t_as_wildcard)?;
        }
    }
    
    let total_time = total_start_time.elapsed().as_nanos();
    println!("Index building completed successfully!");
    println!("Total time: {} ns", total_time);
    
    Ok(())
} 