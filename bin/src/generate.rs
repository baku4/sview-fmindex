use std::fs;
use std::path::PathBuf;
use rand::prelude::*;
use rand::SeedableRng;

pub fn generate_data(
    data_dir: PathBuf,
    text_length: usize,
    pattern_length: usize,
    pattern_count: usize,
    seed: u64,
) -> Result<(), Box<dyn std::error::Error>> {
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

    // 시드 설정으로 랜덤 생성기 초기화
    let mut rng = StdRng::seed_from_u64(seed);

    // ACGT 문자만 사용하여 텍스트 생성
    let alphabet = b"ACGT";
    let text: Vec<u8> = (0..text_length)
        .map(|_| alphabet[rng.gen_range(0..4)])
        .collect();

    // 텍스트에서 패턴 생성 (슬라이스 방식)
    let mut patterns = Vec::new();
    for _ in 0..pattern_count {
        if text_length < pattern_length {
            // 텍스트가 패턴보다 작으면 랜덤 생성
            let pattern: Vec<u8> = (0..pattern_length)
                .map(|_| alphabet[rng.gen_range(0..4)])
                .collect();
            patterns.push(pattern);
        } else {
            // 텍스트에서 랜덤 위치에서 패턴 추출
            let start = rng.gen_range(0..=text_length - pattern_length);
            let pattern = text[start..start + pattern_length].to_vec();
            patterns.push(pattern);
        }
    }

    // text.txt 파일 생성 (개행 없음)
    let text_path = data_dir.join("text.txt");
    fs::write(&text_path, &text)?;
    println!("Text file created: {}", text_path.display());

    // pattern.txt 파일 생성 (개행으로 구분)
    let pattern_path = data_dir.join("pattern.txt");
    let pattern_content = patterns
        .iter()
        .map(|p| String::from_utf8_lossy(p))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(&pattern_path, pattern_content)?;
    println!("Pattern file created: {}", pattern_path.display());

    println!("Data generation completed successfully!");
    Ok(())
} 