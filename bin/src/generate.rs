use std::fs;
use std::path::PathBuf;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

pub fn generate_data(
    data_dir: PathBuf,
    text_length: usize,
    pattern_length: usize,
    pattern_count: usize,
    seed: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = std::time::Instant::now();
    
    println!("Generating benchmark data...");
    println!("Output directory: {}", data_dir.display());
    println!("Text length: {}", text_length);
    println!("Pattern length: {}", pattern_length);
    println!("Pattern count: {}", pattern_count);
    println!("Seed: {}", seed);
    println!();

    // 출력 디렉토리 생성
    if !data_dir.exists() {
        fs::create_dir_all(&data_dir)?;
    }

    // RNG 초기화
    let mut rng = StdRng::seed_from_u64(seed);

    // ACGT 문자 생성
    let nucleotides = [b'A', b'C', b'G', b'T'];

    // text.txt 파일 생성 (개행 없음)
    let text_path = data_dir.join("text.txt");
    let text: Vec<u8> = (0..text_length)
        .map(|_| nucleotides[rng.gen_range(0..4)])
        .collect();
    fs::write(&text_path, &text)?;
    println!("Text file created: {}", text_path.display());

    // pattern.txt 파일 생성 (개행으로 구분)
    let pattern_path = data_dir.join("pattern.txt");
    let patterns: Vec<Vec<u8>> = (0..pattern_count)
        .map(|_| {
            (0..pattern_length)
                .map(|_| nucleotides[rng.gen_range(0..4)])
                .collect()
        })
        .collect();

    let pattern_content = patterns
        .iter()
        .map(|p| String::from_utf8_lossy(p))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(&pattern_path, pattern_content)?;
    println!("Pattern file created: {}", pattern_path.display());

    let total_time = start_time.elapsed().as_nanos();
    println!("Data generation completed successfully!");
    println!("Total time: {} ns", total_time);
    
    Ok(())
} 