use std::fs;
use std::path::PathBuf;
use std::io::{BufReader, BufWriter, Write};
use std::process::Command;

pub mod sview_memory;
pub mod sview_mmap;
pub mod crate_lt_fm_index;

/// Drop page caches using sudo (Linux only)
/// This requires passwordless sudo access for the command
fn drop_page_caches() -> Result<(), Box<dyn std::error::Error>> {
    println!("Dropping page caches (requires sudo)...");

    // sync first to flush dirty pages
    Command::new("sync").status()?;

    // Drop caches: 3 = free pagecache, dentries and inodes
    let status = Command::new("sudo")
        .args(["sh", "-c", "echo 3 > /proc/sys/vm/drop_caches"])
        .status()?;

    if !status.success() {
        return Err("Failed to drop page caches. Make sure you have sudo access.".into());
    }

    println!("Page caches dropped successfully.");
    Ok(())
}

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

pub fn locate_patterns(
    algorithm: Algorithm,
    data_dir: PathBuf,
    treat_t_as_wildcard: bool,
    drop_caches: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let total_start_time = std::time::Instant::now();

    println!("Locating patterns...");
    println!("Algorithm: {:?}", algorithm);
    println!("Data directory: {}", data_dir.display());
    println!("Treat T as wildcard: {}", treat_t_as_wildcard);
    println!("Drop caches: {}", drop_caches);
    println!();

    // 패턴 파일 읽기
    let pattern_path = data_dir.join("pattern.txt");
    if !pattern_path.exists() {
        return Err(format!("Pattern file not found: {}", pattern_path.display()).into());
    }

    // Drop page caches if requested
    if drop_caches {
        drop_page_caches()?;
    }

    // 알고리즘별로 패턴 검색 실행
    let locate_start_time = std::time::Instant::now();
    match algorithm {
        Algorithm::LtFmIndex => {
            crate_lt_fm_index::locate_patterns(&data_dir, treat_t_as_wildcard)?;
        }
        Algorithm::SviewMemory => {
            sview_memory::locate_patterns(&data_dir, treat_t_as_wildcard)?;
        }
        Algorithm::SviewMmap => {
            sview_mmap::locate_patterns(&data_dir, treat_t_as_wildcard)?;
        }
    }
    let locate_time = locate_start_time.elapsed().as_nanos();

    let total_time = total_start_time.elapsed().as_nanos();
    println!("Pattern location completed successfully!");
    println!("Locate time: {} ns", locate_time);
    println!("Total time: {} ns", total_time);

    Ok(())
}

// 공통으로 사용할 BufReader 생성 함수
pub fn create_pattern_reader(pattern_path: &PathBuf) -> Result<BufReader<fs::File>, Box<dyn std::error::Error>> {
    let file = fs::File::open(pattern_path)?;
    let reader = BufReader::with_capacity(64 * 1024, file);
    Ok(reader)
}

// 공통으로 사용할 BufWriter 생성 함수
pub fn create_result_writer(result_path: &PathBuf) -> Result<BufWriter<fs::File>, Box<dyn std::error::Error>> {
    let file = fs::File::create(result_path)?;
    let writer = BufWriter::with_capacity(64 * 1024, file);
    Ok(writer)
}

// 공통으로 사용할 locations 결과 쓰기 함수
pub fn write_locations_to_file(writer: &mut BufWriter<fs::File>, locations: &[u32]) -> Result<(), Box<dyn std::error::Error>> {
    let result_line = locations
        .iter()
        .map(|pos| pos.to_string())
        .collect::<Vec<_>>()
        .join(",");
    
    writeln!(writer, "{}", result_line)?;
    Ok(())
} 